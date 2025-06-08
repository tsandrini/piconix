use crate::{NixExpr, NixStringPart, NixValue};
use indexmap::IndexMap;
use pest::Parser;
use std::path::{Path, PathBuf};

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct NixParser;

fn build_nix_expr_from_pair(pair: pest::iterators::Pair<Rule>, root: &Path) -> NixExpr {
    match pair.as_rule() {
        Rule::nix_expression => build_nix_expr_from_pair(pair.into_inner().next().unwrap(), root),

        // Primitive types
        Rule::integer => NixExpr::Value(NixValue::Int(pair.as_str().parse().unwrap())),
        Rule::float => NixExpr::Value(NixValue::Float(pair.as_str().parse().unwrap())),
        Rule::boolean => NixExpr::Value(NixValue::Bool(pair.as_str() == "true")),
        Rule::null => NixExpr::Value(NixValue::Null),

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
                            build_nix_expr_from_pair(inner_expr, root),
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
        Rule::path => {
            let path_str = pair.as_str();
            let mut path_buf;
            if let Some(stripped) = path_str.strip_prefix("~/") {
                path_buf = home::home_dir().expect("Could not find home directory");
                path_buf.push(stripped);
            } else {
                path_buf = PathBuf::from(path_str);
            }

            let final_path = if path_buf.is_absolute() {
                path_buf
            } else {
                root.join(path_buf)
            };
            // TODO: canonicalize the path?
            NixExpr::Value(NixValue::Path(final_path))
        }
        Rule::search_path => {
            let content = pair.into_inner().next().unwrap().as_str();
            NixExpr::SearchPath(content.to_string())
        }
        // Compound types
        Rule::identifier => NixExpr::Ref(pair.as_str().to_string()),
        Rule::list => NixExpr::List(
            pair.into_inner()
                .map(|p| build_nix_expr_from_pair(p, root))
                .collect(),
        ),
        Rule::attrset => {
            let mut bindings: IndexMap<String, NixExpr> = IndexMap::new();
            for binding_pair in pair.into_inner() {
                // Iterate over `binding` rules
                let mut inner_rules = binding_pair.into_inner();
                let path_pair = inner_rules.next().unwrap();
                let path_str = path_pair.as_str().to_string();
                let path: Vec<&str> = path_str.split('.').collect();

                let expr = build_nix_expr_from_pair(inner_rules.next().unwrap(), root);

                insert_at_path(&mut bindings, &path, expr);
            }
            NixExpr::AttrSet(bindings)
        }
        _ => unreachable!(
            "Unexpected grammar rule: {:?} with content '{}'",
            pair.as_rule(),
            pair.as_str()
        ),
    }
}

fn insert_at_path(attrset: &mut IndexMap<String, NixExpr>, path: &[&str], value: NixExpr) {
    let key = path[0].to_string();
    if path.len() == 1 {
        attrset.insert(key, value);
        return;
    }

    let entry = attrset
        .entry(key)
        .or_insert_with(|| NixExpr::AttrSet(IndexMap::new()));

    if let NixExpr::AttrSet(next_attrset) = entry {
        insert_at_path(next_attrset, &path[1..], value);
    } else {
        // Handle error: trying to define `a.b` when `a` is already something else.
        // This should ideally be a pest::error::Error.
        // For simplicity, we can panic or return a Result.
        panic!("Attribute path conflicts with an existing value.");
    }
}

pub fn parse(input: &str, root: &Path) -> Result<NixExpr, pest::error::Error<Rule>> {
    let expr_pair = NixParser::parse(Rule::source, input)?
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();
    Ok(build_nix_expr_from_pair(expr_pair, root))
}
