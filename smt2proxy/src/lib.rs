// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

#![forbid(unsafe_code)]

use smt2parser::{concrete::*, Numeral};
use std::{
    collections::BTreeMap,
    io::Write,
    sync::{Arc, Mutex},
};
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
pub struct CommandProcessorConfig {
    #[structopt(long, env, parse(from_os_str))]
    smt2proxy_log_path: Option<std::path::PathBuf>,

    #[structopt(long, env)]
    smt2proxy_delay: bool,

    #[structopt(long, env)]
    smt2proxy_shuffle: Option<u64>,

    #[structopt(long, env)]
    smt2proxy_seed: Option<u64>,

    #[structopt(long, env, parse(try_from_str = parse_smt2_options), default_value = "")]
    smt2proxy_options: BTreeMap<Keyword, AttributeValue>,
}

// "key1=value1:key2=value2..."
fn parse_smt2_options(options: &str) -> Result<BTreeMap<Keyword, AttributeValue>, String> {
    let mut map = BTreeMap::new();
    if options.is_empty() {
        return Ok(map);
    }
    for option in options.split(':') {
        let mut it = option.split('=');
        let key = Keyword(
            it.next()
                .ok_or_else(|| "invalid option: missing keys".to_string())?
                .to_string(),
        );
        let value = smt2parser::parse_simple_attribute_value(
            it.next()
                .ok_or_else(|| format!("invalid option: missing value for key '{}'", key))?,
        )
        .ok_or_else(|| format!("invalid option: incorrect value for key '{}'", key))?;
        map.insert(key, value);
    }
    Ok(map)
}

#[derive(Debug)]
pub struct CommandProcessor {
    logger: Option<Arc<Mutex<std::fs::File>>>,
    delay: bool,
    shuffle: Option<rand::rngs::StdRng>,
    options: BTreeMap<Keyword, AttributeValue>,
    has_sent_initial_commands: bool,
    clauses: Vec<Command>,
}

impl From<CommandProcessorConfig> for CommandProcessor {
    fn from(config: CommandProcessorConfig) -> Self {
        use rand::SeedableRng;

        let mut log = config
            .smt2proxy_log_path
            .as_ref()
            .map(|path| std::fs::File::create(path).expect("Failed to open log file"));
        if let Some(f) = &mut log {
            writeln!(f, "; smt2proxy config: {:?}", config).unwrap();
        }

        let delay = config.smt2proxy_shuffle.is_some() || config.smt2proxy_delay;
        let shuffle = config
            .smt2proxy_shuffle
            .map(rand::rngs::StdRng::seed_from_u64);
        let mut options = config.smt2proxy_options;
        if let Some(seed) = config.smt2proxy_seed {
            options.insert(
                Keyword("random-seed".into()),
                AttributeValue::Constant(Constant::Numeral(Numeral::from(seed))),
            );
            options.insert(
                Keyword("smt.random_seed".into()),
                AttributeValue::Constant(Constant::Numeral(Numeral::from(seed))),
            );
        }
        Self {
            logger: log.map(|f| Arc::new(Mutex::new(f))),
            delay,
            shuffle,
            options,
            has_sent_initial_commands: false,
            clauses: Vec::new(),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum CommandKind {
    Definition,
    Clause,
    Skipped,
    Forcing,
    Resetting,
}

impl CommandProcessor {
    pub fn logger(&self) -> &Option<Arc<Mutex<std::fs::File>>> {
        &self.logger
    }

    fn analyze_command(&self, command: &Command) -> CommandKind {
        use Command::*;
        use CommandKind::*;

        match command {
            Assert { .. } => Clause,

            CheckSat
            | CheckSatAssuming(_)
            | Echo { .. }
            | Exit
            | GetAssertions
            | GetAssignment
            | GetInfo(_)
            | GetModel
            | GetOption(_)
            | GetProof
            | GetUnsatAssumptions
            | GetUnsatCore
            | GetValue(_)
            | Push(_)
            | Pop(_)
            | SetInfo(_, _)
            | SetLogic(_) => Forcing,

            SetOption(keyword, _) => {
                if self.options.contains_key(&keyword) {
                    Skipped
                } else {
                    Forcing
                }
            }

            Reset | ResetAssertions => Resetting,

            _ => Definition,
        }
    }

    fn initial_commands_if_needed(&mut self) -> Vec<Command> {
        let mut commands = Vec::new();
        if self.has_sent_initial_commands {
            return commands;
        }
        for (key, value) in &self.options {
            commands.push(Command::SetOption(key.clone(), value.clone()));
        }
        self.has_sent_initial_commands = true;
        commands
    }

    pub fn process(&mut self, command: Command) -> Vec<Command> {
        use CommandKind::*;

        let mut commands = self.initial_commands_if_needed();
        let kind = self.analyze_command(&command);
        if self.delay {
            match kind {
                Clause => {
                    self.clauses.push(command);
                }
                Skipped => (),
                Definition => {
                    commands.push(command);
                }
                Resetting => {
                    commands.extend(self.flush());
                    commands.push(command);
                    self.has_sent_initial_commands = false;
                }
                Forcing => {
                    commands.extend(self.flush());
                    commands.push(command);
                }
            }
        } else {
            match kind {
                Skipped => (),
                Resetting => {
                    commands.push(command);
                    self.has_sent_initial_commands = false;
                }
                _ => {
                    commands.push(command);
                }
            }
        };
        if let Some(logger) = &self.logger {
            let mut f = logger.lock().unwrap();
            for command in &commands {
                writeln!(f, "{}", command).expect("Failed to write to log file");
            }
        }
        commands
    }

    fn flush(&mut self) -> Vec<Command> {
        use rand::prelude::SliceRandom;

        let mut r = std::mem::take(&mut self.clauses);
        if let Some(rng) = &mut self.shuffle {
            r.shuffle(rng);
        }
        r
    }

    pub fn stop(&mut self) -> Vec<Command> {
        self.flush()
    }
}
