use crate::{NixBinaryOp, NixExpr, NixStringPart, NixUnaryOp, NixValue};
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_token_stream(ast: &NixExpr) -> TokenStream {
    match ast {
        NixExpr::Value(value) => match value {
            NixValue::Int(i) => {
                quote! { ::rust_tinynix::NixExpr::Value(::rust_tinynix::NixValue::Int(#i)) }
            }
            NixValue::Float(f) => {
                quote! { ::rust_tinynix::NixExpr::Value(::rust_tinynix::NixValue::Float(#f)) }
            }
            NixValue::Bool(b) => {
                quote! { ::rust_tinynix::NixExpr::Value(::rust_tinynix::NixValue::Bool(#b)) }
            }
            NixValue::String(s) => {
                quote! { ::rust_tinynix::NixExpr::Value(::rust_tinynix::NixValue::String(#s.to_string())) }
            }
            NixValue::Null => {
                quote! { ::rust_tinynix::NixExpr::Value(::rust_tinynix::NixValue::Null) }
            }
            NixValue::Path(p) => {
                let path_str = p.to_str().expect("Path is not valid UTF-8");
                quote! { ::rust_tinynix::NixExpr::Value(::rust_tinynix::NixValue::Path(::std::path::PathBuf::from(#path_str))) }
            }
        },
        NixExpr::UnaryOp { op, expr } => {
            let expr_ast = generate_token_stream(expr);
            let op_token = match op {
                NixUnaryOp::Neg => quote! { ::rust_tinynix::NixUnaryOp::Neg },
                NixUnaryOp::Not => quote! { ::rust_tinynix::NixUnaryOp::Not },
            };
            quote! {
                ::rust_tinynix::NixExpr::UnaryOp {
                    op: #op_token,
                    expr: Box::new(#expr_ast),
                }
            }
        }
        NixExpr::BinaryOp { op, left, right } => {
            let left_ast = generate_token_stream(left);
            let right_ast = generate_token_stream(right);
            let op_token = match op {
                NixBinaryOp::Add => quote! { ::rust_tinynix::NixBinaryOp::Add },
                NixBinaryOp::Sub => quote! { ::rust_tinynix::NixBinaryOp::Sub },
            };
            quote! {
                ::rust_tinynix::NixExpr::BinaryOp {
                    op: #op_token,
                    left: Box::new(#left_ast),
                    right: Box::new(#right_ast),
                }
            }
        }
        NixExpr::InterpolatedString(parts) => {
            let quoted_parts = parts.iter().map(|part| match part {
                NixStringPart::Literal(s) => {
                    quote! { ::rust_tinynix::NixStringPart::Literal(#s.to_string()) }
                }
                NixStringPart::Interpolation(ast) => {
                    let quoted_ast = generate_token_stream(ast);
                    quote! { ::rust_tinynix::NixStringPart::Interpolation(Box::new(#quoted_ast)) }
                }
            });
            quote! { ::rust_tinynix::NixExpr::InterpolatedString(vec![#(#quoted_parts),*]) }
        }
        NixExpr::SearchPath(s) => {
            quote! { ::rust_tinynix::NixExpr::SearchPath(#s.to_string()) }
        }
        NixExpr::Ref(s) => quote! { ::rust_tinynix::NixExpr::Ref(#s.to_string()) },
        NixExpr::List(items) => {
            let quoted_items = items.iter().map(generate_token_stream);
            quote! { ::rust_tinynix::NixExpr::List(vec![#(#quoted_items),*]) }
        }
        NixExpr::With { environment, body } => {
            let env_ast = generate_token_stream(environment);
            let body_ast = generate_token_stream(body);
            quote! {
                ::rust_tinynix::NixExpr::With {
                    environment: Box::new(#env_ast),
                    body: Box::new(#body_ast),
                }
            }
        }
        NixExpr::LetIn { bindings, body } => {
            let quoted_bindings = bindings.iter().map(|(k, v)| {
                let key_str = k;
                let val_ast = generate_token_stream(v);
                quote! { (#key_str.to_string(), #val_ast) }
            });
            let body_ast = generate_token_stream(body);
            quote! {
                ::rust_tinynix::NixExpr::LetIn {
                    bindings: vec![#(#quoted_bindings),*].into_iter().collect(),
                    body: Box::new(#body_ast),
                }
            }
        }
        NixExpr::AttrSet {
            recursive,
            bindings,
        } => {
            let quoted_bindings = bindings.iter().map(|(k, v)| {
                let key_str = k;
                let val_ast = generate_token_stream(v);
                quote! { (#key_str.to_string(), #val_ast) }
            });
            quote! {
                ::rust_tinynix::NixExpr::AttrSet {
                    recursive: #recursive,
                    bindings: vec![#(#quoted_bindings),*].into_iter().collect()
                }
            }
        }
    }
}
