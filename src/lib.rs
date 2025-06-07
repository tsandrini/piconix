#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),
    Interpolation(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    SimpleString(String),
    InterpolatedString(Vec<StringPart>),
    Ref(String),
    List(Vec<Expr>),
    AttrSet(Vec<(String, Expr)>),
}

// Re-export the procedural macro from the implementation crate.
pub use rust_nix_macro_impl::nix;
