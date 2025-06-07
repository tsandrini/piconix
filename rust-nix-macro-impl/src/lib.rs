use pest::Parser;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

mod internal_parser {
    use pest_derive::Parser;
    #[derive(Parser)]
    #[grammar = "grammar.pest"]
    pub struct NixParser; // This can be pub inside the private module.
}

use internal_parser::{NixParser, Rule};

enum Ast {
    Int(i64),
    Var(String),
    List(Vec<Ast>),
    AttrSet(Vec<(String, Ast)>),
}

fn build_ast_from_pair(pair: pest::iterators::Pair<Rule>) -> Ast {
    match pair.as_rule() {
        Rule::expr => build_ast_from_pair(pair.into_inner().next().unwrap()),
        Rule::integer => Ast::Int(pair.as_str().parse().unwrap_or(0)),
        Rule::identifier => Ast::Var(pair.as_str().to_string()),
        Rule::list => {
            let items = pair.into_inner().map(build_ast_from_pair).collect();
            Ast::List(items)
        }
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
        _ => unreachable!("Should not receive rule: {:?}", pair.as_rule()),
    }
}

fn generate_code_from_ast(ast: Ast) -> TokenStream2 {
    match ast {
        Ast::Int(i) => quote! { ::rust_nix_macro::Expr::Int(#i) },
        Ast::Var(s) => quote! { ::rust_nix_macro::Expr::Var(#s.to_string()) },
        Ast::List(items) => {
            let quoted_items = items.into_iter().map(generate_code_from_ast);
            quote! {
                ::rust_nix_macro::Expr::List(vec![#(#quoted_items),*])
            }
        }
        Ast::AttrSet(bindings) => {
            let quoted_bindings = bindings.into_iter().map(|(k, v)| {
                let key_str = k;
                let val_ast = generate_code_from_ast(v);
                quote! { (#key_str.to_string(), #val_ast) }
            });
            quote! {
                ::rust_nix_macro::Expr::AttrSet(vec![#(#quoted_bindings),*])
            }
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

    let nix_file_pair = parsed.into_iter().next().unwrap();
    let expr_pair = nix_file_pair.into_inner().next().unwrap();
    let ast = build_ast_from_pair(expr_pair);

    generate_code_from_ast(ast).into()
}
