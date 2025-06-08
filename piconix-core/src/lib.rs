use indexmap::IndexMap;

pub mod codegen;
pub mod eval;
pub mod parser;

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

pub fn nix_str(input: &str) -> Result<NixExpr, String> {
    parser::parse(input).map_err(|e| e.to_string())
}

pub fn nix_file(path: impl AsRef<std::path::Path>) -> Result<NixExpr, String> {
    let path_ref = path.as_ref();
    let content = std::fs::read_to_string(path_ref)
        .map_err(|e| format!("Failed to read file '{}': {}", path_ref.display(), e))?;
    nix_str(&content)
}
