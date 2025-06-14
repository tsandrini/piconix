use indexmap::IndexMap;
use std::path::{Path, PathBuf};

pub mod codegen;
pub mod eval;
pub mod parser;

#[derive(Debug, Clone, PartialEq)]
pub enum NixValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Path(PathBuf),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NixStringPart {
    Literal(String),
    Interpolation(Box<NixExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NixUnaryOp {
    Neg, // Arithmetic negation
    Not, // Logical negation
}

#[derive(Debug, Clone, PartialEq)]
pub enum NixBinaryOp {
    Add,
    Sub,
}

// TODO: boxes
#[derive(Debug, Clone, PartialEq)]
pub enum NixExpr {
    Value(NixValue),
    InterpolatedString(Vec<NixStringPart>),
    Ref(String),
    List(Vec<NixExpr>),
    AttrSet {
        recursive: bool,
        bindings: IndexMap<String, NixExpr>,
    },
    UnaryOp {
        op: NixUnaryOp,
        expr: Box<NixExpr>,
    },
    BinaryOp {
        op: NixBinaryOp,
        left: Box<NixExpr>,
        right: Box<NixExpr>,
    },
    SearchPath(String),
    LetIn {
        bindings: IndexMap<String, NixExpr>,
        body: Box<NixExpr>,
    },
    With {
        environment: Box<NixExpr>,
        body: Box<NixExpr>,
    },
    // Future additions:
    // Function(...)
    // Thunk(...)
}

pub fn nix_str(input: &str, root: &Path) -> Result<NixExpr, String> {
    parser::parse(input, root).map_err(|e| e.to_string())
}

pub fn nix_file(path: impl AsRef<std::path::Path>, root: &Path) -> Result<NixExpr, String> {
    let path_ref = path.as_ref();
    let content = std::fs::read_to_string(path_ref)
        .map_err(|e| format!("Failed to read file '{}': {}", path_ref.display(), e))?;
    nix_str(&content, root)
}
