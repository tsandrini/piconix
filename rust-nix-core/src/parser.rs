use crate::{NixExpr, NixStringPart, NixValue};
use indexmap::IndexMap;
use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct NixParser;

fn build_nix_expr_from_pair(pair: pest::iterators::Pair<Rule>) -> NixExpr {
    match pair.as_rule() {
        Rule::nix_expression | Rule::term => {
            build_nix_expr_from_pair(pair.into_inner().next().unwrap())
        }

        // Primitive types
        Rule::integer => NixExpr::Value(NixValue::Int(pair.as_str().parse().unwrap())),
        Rule::float => NixExpr::Value(NixValue::Float(pair.as_str().parse().unwrap())),
        Rule::boolean => NixExpr::Value(NixValue::Bool(pair.as_str() == "true")),

        Rule::string => {
            let mut parts: Vec<NixStringPart> = Vec::new();
            for string_content_pair in pair.into_inner() {
                let part = string_content_pair.into_inner().next().unwrap();
                match part.as_rule() {
                    Rule::string_literal_part
                    | Rule::dollar_literal
                    | Rule::single_quote_literal => {
                        parts.push(NixStringPart::Literal(part.as_str().to_string()));
                    }
                    Rule::escaped_quote => parts.push(NixStringPart::Literal("\"".to_string())),
                    Rule::escaped_interpolation => {
                        parts.push(NixStringPart::Literal("${".to_string()))
                    }
                    Rule::interpolation => {
                        let inner_expr = part.into_inner().next().unwrap();
                        parts.push(NixStringPart::Interpolation(Box::new(
                            build_nix_expr_from_pair(inner_expr),
                        )));
                    }
                    _ => unreachable!("Unexpected string part: {:?}", part.as_rule()),
                }
            }

            if parts.len() == 1 {
                if let NixStringPart::Literal(s) = &parts[0] {
                    return NixExpr::Value(NixValue::String(s.clone()));
                }
            }
            NixExpr::InterpolatedString(parts)
        }

        // Compound types
        Rule::identifier => NixExpr::Ref(pair.as_str().to_string()),
        Rule::list => NixExpr::List(pair.into_inner().map(build_nix_expr_from_pair).collect()),
        Rule::attrset => {
            let bindings: IndexMap<String, NixExpr> = pair
                .into_inner()
                .map(|binding_pair| {
                    let mut inner_rules = binding_pair.into_inner();
                    let ident = inner_rules.next().unwrap().as_str().to_string();
                    let expr = build_nix_expr_from_pair(inner_rules.next().unwrap());
                    (ident, expr)
                })
                .collect();
            NixExpr::AttrSet(bindings)
        }
        _ => unreachable!(
            "Unexpected grammar rule: {:?} with content '{}'",
            pair.as_rule(),
            pair.as_str()
        ),
    }
}

pub fn parse(input: &str) -> Result<NixExpr, pest::error::Error<Rule>> {
    let expr_pair = NixParser::parse(Rule::source, input)?
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();
    Ok(build_nix_expr_from_pair(expr_pair))
}
