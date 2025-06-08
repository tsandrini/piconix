use pest::Parser;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

mod internal_parser {
    use pest_derive::Parser;
    #[derive(Parser)]
    #[grammar = "grammar.pest"]
    pub struct NixParser;
}

use internal_parser::{NixParser, Rule};

enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

enum StringPart {
    Literal(String),
    Interpolation(Box<Ast>),
}

enum Ast {
    Value(Value),
    InterpolatedString(Vec<StringPart>),
    Ref(String),
    List(Vec<Ast>),
    AttrSet(Vec<(String, Ast)>),
}

fn build_ast_from_pair(pair: pest::iterators::Pair<Rule>) -> Ast {
    match pair.as_rule() {
        Rule::nix_expression | Rule::term => build_ast_from_pair(pair.into_inner().next().unwrap()),

        // Primitive types
        Rule::integer => Ast::Value(Value::Int(pair.as_str().parse().unwrap())),
        Rule::float => Ast::Value(Value::Float(pair.as_str().parse().unwrap())),
        Rule::boolean => Ast::Value(Value::Bool(pair.as_str() == "true")),

        Rule::string => {
            let mut parts: Vec<StringPart> = Vec::new();
            // A "string" pair contains "string_content" pairs. We must iterate
            // through those and then look at the rule *inside* each one.
            for string_content_pair in pair.into_inner() {
                let part = string_content_pair.into_inner().next().unwrap();
                match part.as_rule() {
                    Rule::string_literal_part
                    | Rule::dollar_literal
                    | Rule::single_quote_literal => {
                        parts.push(StringPart::Literal(part.as_str().to_string()));
                    }
                    Rule::escaped_quote => parts.push(StringPart::Literal("\"".to_string())),
                    Rule::escaped_interpolation => {
                        parts.push(StringPart::Literal("${".to_string()))
                    }
                    Rule::interpolation => {
                        let inner_expr = part.into_inner().next().unwrap();
                        parts.push(StringPart::Interpolation(Box::new(build_ast_from_pair(
                            inner_expr,
                        ))));
                    }
                    _ => unreachable!("Unexpected string part: {:?}", part.as_rule()),
                }
            }

            // If a string has no interpolations, it becomes a simple Value::String.
            if parts.len() == 1 {
                if let StringPart::Literal(s) = &parts[0] {
                    return Ast::Value(Value::String(s.clone()));
                }
            }
            Ast::InterpolatedString(parts)
        }

        // Compound types
        Rule::identifier => Ast::Ref(pair.as_str().to_string()),
        Rule::list => Ast::List(pair.into_inner().map(build_ast_from_pair).collect()),
        Rule::attrset => {
            let bindings = pair
                .into_inner()
                .map(|binding_pair| {
                    let mut inner_rules = binding_pair.into_inner();
                    let ident = inner_rules.next().unwrap().as_str().to_string();
                    let expr = build_ast_from_pair(inner_rules.next().unwrap());
                    (ident, expr)
                })
                .collect();
            Ast::AttrSet(bindings)
        }
        _ => unreachable!(
            "Unexpected grammar rule: {:?} with content '{}'",
            pair.as_rule(),
            pair.as_str()
        ),
    }
}

fn generate_code_from_ast(ast: Ast) -> TokenStream2 {
    match ast {
        Ast::Value(value) => match value {
            Value::Int(i) => {
                quote! { ::rust_nix_macro::NixExpr::Value(::rust_nix_macro::NixValue::Int(#i)) }
            }
            Value::Float(f) => {
                quote! { ::rust_nix_macro::NixExpr::Value(::rust_nix_macro::NixValue::Float(#f)) }
            }
            Value::Bool(b) => {
                quote! { ::rust_nix_macro::NixExpr::Value(::rust_nix_macro::NixValue::Bool(#b)) }
            }
            Value::String(s) => {
                quote! { ::rust_nix_macro::NixExpr::Value(::rust_nix_macro::NixValue::String(#s.to_string())) }
            }
        },
        Ast::InterpolatedString(parts) => {
            let quoted_parts = parts.into_iter().map(|part| match part {
                StringPart::Literal(s) => {
                    quote! { ::rust_nix_macro::NixStringPart::Literal(#s.to_string()) }
                }
                StringPart::Interpolation(ast) => {
                    let quoted_ast = generate_code_from_ast(*ast);
                    quote! { ::rust_nix_macro::NixStringPart::Interpolation(Box::new(#quoted_ast)) }
                }
            });
            quote! { ::rust_nix_macro::NixExpr::InterpolatedString(vec![#(#quoted_parts),*]) }
        }
        Ast::Ref(s) => quote! { ::rust_nix_macro::NixExpr::Ref(#s.to_string()) },
        Ast::List(items) => {
            let quoted_items = items.into_iter().map(generate_code_from_ast);
            quote! { ::rust_nix_macro::NixExpr::List(vec![#(#quoted_items),*]) }
        }
        Ast::AttrSet(bindings) => {
            let quoted_bindings = bindings.into_iter().map(|(k, v)| {
                let key_str = k;
                let val_ast = generate_code_from_ast(v);
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

#[proc_macro]
pub fn nix(input: TokenStream) -> TokenStream {
    let code_as_string = input.to_string();
    let parsed = match NixParser::parse(Rule::source, &code_as_string) {
        Ok(pairs) => pairs,
        Err(e) => {
            let msg = format!("Nix parsing failed:\n{}", e);
            return syn::Error::new(proc_macro2::Span::call_site(), msg)
                .to_compile_error()
                .into();
        }
    };
    let expr_pair = parsed
        .into_iter()
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();
    let ast = build_ast_from_pair(expr_pair);
    generate_code_from_ast(ast).into()
}
