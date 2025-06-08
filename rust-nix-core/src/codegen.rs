use crate::{NixExpr, NixStringPart, NixValue};
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_token_stream(ast: &NixExpr) -> TokenStream {
    match ast {
        NixExpr::Value(value) => match value {
            NixValue::Int(i) => {
                quote! { ::rust_nix_macro::NixExpr::Value(::rust_nix_macro::NixValue::Int(#i)) }
            }
            NixValue::Float(f) => {
                quote! { ::rust_nix_macro::NixExpr::Value(::rust_nix_macro::NixValue::Float(#f)) }
            }
            NixValue::Bool(b) => {
                quote! { ::rust_nix_macro::NixExpr::Value(::rust_nix_macro::NixValue::Bool(#b)) }
            }
            NixValue::String(s) => {
                quote! { ::rust_nix_macro::NixExpr::Value(::rust_nix_macro::NixValue::String(#s.to_string())) }
            }
        },
        NixExpr::InterpolatedString(parts) => {
            let quoted_parts = parts.iter().map(|part| match part {
                NixStringPart::Literal(s) => {
                    quote! { ::rust_nix_macro::NixStringPart::Literal(#s.to_string()) }
                }
                NixStringPart::Interpolation(ast) => {
                    let quoted_ast = generate_token_stream(ast); // Recursive call
                    quote! { ::rust_nix_macro::NixStringPart::Interpolation(Box::new(#quoted_ast)) }
                }
            });
            quote! { ::rust_nix_macro::NixExpr::InterpolatedString(vec![#(#quoted_parts),*]) }
        }
        NixExpr::Ref(s) => quote! { ::rust_nix_macro::NixExpr::Ref(#s.to_string()) },
        NixExpr::List(items) => {
            let quoted_items = items.iter().map(generate_token_stream);
            quote! { ::rust_nix_macro::NixExpr::List(vec![#(#quoted_items),*]) }
        }
        NixExpr::AttrSet(bindings) => {
            let quoted_bindings = bindings.iter().map(|(k, v)| {
                let key_str = k;
                let val_ast = generate_token_stream(v);
                quote! { (#key_str.to_string(), #val_ast) }
            });
            quote! {
                ::rust_nix_macro::NixExpr::AttrSet(
                    vec![#(#quoted_bindings),*].into_iter().collect()
                )
            }
        }
    }
}
