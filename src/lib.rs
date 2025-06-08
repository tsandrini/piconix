use indexmap::IndexMap;

#[derive(Debug, Clone, PartialEq)]
pub enum NixValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NixStringPart {
    Literal(String),
    Interpolation(Box<NixExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NixExpr {
    Value(NixValue),
    InterpolatedString(Vec<NixStringPart>),
    Ref(String),
    List(Vec<NixExpr>),
    AttrSet(IndexMap<String, NixExpr>),
    // Future additions:
    // Function(...)
    // Thunk(...)
}

pub use eval::{EvaluationError, Scope, nix_eval};
pub use rust_nix_macro_impl::nix;

pub mod eval;
