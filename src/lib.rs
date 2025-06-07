#[derive(Debug)]
pub enum Expr {
    Int(i64),
    Var(String),
    List(Vec<Expr>),
    AttrSet(Vec<(String, Expr)>),
}

pub use rust_nix_macro_impl::nix;
