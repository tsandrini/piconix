use crate::{NixExpr, NixStringPart, NixValue};
use indexmap::IndexMap;

// Scope now uses owned Strings for keys to allow for dynamic extension.
pub type Scope = IndexMap<String, NixExpr>;

#[derive(Debug, PartialEq)]
pub enum EvaluationError {
    UndefinedVariable(String),
    TypeMismatch(String),
}

pub fn nix_eval(expr: &NixExpr, scope: &Scope) -> Result<NixExpr, EvaluationError> {
    match expr {
        NixExpr::Value(_) => Ok(expr.clone()),

        NixExpr::Ref(name) => scope
            .get(name)
            .cloned()
            .ok_or_else(|| EvaluationError::UndefinedVariable(name.clone())),

        NixExpr::LetIn { bindings, body } => {
            // NOTE: This is a simplified, strict evaluation of let bindings.
            // Nix's let bindings are lazy and mutually recursive. This implementation
            // only supports bindings that refer to previously defined bindings in the same block.
            let mut extended_scope = scope.clone();
            for (key, value_expr) in bindings {
                let evaluated_value = nix_eval(value_expr, &extended_scope)?;
                extended_scope.insert(key.clone(), evaluated_value);
            }
            // The body is evaluated in the scope containing all let-bindings.
            nix_eval(body, &extended_scope)
        }

        NixExpr::List(items) => {
            let evaluated_items = items
                .iter()
                .map(|item| nix_eval(item, scope))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(NixExpr::List(evaluated_items))
        }

        NixExpr::AttrSet {
            recursive,
            bindings,
        } => {
            // TODO: The `recursive` flag is not yet handled.
            let evaluated_bindings = bindings
                .iter()
                .map(|(key, value)| {
                    let evaluated_value = nix_eval(value, scope)?;
                    Ok((key.clone(), evaluated_value))
                })
                .collect::<Result<IndexMap<_, _>, _>>()?;
            Ok(NixExpr::AttrSet {
                recursive: *recursive,
                bindings: evaluated_bindings,
            })
        }
        NixExpr::SearchPath(path) => Ok(NixExpr::SearchPath(path.clone())),

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
                    }
                }
            }
            Ok(NixExpr::Value(NixValue::String(result)))
        }
    }
}
