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

enum StringPart {
    Literal(String),
    Interpolation(Box<Ast>),
}

enum Ast {
    Int(i64),
    SimpleString(String),
    InterpolatedString(Vec<StringPart>),
    Ref(String),
    List(Vec<Ast>),
    AttrSet(Vec<(String, Ast)>),
}

fn build_ast_from_pair(pair: pest::iterators::Pair<Rule>) -> Ast {
    match pair.as_rule() {
        Rule::expr => build_ast_from_pair(pair.into_inner().next().unwrap()),
        Rule::integer => Ast::Int(pair.as_str().parse().unwrap_or(0)),
        Rule::string => {
            let parts: Vec<StringPart> = pair
                .into_inner()
                .map(|string_content_pair| {
                    let part = string_content_pair.into_inner().next().unwrap();
                    match part.as_rule() {
                        // Literal parts are passed through directly
                        Rule::string_literal_part
                        | Rule::dollar_literal
                        | Rule::single_quote_literal => {
                            StringPart::Literal(part.as_str().to_string())
                        }
                        // Escape sequences are converted back to their literal string value
                        Rule::escaped_quote => StringPart::Literal("\"".to_string()),
                        Rule::escaped_interpolation => StringPart::Literal("${".to_string()),

                        // Interpolations are parsed recursively
                        Rule::interpolation => {
                            let inner_expr = part.into_inner().next().unwrap();
                            StringPart::Interpolation(Box::new(build_ast_from_pair(inner_expr)))
                        }
                        _ => unreachable!("Unexpected string part: {:?}", part.as_rule()),
                    }
                })
                .collect();

            if parts.len() == 1 {
                if let StringPart::Literal(s) = &parts[0] {
                    return Ast::SimpleString(s.clone());
                }
            }
            Ast::InterpolatedString(parts)
        }
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
        _ => unreachable!("Unexpected rule: {:?}", pair.as_rule()),
    }
}

fn generate_code_from_string_part(part: StringPart) -> TokenStream2 {
    match part {
        StringPart::Literal(s) => {
            quote! { ::rust_nix_macro::StringPart::Literal(#s.to_string()) }
        }
        StringPart::Interpolation(ast) => {
            let quoted_ast = generate_code_from_ast(*ast);
            quote! { ::rust_nix_macro::StringPart::Interpolation(Box::new(#quoted_ast)) }
        }
    }
}

fn generate_code_from_ast(ast: Ast) -> TokenStream2 {
    match ast {
        Ast::Int(i) => quote! { ::rust_nix_macro::Expr::Int(#i) },
        Ast::SimpleString(s) => quote! { ::rust_nix_macro::Expr::SimpleString(#s.to_string()) },
        Ast::InterpolatedString(parts) => {
            let quoted_parts = parts.into_iter().map(generate_code_from_string_part);
            quote! { ::rust_nix_macro::Expr::InterpolatedString(vec![#(#quoted_parts),*]) }
        }
        Ast::Ref(s) => quote! { ::rust_nix_macro::Expr::Ref(#s.to_string()) },
        Ast::List(items) => {
            let quoted_items = items.into_iter().map(generate_code_from_ast);
            quote! { ::rust_nix_macro::Expr::List(vec![#(#quoted_items),*]) }
        }
        Ast::AttrSet(bindings) => {
            let quoted_bindings = bindings.into_iter().map(|(k, v)| {
                let key_str = k;
                let val_ast = generate_code_from_ast(v);
                quote! { (#key_str.to_string(), #val_ast) }
            });
            quote! { ::rust_nix_macro::Expr::AttrSet(vec![#(#quoted_bindings),*]) }
        }
    }
}

#[proc_macro]
pub fn nix(input: TokenStream) -> TokenStream {
    let code_as_string = input.to_string();
    let parsed = match NixParser::parse(Rule::nix_file, &code_as_string) {
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
