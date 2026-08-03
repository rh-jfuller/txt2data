mod ast;
mod charclass;
mod earley;
mod grammar;
mod serialize;

pub use ast::{Grammar, Mark, Rule};
pub use earley::Parser;
pub use grammar::parse_grammar;
pub use serialize::{ParseTree, TreeNode, to_json, to_sql, to_xml, to_yaml};

const DEFAULT_MAX_GRAMMAR: usize = 1_024 * 1_024;
const DEFAULT_MAX_INPUT: usize = 10 * 1_024 * 1_024;

fn env_limit(var: &str, default: usize) -> usize {
    std::env::var(var)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn check_limits(grammar_src: &str, input: &str) -> Result<(), String> {
    let max_grammar = env_limit("TXT2DATA_MAX_GRAMMAR", DEFAULT_MAX_GRAMMAR);
    if grammar_src.len() > max_grammar {
        return Err(format!(
            "Grammar too large ({} bytes, max {max_grammar})",
            grammar_src.len()
        ));
    }
    let max_input = env_limit("TXT2DATA_MAX_INPUT", DEFAULT_MAX_INPUT);
    if input.len() > max_input {
        return Err(format!(
            "Input too large ({} bytes, max {max_input})",
            input.len()
        ));
    }
    Ok(())
}

/// Grammar + input -> JSON string.
///
/// # Errors
///
/// Returns error if grammar is invalid, input doesn't match,
/// or either exceeds size limits.
pub fn parse_to_json(grammar_src: &str, input: &str) -> Result<String, String> {
    check_limits(grammar_src, input)?;
    let grammar = parse_grammar(grammar_src).map_err(|e| format!("Grammar error: {e}"))?;
    let parser = Parser::new(&grammar);
    let tree = parser
        .parse(input)
        .map_err(|e| format!("Parse error: {e}"))?;
    Ok(to_json(&tree))
}

/// Grammar + input -> XML string.
///
/// # Errors
///
/// Returns error if grammar is invalid, input doesn't match,
/// or either exceeds size limits.
pub fn parse_to_xml(grammar_src: &str, input: &str) -> Result<String, String> {
    check_limits(grammar_src, input)?;
    let grammar = parse_grammar(grammar_src).map_err(|e| format!("Grammar error: {e}"))?;
    let parser = Parser::new(&grammar);
    let tree = parser
        .parse(input)
        .map_err(|e| format!("Parse error: {e}"))?;
    Ok(to_xml(&tree))
}

/// Grammar + input -> YAML string.
///
/// # Errors
///
/// Returns error if grammar is invalid, input doesn't match,
/// or either exceeds size limits.
pub fn parse_to_yaml(grammar_src: &str, input: &str) -> Result<String, String> {
    check_limits(grammar_src, input)?;
    let grammar = parse_grammar(grammar_src).map_err(|e| format!("Grammar error: {e}"))?;
    let parser = Parser::new(&grammar);
    let tree = parser
        .parse(input)
        .map_err(|e| format!("Parse error: {e}"))?;
    Ok(to_yaml(&tree))
}

/// Grammar + input -> SQL INSERT statements.
///
/// # Errors
///
/// Returns error if grammar is invalid, input doesn't match,
/// or either exceeds size limits.
pub fn parse_to_sql(grammar_src: &str, input: &str) -> Result<String, String> {
    check_limits(grammar_src, input)?;
    let grammar = parse_grammar(grammar_src).map_err(|e| format!("Grammar error: {e}"))?;
    let parser = Parser::new(&grammar);
    let tree = parser
        .parse(input)
        .map_err(|e| format!("Parse error: {e}"))?;
    Ok(to_sql(&tree))
}

#[cfg(feature = "wasm")]
mod wasm_api;
