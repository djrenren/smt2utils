// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The visiting traits expected by the SMT2 parser.

use crate::{Binary, Decimal, Hexadecimal, Numeral};

pub trait ConstantVisitor {
    type T;

    fn visit_numeral_constant(&mut self, value: Numeral) -> Self::T;
    fn visit_decimal_constant(&mut self, value: Decimal) -> Self::T;
    fn visit_hexadecimal_constant(&mut self, value: Hexadecimal) -> Self::T;
    fn visit_binary_constant(&mut self, value: Binary) -> Self::T;
    fn visit_string_constant(&mut self, value: String) -> Self::T;
}

pub trait SymbolVisitor {
    type T;

    fn visit_symbol(&mut self, value: String) -> Self::T;
}

pub trait KeywordVisitor {
    type T;

    fn visit_keyword(&mut self, value: String) -> Self::T;
}

pub trait SExprVisitor<Constant, Symbol, Keyword> {
    type T;

    fn visit_constant_s_expr(&mut self, value: Constant) -> Self::T;
    fn visit_symbol_s_expr(&mut self, value: Symbol) -> Self::T;
    fn visit_keyword_s_expr(&mut self, value: Keyword) -> Self::T;
    fn visit_application_s_expr(&mut self, values: Vec<Self::T>) -> Self::T;
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Index<Symbol> {
    Numeral(Numeral),
    Symbol(Symbol),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Identifier<Symbol> {
    Simple {
        symbol: Symbol,
    },
    Indexed {
        symbol: Symbol,
        indices: Vec<Index<Symbol>>,
    },
}

pub trait SortVisitor<Symbol> {
    type T;

    fn visit_simple_sort(&mut self, identifier: Identifier<Symbol>) -> Self::T;
    fn visit_parameterized_sort(
        &mut self,
        identifier: Identifier<Symbol>,
        parameters: Vec<Self::T>,
    ) -> Self::T;
}

pub trait QualIdentifierVisitor<Identifier, Sort> {
    type T;

    fn visit_simple_identifier(&mut self, identifier: Identifier) -> Self::T;
    fn visit_sorted_identifier(&mut self, identifier: Identifier, sort: Sort) -> Self::T;
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum AttributeValue<Constant, Symbol, SExpr> {
    None,
    Constant(Constant),
    Symbol(Symbol),
    SExpr(Vec<SExpr>),
}

pub trait TermVisitor<Constant, QualIdentifier, Keyword, SExpr, Symbol, Sort> {
    type T;

    fn visit_constant(&mut self, constant: Constant) -> Self::T;

    fn visit_qual_identifier(&mut self, qual_identifier: QualIdentifier) -> Self::T;

    fn visit_application(
        &mut self,
        qual_identifier: QualIdentifier,
        arguments: Vec<Self::T>,
    ) -> Self::T;

    fn visit_let(&mut self, var_bindings: Vec<(Symbol, Self::T)>, term: Self::T) -> Self::T;

    fn visit_forall(&mut self, vars: Vec<(Symbol, Sort)>, term: Self::T) -> Self::T;

    fn visit_exists(&mut self, vars: Vec<(Symbol, Sort)>, term: Self::T) -> Self::T;

    fn visit_match(&mut self, term: Self::T, cases: Vec<(Vec<Symbol>, Self::T)>) -> Self::T;

    fn visit_attributes(
        &mut self,
        term: Self::T,
        attributes: Vec<(Keyword, AttributeValue<Constant, Symbol, SExpr>)>,
    ) -> Self::T;
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ConstructorDec<Symbol, Sort> {
    pub symbol: Symbol,
    pub selectors: Vec<(Symbol, Sort)>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct DatatypeDec<Symbol, Sort> {
    pub parameters: Vec<Symbol>,
    pub constructors: Vec<ConstructorDec<Symbol, Sort>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct FunctionDec<Symbol, Sort> {
    pub name: Symbol,
    pub parameters: Vec<(Symbol, Sort)>,
    pub result: Sort,
}

pub trait CommandVisitor<Term, Symbol, Sort, Keyword, Constant, SExpr> {
    type T;

    fn visit_assert(&mut self, term: Term) -> Self::T;

    fn visit_check_sat(&mut self) -> Self::T;

    fn visit_check_sat_assuming(&mut self, literals: Vec<(Symbol, bool)>) -> Self::T;

    fn visit_declare_const(&mut self, symbol: Symbol, sort: Sort) -> Self::T;

    fn visit_declare_datatype(
        &mut self,
        symbol: Symbol,
        datatype: DatatypeDec<Symbol, Sort>,
    ) -> Self::T;

    fn visit_declare_datatypes(
        &mut self,
        datatypes: Vec<(Symbol, Numeral, DatatypeDec<Symbol, Sort>)>,
    ) -> Self::T;

    fn visit_declare_fun(&mut self, symbol: Symbol, parameters: Vec<Sort>, sort: Sort) -> Self::T;

    fn visit_declare_sort(&mut self, symbol: Symbol, arity: Numeral) -> Self::T;

    fn visit_define_fun(&mut self, sig: FunctionDec<Symbol, Sort>, term: Term) -> Self::T;

    fn visit_define_fun_rec(&mut self, sig: FunctionDec<Symbol, Sort>, term: Term) -> Self::T;

    fn visit_define_funs_rec(&mut self, funs: Vec<(FunctionDec<Symbol, Sort>, Term)>) -> Self::T;

    fn visit_define_sort(&mut self, symbol: Symbol, parameters: Vec<Symbol>, sort: Sort)
        -> Self::T;

    fn visit_echo(&mut self, value: String) -> Self::T;

    fn visit_exit(&mut self) -> Self::T;

    fn visit_get_assertions(&mut self) -> Self::T;

    fn visit_get_assignment(&mut self) -> Self::T;

    fn visit_get_info(&mut self, info_flag: Keyword) -> Self::T;

    fn visit_get_model(&mut self) -> Self::T;

    fn visit_get_option(&mut self, option: Keyword) -> Self::T;

    fn visit_get_proof(&mut self) -> Self::T;

    fn visit_get_unsat_assumptions(&mut self) -> Self::T;

    fn visit_get_unsat_core(&mut self) -> Self::T;

    fn visit_get_value(&mut self, terms: Vec<Term>) -> Self::T;

    fn visit_pop(&mut self, num: Numeral) -> Self::T;

    fn visit_push(&mut self, num: Numeral) -> Self::T;

    fn visit_reset(&mut self) -> Self::T;

    fn visit_reset_assertions(&mut self) -> Self::T;

    fn visit_set_info(
        &mut self,
        key: Keyword,
        value: AttributeValue<Constant, Symbol, SExpr>,
    ) -> Self::T;

    fn visit_set_logic(&mut self, logic: Symbol) -> Self::T;

    fn visit_set_option(
        &mut self,
        key: Keyword,
        value: AttributeValue<Constant, Symbol, SExpr>,
    ) -> Self::T;
}

pub trait SMT2Visitor:
    ConstantVisitor<T = <Self as SMT2Visitor>::Constant>
    + SymbolVisitor<T = <Self as SMT2Visitor>::Symbol>
    + KeywordVisitor<T = <Self as SMT2Visitor>::Keyword>
    + SExprVisitor<
        <Self as SMT2Visitor>::Constant,
        <Self as SMT2Visitor>::Symbol,
        <Self as SMT2Visitor>::Keyword,
        T = <Self as SMT2Visitor>::SExpr,
    > + QualIdentifierVisitor<
        Identifier<<Self as SMT2Visitor>::Symbol>,
        <Self as SMT2Visitor>::Sort,
        T = <Self as SMT2Visitor>::QualIdentifier,
    > + SortVisitor<<Self as SMT2Visitor>::Symbol, T = <Self as SMT2Visitor>::Sort>
    + TermVisitor<
        <Self as SMT2Visitor>::Constant,
        <Self as SMT2Visitor>::QualIdentifier,
        <Self as SMT2Visitor>::Keyword,
        <Self as SMT2Visitor>::SExpr,
        <Self as SMT2Visitor>::Symbol,
        <Self as SMT2Visitor>::Sort,
        T = <Self as SMT2Visitor>::Term,
    > + CommandVisitor<
        <Self as SMT2Visitor>::Term,
        <Self as SMT2Visitor>::Symbol,
        <Self as SMT2Visitor>::Sort,
        <Self as SMT2Visitor>::Keyword,
        <Self as SMT2Visitor>::Constant,
        <Self as SMT2Visitor>::SExpr,
        T = <Self as SMT2Visitor>::Command,
    >
{
    type Constant;
    type QualIdentifier;
    type Keyword;
    type Sort;
    type SExpr;
    type Symbol;
    type Term;
    type Command;
}

impl<Symbol> std::fmt::Display for Index<Symbol>
where
    Symbol: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ⟨numeral⟩ | ⟨symbol⟩
        match self {
            Index::Numeral(num) => write!(f, "{}", num),
            Index::Symbol(sym) => write!(f, "{}", sym),
        }
    }
}

impl<Symbol> std::fmt::Display for Identifier<Symbol>
where
    Symbol: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ⟨symbol⟩ | ( _ ⟨symbol⟩ ⟨index⟩+ )
        match self {
            Identifier::Simple { symbol } => write!(f, "{}", symbol),
            Identifier::Indexed { symbol, indices } => write!(
                f,
                "(_ {} {})",
                symbol,
                indices
                    .iter()
                    .map(|i| format!("{}", i))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        }
    }
}

impl<Constant, Symbol, SExpr> std::fmt::Display for AttributeValue<Constant, Symbol, SExpr>
where
    Constant: std::fmt::Display,
    Symbol: std::fmt::Display,
    SExpr: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ⟨spec_constant⟩ | ⟨symbol⟩ | ( ⟨s_expr⟩∗ ) |
        use AttributeValue::*;
        match self {
            None => Ok(()),
            Constant(c) => write!(f, " {}", c),
            Symbol(s) => write!(f, " {}", s),
            SExpr(values) => write!(
                f,
                " ({})",
                values
                    .iter()
                    .map(|i| format!("{}", i))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        }
    }
}

impl<Symbol, Sort> std::fmt::Display for ConstructorDec<Symbol, Sort>
where
    Symbol: std::fmt::Display,
    Sort: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ( ⟨symbol⟩ ⟨selector_dec⟩∗ )
        write!(
            f,
            "({} {})",
            self.symbol,
            self.selectors
                .iter()
                .map(|(symbol, sort)| format!("({} {})", symbol, sort))
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

impl<Symbol, Sort> std::fmt::Display for DatatypeDec<Symbol, Sort>
where
    Symbol: std::fmt::Display,
    Sort: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ( ⟨constructor_dec⟩+ ) | ( par ( ⟨symbol⟩+ ) ( ⟨constructor_dec⟩+ ) )
        if self.parameters.is_empty() {
            write!(
                f,
                "({})",
                self.constructors
                    .iter()
                    .map(|c| format!("{}", c))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        } else {
            let symbols = format!(
                "({})",
                self.parameters
                    .iter()
                    .map(|c| format!("{}", c))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            let constructors = format!(
                "({})",
                self.constructors
                    .iter()
                    .map(|c| format!("{}", c))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            write!(f, "(par {} {})", symbols, constructors)
        }
    }
}

impl<Symbol, Sort> std::fmt::Display for FunctionDec<Symbol, Sort>
where
    Symbol: std::fmt::Display,
    Sort: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ⟨symbol⟩ ( ⟨sorted_var⟩∗ ) ⟨sort⟩
        let params = self
            .parameters
            .iter()
            .map(|(symbol, sort)| format!("({} {})", symbol, sort))
            .collect::<Vec<_>>()
            .join(" ");
        write!(f, "{} ({}) {}", self.name, params, self.result)
    }
}
