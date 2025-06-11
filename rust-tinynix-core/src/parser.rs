use crate::{NixBinaryOp, NixExpr, NixStringPart, NixUnaryOp, NixValue};
use indexmap::IndexMap;
use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::{Assoc, Op, PrattParser};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct NixParser;

static PRATT_PARSER: OnceLock<PrattParser<Rule>> = OnceLock::new();

fn get_pratt_parser() -> &'static PrattParser<Rule> {
    PRATT_PARSER.get_or_init(|| {
        use Assoc::*;
        use Rule::*;

        PrattParser::new().op(Op::infix(infix_op, Left))
    })
}

fn parse_op_expr(pairs: Pairs<Rule>, root: &Path) -> NixExpr {
    get_pratt_parser()
        .map_primary(|primary| build_nix_expr_from_pair(primary, root))
        .map_infix(|lhs, op, rhs| {
            let op_pair = op.into_inner().next().unwrap();
            let op = match op_pair.as_rule() {
                Rule::add => NixBinaryOp::Add,
                Rule::sub => NixBinaryOp::Sub,
                _ => unreachable!("Encountered non-infix operator in infix position"),
            };
            NixExpr::BinaryOp {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            }
        })
        .parse(pairs)
}

fn build_nix_expr_from_pair(pair: Pair<Rule>, root: &Path) -> NixExpr {
    match pair.as_rule() {
        // --- Structural Rules that simply wrap another rule ---
        Rule::nix_expression | Rule::atomic_expr | Rule::literal | Rule::path_types => {
            build_nix_expr_from_pair(pair.into_inner().next().unwrap(), root)
        }

        // --- Logic Rules ---
        Rule::op_expr => parse_op_expr(pair.into_inner(), root),
        Rule::let_in_expr => {
            let mut pairs = pair.into_inner();
            let body_pair = pairs
                .next_back()
                .expect("let-in expression must have a body");
            let body = build_nix_expr_from_pair(body_pair, root);
            let bindings = build_bindings_from_pairs(pairs, root);
            NixExpr::LetIn {
                bindings,
                body: Box::new(body),
            }
        }
        Rule::with_expr => {
            let mut pairs = pair.into_inner();
            let environment_pair = pairs
                .next()
                .expect("with expression must have an environment");
            let body_pair = pairs.next().expect("with expression must have a body");
            let environment = build_nix_expr_from_pair(environment_pair, root);
            let body = build_nix_expr_from_pair(body_pair, root);
            NixExpr::With {
                environment: Box::new(environment),
                body: Box::new(body),
            }
        }
        Rule::term => {
            let mut pairs = pair.into_inner();
            let atomic_pair = pairs.next_back().unwrap();
            let mut current_expr = build_nix_expr_from_pair(atomic_pair, root);

            // The remaining pairs are prefix_ops. We iterate in reverse to apply them
            // from the inside out (closest to the atomic_expr first).
            for op_pair in pairs.rev() {
                let inner_op = op_pair.into_inner().next().unwrap();
                let op = match inner_op.as_rule() {
                    Rule::arith_neg => NixUnaryOp::Neg,
                    Rule::logic_neg => NixUnaryOp::Not,
                    _ => unreachable!("Encountered a non-prefix operator in a term"),
                };
                current_expr = NixExpr::UnaryOp {
                    op,
                    expr: Box::new(current_expr),
                };
            }
            current_expr
        }

        // --- Concrete Atomic Rules ---
        Rule::integer => NixExpr::Value(NixValue::Int(pair.as_str().parse().unwrap())),
        Rule::float => NixExpr::Value(NixValue::Float(pair.as_str().parse().unwrap())),
        Rule::boolean => NixExpr::Value(NixValue::Bool(pair.as_str() == "true")),
        Rule::null => NixExpr::Value(NixValue::Null),
        Rule::identifier => NixExpr::Ref(pair.as_str().to_string()),
        Rule::list => NixExpr::List(
            pair.into_inner()
                .map(|p| build_nix_expr_from_pair(p, root))
                .collect(),
        ),
        Rule::attrset => {
            let mut inner = pair.into_inner();
            let mut recursive = false;
            if let Some(token) = inner.peek() {
                if token.as_rule() == Rule::rec {
                    recursive = true;
                    inner.next();
                }
            }
            let bindings = build_bindings_from_pairs(inner, root);
            NixExpr::AttrSet {
                recursive,
                bindings,
            }
        }
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
            NixExpr::Value(NixValue::Path(final_path))
        }
        Rule::search_path => {
            let content = pair.into_inner().next().unwrap().as_str();
            NixExpr::SearchPath(content.to_string())
        }

        // --- This catch-all prevents panics for truly unhandled rules.
        _ => unreachable!(
            "build_nix_expr_from_pair: Unhandled rule: `{:?}` with content `{}`",
            pair.as_rule(),
            pair.as_str()
        ),
    }
}

// build_bindings_from_pairs, insert_at_path, and parse are unchanged
fn build_bindings_from_pairs(pairs: Pairs<Rule>, root: &Path) -> IndexMap<String, NixExpr> {
    let mut bindings: IndexMap<String, NixExpr> = IndexMap::new();
    for attr_binding_pair in pairs {
        let binding_rule_pair = attr_binding_pair.into_inner().next().unwrap();
        match binding_rule_pair.as_rule() {
            Rule::binding => {
                let mut inner_rules = binding_rule_pair.into_inner();
                let path_pair = inner_rules.next().unwrap();
                let path_str = path_pair.as_str().to_string();
                let path: Vec<&str> = path_str.split('.').collect();
                let expr = build_nix_expr_from_pair(inner_rules.next().unwrap(), root);
                insert_at_path(&mut bindings, &path, expr);
            }
            Rule::inherit_binding => {
                let mut inner_inherit = binding_rule_pair.into_inner();
                let mut scope_ident: Option<String> = None;
                if let Some(token) = inner_inherit.peek() {
                    if token.as_rule() == Rule::identifier {
                        scope_ident = Some(token.as_str().to_string());
                        inner_inherit.next();
                    }
                }
                for ident_to_inherit_pair in inner_inherit {
                    let ident_name = ident_to_inherit_pair.as_str().to_string();
                    let value_expr = match &scope_ident {
                        Some(scope) => NixExpr::Ref(format!("{}.{}", scope, ident_name)),
                        None => NixExpr::Ref(ident_name.clone()),
                    };
                    bindings.insert(ident_name, value_expr);
                }
            }
            _ => unreachable!(
                "Unexpected rule inside attr_binding: {:?}",
                binding_rule_pair.as_rule()
            ),
        }
    }
    bindings
}

fn insert_at_path(attrset: &mut IndexMap<String, NixExpr>, path: &[&str], value: NixExpr) {
    let key = path[0].to_string();
    if path.len() == 1 {
        attrset.insert(key, value);
        return;
    }
    let entry = attrset.entry(key).or_insert_with(|| NixExpr::AttrSet {
        recursive: false,
        bindings: IndexMap::new(),
    });
    if let NixExpr::AttrSet { bindings, .. } = entry {
        insert_at_path(bindings, &path[1..], value);
    } else {
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
