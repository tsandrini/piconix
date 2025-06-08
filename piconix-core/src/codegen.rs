use crate::{NixExpr, NixStringPart, NixValue};
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_token_stream(ast: &NixExpr) -> TokenStream {
    match ast {
        NixExpr::Value(value) => match value {
            NixValue::Int(i) => {
                quote! { ::piconix::NixExpr::Value(::piconix::NixValue::Int(#i)) }
            }
            NixValue::Float(f) => {
                quote! { ::piconix::NixExpr::Value(::piconix::NixValue::Float(#f)) }
            }
            NixValue::Bool(b) => {
                quote! { ::piconix::NixExpr::Value(::piconix::NixValue::Bool(#b)) }
            }
            NixValue::String(s) => {
                quote! { ::piconix::NixExpr::Value(::piconix::NixValue::String(#s.to_string())) }
            }
        },
        NixExpr::InterpolatedString(parts) => {
            let quoted_parts = parts.iter().map(|part| match part {
                NixStringPart::Literal(s) => {
                    quote! { ::piconix::NixStringPart::Literal(#s.to_string()) }
                }
                NixStringPart::Interpolation(ast) => {
                    let quoted_ast = generate_token_stream(ast); // Recursive call
                    quote! { ::piconix::NixStringPart::Interpolation(Box::new(#quoted_ast)) }
                }
            });
            quote! { ::piconix::NixExpr::InterpolatedString(vec![#(#quoted_parts),*]) }
        }
        NixExpr::Ref(s) => quote! { ::piconix::NixExpr::Ref(#s.to_string()) },
        NixExpr::List(items) => {
            let quoted_items = items.iter().map(generate_token_stream);
            quote! { ::piconix::NixExpr::List(vec![#(#quoted_items),*]) }
        }
        NixExpr::AttrSet(bindings) => {
            let quoted_bindings = bindings.iter().map(|(k, v)| {
                let key_str = k;
                let val_ast = generate_token_stream(v);
                quote! { (#key_str.to_string(), #val_ast) }
            });
            quote! {
                ::piconix::NixExpr::AttrSet(
                    vec![#(#quoted_bindings),*].into_iter().collect()
                )
            }
        }
    }
}
