mod ast;
mod charclass;
mod earley;
mod grammar;
mod serialize;

pub use ast::{Grammar, Mark, Rule};
pub use earley::Parser;
pub use grammar::parse_grammar;
pub use serialize::{ParseTree, TreeNode, to_json, to_sql, to_xml, to_yaml};

/// Grammar + input -> JSON string.
///
/// # Errors
///
/// Returns error if the grammar is invalid or input doesn't match.
pub fn parse_to_json(grammar_src: &str, input: &str) -> Result<String, String> {
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
/// Returns error if the grammar is invalid or input doesn't match.
pub fn parse_to_xml(grammar_src: &str, input: &str) -> Result<String, String> {
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
/// Returns error if the grammar is invalid or input doesn't match.
pub fn parse_to_yaml(grammar_src: &str, input: &str) -> Result<String, String> {
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
/// Returns error if the grammar is invalid or input doesn't match.
pub fn parse_to_sql(grammar_src: &str, input: &str) -> Result<String, String> {
    let grammar = parse_grammar(grammar_src).map_err(|e| format!("Grammar error: {e}"))?;
    let parser = Parser::new(&grammar);
    let tree = parser
        .parse(input)
        .map_err(|e| format!("Parse error: {e}"))?;
    Ok(to_sql(&tree))
}

#[cfg(feature = "wasm")]
mod wasm_api;
