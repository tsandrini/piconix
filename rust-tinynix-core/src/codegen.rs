use crate::{NixExpr, NixStringPart, NixValue};
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
                // TODO doublecheck this
                let path_str = p.to_str().expect("Path is not valid UTF-8");
                quote! { ::rust_tinynix::NixExpr::Value(::rust_tinynix::NixValue::Path(::std::path::PathBuf::from(#path_str))) }
            }
        },
        NixExpr::InterpolatedString(parts) => {
            let quoted_parts = parts.iter().map(|part| match part {
                NixStringPart::Literal(s) => {
                    quote! { ::rust_tinynix::NixStringPart::Literal(#s.to_string()) }
                }
                NixStringPart::Interpolation(ast) => {
                    let quoted_ast = generate_token_stream(ast); // Recursive call
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
        NixExpr::AttrSet(bindings) => {
            let quoted_bindings = bindings.iter().map(|(k, v)| {
                let key_str = k;
                let val_ast = generate_token_stream(v);
                quote! { (#key_str.to_string(), #val_ast) }
            });
            quote! {
                ::rust_tinynix::NixExpr::AttrSet(
                    vec![#(#quoted_bindings),*].into_iter().collect()
                )
            }
        }
    }
}
