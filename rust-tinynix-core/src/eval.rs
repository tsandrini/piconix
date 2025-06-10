use crate::{NixBinaryOp, NixExpr, NixStringPart, NixUnaryOp, NixValue};
use indexmap::IndexMap;

// Scope now uses owned Strings for keys to allow for dynamic extension.
pub type Scope = IndexMap<String, NixExpr>;

#[derive(Debug, PartialEq)]
pub enum EvaluationError {
    UndefinedVariable(String),
    TypeMismatch(String),
    UnsupportedOperation(String),
}

pub fn nix_eval(expr: &NixExpr, scope: &Scope) -> Result<NixExpr, EvaluationError> {
    match expr {
        NixExpr::Value(_) => Ok(expr.clone()),

        NixExpr::Ref(name) => scope
            .get(name)
            .cloned()
            .ok_or_else(|| EvaluationError::UndefinedVariable(name.clone())),

        NixExpr::UnaryOp { op, expr } => {
            let val = nix_eval(expr, scope)?;
            if let NixExpr::Value(v) = val {
                match op {
                    NixUnaryOp::Neg => match v {
                        NixValue::Int(i) => Ok(NixExpr::Value(NixValue::Int(-i))),
                        NixValue::Float(f) => Ok(NixExpr::Value(NixValue::Float(-f))),
                        _ => Err(EvaluationError::TypeMismatch(
                            "Cannot negate a non-numeric value.".to_string(),
                        )),
                    },
                    NixUnaryOp::Not => match v {
                        NixValue::Bool(b) => Ok(NixExpr::Value(NixValue::Bool(!b))),
                        _ => Err(EvaluationError::TypeMismatch(
                            "Cannot perform logical negation on a non-boolean value.".to_string(),
                        )),
                    },
                }
            } else {
                Err(EvaluationError::TypeMismatch(
                    "Cannot perform unary operation on a non-value.".to_string(),
                ))
            }
        }

        NixExpr::BinaryOp { op, left, right } => {
            let l_val = nix_eval(left, scope)?;
            let r_val = nix_eval(right, scope)?;

            if let (NixExpr::Value(l), NixExpr::Value(r)) = (l_val, r_val) {
                match op {
                    NixBinaryOp::Add => match (l, r) {
                        // Arithmetic
                        (NixValue::Int(a), NixValue::Int(b)) => {
                            Ok(NixExpr::Value(NixValue::Int(a + b)))
                        }
                        (NixValue::Float(a), NixValue::Float(b)) => {
                            Ok(NixExpr::Value(NixValue::Float(a + b)))
                        }
                        (NixValue::Int(a), NixValue::Float(b)) => {
                            Ok(NixExpr::Value(NixValue::Float(a as f64 + b)))
                        }
                        (NixValue::Float(a), NixValue::Int(b)) => {
                            Ok(NixExpr::Value(NixValue::Float(a + b as f64)))
                        }
                        // String concatenation
                        (NixValue::String(a), NixValue::String(b)) => {
                            Ok(NixExpr::Value(NixValue::String(format!("{}{}", a, b))))
                        }
                        // TODO: Add other + operations (lists, paths, etc.)
                        _ => Err(EvaluationError::UnsupportedOperation(
                            "Addition is not supported for these types.".to_string(),
                        )),
                    },
                    NixBinaryOp::Sub => match (l, r) {
                        (NixValue::Int(a), NixValue::Int(b)) => {
                            Ok(NixExpr::Value(NixValue::Int(a - b)))
                        }
                        (NixValue::Float(a), NixValue::Float(b)) => {
                            Ok(NixExpr::Value(NixValue::Float(a - b)))
                        }
                        (NixValue::Int(a), NixValue::Float(b)) => {
                            Ok(NixExpr::Value(NixValue::Float(a as f64 - b)))
                        }
                        (NixValue::Float(a), NixValue::Int(b)) => {
                            Ok(NixExpr::Value(NixValue::Float(a - b as f64)))
                        }
                        _ => Err(EvaluationError::UnsupportedOperation(
                            "Subtraction is not supported for these types.".to_string(),
                        )),
                    },
                }
            } else {
                Err(EvaluationError::TypeMismatch(
                    "Cannot perform binary operation on non-values.".to_string(),
                ))
            }
        }

        NixExpr::With { environment, body } => {
            let evaluated_env = nix_eval(environment, scope)?;

            if let NixExpr::AttrSet { bindings, .. } = evaluated_env {
                let mut extended_scope = scope.clone();
                for (key, value) in bindings {
                    extended_scope.insert(key, value);
                }
                nix_eval(body, &extended_scope)
            } else {
                Err(EvaluationError::TypeMismatch(
                    "Expression in 'with' must evaluate to an attribute set.".to_string(),
                ))
            }
        }

        NixExpr::LetIn { bindings, body } => {
            let mut extended_scope = scope.clone();
            for (key, value_expr) in bindings {
                let evaluated_value = nix_eval(value_expr, &extended_scope)?;
                extended_scope.insert(key.clone(), evaluated_value);
            }
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
                        // This logic should be expanded to handle auto-coercion to string
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
