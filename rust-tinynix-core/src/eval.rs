use crate::{NixExpr, NixStringPart, NixValue};
use indexmap::IndexMap;

pub type Scope<'a> = IndexMap<&'a str, NixExpr>;

#[derive(Debug, PartialEq)]
pub enum EvaluationError {
    UndefinedVariable(String),
    TypeMismatch(String),
}

pub fn nix_eval<'a>(expr: &NixExpr, scope: &Scope<'a>) -> Result<NixExpr, EvaluationError> {
    match expr {
        // NixExpr::Value(NixValue::Null) => Ok(NixExpr::Value(NixValue::Null)),
        NixExpr::Value(_) => Ok(expr.clone()),

        NixExpr::Ref(name) => scope
            .get(name.as_str())
            .cloned()
            .ok_or_else(|| EvaluationError::UndefinedVariable(name.clone())),

        NixExpr::List(items) => {
            let evaluated_items = items
                .iter()
                .map(|item| nix_eval(item, scope))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(NixExpr::List(evaluated_items))
        }

        NixExpr::AttrSet(bindings) => {
            let evaluated_bindings = bindings
                .iter()
                .map(|(key, value)| {
                    let evaluated_value = nix_eval(value, scope)?;
                    Ok((key.clone(), evaluated_value))
                })
                .collect::<Result<IndexMap<_, _>, _>>()?;
            Ok(NixExpr::AttrSet(evaluated_bindings))
        }
        NixExpr::SearchPath(path) => {
            // For now, we just return the search path as is.
            // In a real implementation, you might resolve this to a list of paths.
            Ok(NixExpr::SearchPath(path.clone()))
        }

        NixExpr::InterpolatedString(parts) => {
            let mut result = String::new();
            for part in parts {
                match part {
                    NixStringPart::Literal(s) => result.push_str(s),
                    NixStringPart::Interpolation(expr_to_interpolate) => {
                        let evaluated_expr = nix_eval(expr_to_interpolate, scope)?;
                        if let NixExpr::Value(NixValue::String(s)) = evaluated_expr {
                            result.push_str(&s);
                        } else {
                            return Err(EvaluationError::TypeMismatch(
                                "Expected a string for interpolation.".to_string(),
                            ));
                        }
                        // let s = expr_to_string(&evaluated_expr)?;
                        // result.push_str(&s);
                    }
                }
            }
            Ok(NixExpr::Value(NixValue::String(result)))
        }
    }
}

// This is a great idea, but cppnix doesn't do this.
#[allow(dead_code)]
fn expr_to_string(expr: &NixExpr) -> Result<String, EvaluationError> {
    match expr {
        NixExpr::Value(value) => match value {
            NixValue::String(s) => Ok(s.clone()),
            NixValue::Int(i) => Ok(i.to_string()),
            NixValue::Float(f) => Ok(f.to_string()),
            NixValue::Bool(b) => Ok(b.to_string()),
            NixValue::Null => Ok("null".to_string()),
            NixValue::Path(p) => p
                .to_str()
                .map(|s| s.to_string())
                .ok_or_else(|| EvaluationError::TypeMismatch("Invalid path".to_string())),
        },
        _ => Err(EvaluationError::TypeMismatch(
            "Cannot interpolate this expression type into a string.".to_string(),
        )),
    }
}
