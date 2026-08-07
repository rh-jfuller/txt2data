/// Parse tree -> JSON, XML, YAML, SQL INSERT, or S-expression.
use std::fmt::Write;

use crate::ast::Mark;

#[derive(Debug, Clone)]
pub struct ParseTree {
    pub root: TreeNode,
}

#[derive(Debug, Clone)]
pub enum TreeNode {
    Element {
        mark: Mark,
        name: String,
        children: Vec<TreeNode>,
    },
    Text {
        mark: Mark,
        value: String,
    },
    Insertion {
        value: String,
    },
}

#[must_use]
pub fn to_xml(tree: &ParseTree) -> String {
    let mut buf = String::with_capacity(256);
    buf.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    serialize_xml_node(&tree.root, &mut buf, 0, false);
    buf
}

#[must_use]
pub fn to_json(tree: &ParseTree) -> String {
    let mut buf = String::with_capacity(256);
    serialize_json_node(&tree.root, &mut buf, 0);
    buf.push('\n');
    buf
}

#[must_use]
pub fn to_yaml(tree: &ParseTree) -> String {
    let mut buf = String::with_capacity(256);
    buf.push_str("---\n");
    serialize_yaml_node(&tree.root, &mut buf, 0, false);
    buf
}

#[must_use]
pub fn to_sql(tree: &ParseTree) -> String {
    let mut buf = String::with_capacity(256);
    serialize_sql(&tree.root, &mut buf);
    buf
}

#[must_use]
pub fn to_sexp(tree: &ParseTree) -> String {
    let mut buf = String::with_capacity(256);
    serialize_sexp_node(&tree.root, &mut buf, 0);
    buf.push('\n');
    buf
}

fn serialize_sexp_node(node: &TreeNode, buf: &mut String, depth: usize) {
    match node {
        TreeNode::Text { mark, value } => {
            if *mark != Mark::Hidden && !value.is_empty() {
                let indent = "  ".repeat(depth);
                let _ = writeln!(buf, "{indent}{}", sexp_quote(value));
            }
        }
        TreeNode::Insertion { value } => {
            if !value.is_empty() {
                let indent = "  ".repeat(depth);
                let _ = writeln!(buf, "{indent}{}", sexp_quote(value));
            }
        }
        TreeNode::Element {
            mark,
            name,
            children,
        } => match mark {
            Mark::Hidden => {
                // Promote visible named children; drop bare text.
                for child in children {
                    if let TreeNode::Element {
                        mark: Mark::None | Mark::Element | Mark::Attribute,
                        ..
                    } = child
                    {
                        serialize_sexp_node(child, buf, depth);
                    }
                }
            }
            Mark::Attribute => {
                let text = collect_text(node);
                let indent = "  ".repeat(depth);
                let _ = writeln!(buf, "{indent}(@{name} {})", sexp_quote(&text));
            }
            Mark::None | Mark::Element => {
                let indent = "  ".repeat(depth);
                let attrs = collect_attributes(children);
                let non_attr: Vec<&TreeNode> = children
                    .iter()
                    .filter(|c| {
                        has_visible_content(c)
                            && !matches!(
                                c,
                                TreeNode::Element {
                                    mark: Mark::Attribute,
                                    ..
                                }
                            )
                    })
                    .collect();
                let text_only = attrs.is_empty() && non_attr.iter().all(|c| is_text_like(c));
                if text_only {
                    let text = collect_text(node);
                    let _ = writeln!(buf, "{indent}({name} {})", sexp_quote(&text));
                } else {
                    let _ = writeln!(buf, "{indent}({name}");
                    for (aname, aval) in &attrs {
                        let inner = "  ".repeat(depth + 1);
                        let _ = writeln!(buf, "{inner}(@{aname} {})", sexp_quote(aval));
                    }
                    for child in &non_attr {
                        serialize_sexp_node(child, buf, depth + 1);
                    }
                    let _ = writeln!(buf, "{indent})");
                }
            }
        },
    }
}

fn sexp_quote(s: &str) -> String {
    let needs_quote = s.is_empty()
        || s.contains(|c: char| c.is_whitespace() || c == '(' || c == ')' || c == '"' || c == '\\');
    if needs_quote {
        let mut buf = String::with_capacity(s.len() + 2);
        buf.push('"');
        for ch in s.chars() {
            match ch {
                '\\' => buf.push_str("\\\\"),
                '"' => buf.push_str("\\\""),
                '\n' => buf.push_str("\\n"),
                '\t' => buf.push_str("\\t"),
                _ => buf.push(ch),
            }
        }
        buf.push('"');
        buf
    } else {
        s.to_string()
    }
}

fn serialize_xml_node(node: &TreeNode, buf: &mut String, depth: usize, inline: bool) {
    match node {
        TreeNode::Text { mark, value } => {
            if *mark != Mark::Hidden {
                buf.push_str(&xml_escape(value));
            }
        }
        TreeNode::Insertion { value } => {
            buf.push_str(&xml_escape(value));
        }
        TreeNode::Element {
            mark,
            name,
            children,
        } => match mark {
            Mark::Hidden => {
                for child in children {
                    serialize_xml_node(child, buf, depth, inline);
                }
            }
            Mark::Attribute => {}
            Mark::None | Mark::Element => {
                serialize_xml_element(name, children, buf, depth, inline);
            }
        },
    }
}

fn serialize_xml_element(
    name: &str,
    children: &[TreeNode],
    buf: &mut String,
    depth: usize,
    inline: bool,
) {
    let indent = if inline {
        String::new()
    } else {
        "  ".repeat(depth)
    };

    let attrs = collect_attributes(children);
    buf.push_str(&indent);
    buf.push('<');
    buf.push_str(name);
    for (aname, aval) in &attrs {
        buf.push(' ');
        buf.push_str(aname);
        buf.push_str("=\"");
        buf.push_str(&xml_escape_attr(aval));
        buf.push('"');
    }

    let non_attr: Vec<&TreeNode> = children
        .iter()
        .filter(|c| {
            !matches!(
                c,
                TreeNode::Element {
                    mark: Mark::Attribute,
                    ..
                }
            )
        })
        .collect();

    let has_content = non_attr.iter().any(|c| has_visible_content(c));
    if has_content {
        serialize_xml_children(name, &non_attr, buf, depth, inline, &indent);
    } else {
        buf.push_str("/>");
        if !inline {
            buf.push('\n');
        }
    }
}

fn serialize_xml_children(
    name: &str,
    children: &[&TreeNode],
    buf: &mut String,
    depth: usize,
    inline: bool,
    indent: &str,
) {
    let has_text = has_visible_text_content(children);
    let has_elements = children.iter().any(|c| {
        matches!(
            c,
            TreeNode::Element {
                mark: Mark::None | Mark::Element,
                ..
            }
        )
    });
    let mixed = has_text && has_elements;

    if mixed || inline {
        buf.push('>');
        for child in children {
            serialize_xml_node(child, buf, 0, true);
        }
        buf.push_str("</");
        buf.push_str(name);
        buf.push('>');
        if !inline {
            buf.push('\n');
        }
    } else if has_elements {
        buf.push_str(">\n");
        for child in children {
            serialize_xml_node(child, buf, depth + 1, false);
        }
        buf.push_str(indent);
        buf.push_str("</");
        buf.push_str(name);
        buf.push_str(">\n");
    } else {
        buf.push('>');
        for child in children {
            serialize_xml_node(child, buf, depth, true);
        }
        buf.push_str("</");
        buf.push_str(name);
        buf.push_str(">\n");
    }
}

fn serialize_json_node(node: &TreeNode, buf: &mut String, depth: usize) {
    match node {
        TreeNode::Text { mark, value } => {
            if *mark != Mark::Hidden {
                buf.push('"');
                buf.push_str(&json_escape(value));
                buf.push('"');
            }
        }
        TreeNode::Insertion { value } => {
            buf.push('"');
            buf.push_str(&json_escape(value));
            buf.push('"');
        }
        TreeNode::Element {
            mark,
            name: el_name,
            children,
        } => match mark {
            Mark::Hidden => {
                let visible: Vec<&TreeNode> =
                    children.iter().filter(|c| has_visible_content(c)).collect();
                if visible.len() == 1 {
                    serialize_json_node(visible[0], buf, depth);
                } else {
                    for child in &visible {
                        serialize_json_node(child, buf, depth);
                    }
                }
            }
            Mark::Attribute => {
                let text = collect_text(node);
                buf.push('"');
                buf.push_str(&json_escape(&text));
                buf.push('"');
            }
            Mark::None | Mark::Element => {
                let attrs = collect_attributes(children);
                let elements: Vec<&TreeNode> = children
                    .iter()
                    .filter(|c| {
                        !matches!(
                            c,
                            TreeNode::Element {
                                mark: Mark::Attribute,
                                ..
                            }
                        )
                    })
                    .collect();
                serialize_json_element(el_name, &attrs, &elements, buf, depth);
            }
        },
    }
}

fn serialize_json_element(
    el_name: &str,
    attrs: &[(String, String)],
    elements: &[&TreeNode],
    buf: &mut String,
    depth: usize,
) {
    let _ = el_name;
    let indent = "  ".repeat(depth);
    let inner_indent = "  ".repeat(depth + 1);
    let all_text = elements.iter().all(|c| is_text_like(c));

    let _ = writeln!(buf, "{indent}{{");
    let mut first = true;

    for (aname, aval) in attrs {
        if !first {
            buf.push_str(",\n");
        }
        let _ = write!(buf, "{inner_indent}\"@{aname}\": \"{}\"", json_escape(aval));
        first = false;
    }

    if all_text {
        let text = elements.iter().map(|c| collect_text(c)).collect::<String>();
        if !text.is_empty() || attrs.is_empty() {
            if !first {
                buf.push_str(",\n");
            }
            let _ = write!(buf, "{inner_indent}\"#text\": \"{}\"", json_escape(&text));
        }
    } else {
        let visible = collect_visible_elements(elements);
        let groups = group_by_name(&visible);

        for (cname, nodes) in &groups {
            if !first {
                buf.push_str(",\n");
            }
            if nodes.len() == 1 {
                let _ = write!(buf, "{inner_indent}\"{cname}\": ");
                serialize_json_node(nodes[0], buf, depth + 1);
            } else {
                let _ = writeln!(buf, "{inner_indent}\"{cname}\": [");
                for (i, node) in nodes.iter().enumerate() {
                    serialize_json_node(node, buf, depth + 2);
                    if i + 1 < nodes.len() {
                        buf.push(',');
                    }
                    buf.push('\n');
                }
                let _ = write!(buf, "{inner_indent}]");
            }
            first = false;
        }
    }

    let _ = write!(buf, "\n{indent}}}");
}

// --- YAML ---

fn serialize_yaml_node(node: &TreeNode, buf: &mut String, depth: usize, inline: bool) {
    match node {
        TreeNode::Text { mark, value } => {
            if *mark != Mark::Hidden {
                yaml_write_scalar(buf, value, depth, inline);
            }
        }
        TreeNode::Insertion { value } => {
            yaml_write_scalar(buf, value, depth, inline);
        }
        TreeNode::Element {
            mark,
            name,
            children,
        } => match mark {
            Mark::Hidden => {
                for child in children {
                    serialize_yaml_node(child, buf, depth, inline);
                }
            }
            Mark::Attribute => {}
            Mark::None | Mark::Element => {
                serialize_yaml_element(name, children, buf, depth, inline);
            }
        },
    }
}

fn yaml_write_scalar(buf: &mut String, value: &str, depth: usize, inline: bool) {
    if inline {
        buf.push_str(&yaml_escape(value));
    } else {
        let indent = "  ".repeat(depth);
        let _ = writeln!(buf, "{indent}{}", yaml_escape(value));
    }
}

fn serialize_yaml_element(
    name: &str,
    children: &[TreeNode],
    buf: &mut String,
    depth: usize,
    inline: bool,
) {
    let attrs = collect_attributes(children);
    let elements: Vec<&TreeNode> = children
        .iter()
        .filter(|c| {
            !matches!(
                c,
                TreeNode::Element {
                    mark: Mark::Attribute,
                    ..
                }
            )
        })
        .collect();
    let all_text = elements.iter().all(|c| is_text_like(c));
    let indent = "  ".repeat(depth);

    if all_text {
        let text: String = elements.iter().map(|c| collect_text(c)).collect();
        if inline {
            let _ = write!(buf, "{name}: {}", yaml_escape(&text));
        } else {
            let _ = writeln!(buf, "{indent}{name}: {}", yaml_escape(&text));
        }
        for (aname, aval) in &attrs {
            let _ = writeln!(buf, "{indent}{aname}: {}", yaml_escape(aval));
        }
        return;
    }

    let _ = writeln!(buf, "{indent}{name}:");
    for (aname, aval) in &attrs {
        let _ = writeln!(buf, "{indent}  {aname}: {}", yaml_escape(aval));
    }

    let visible = collect_visible_elements(&elements);
    let groups = group_by_name(&visible);

    for (_, nodes) in &groups {
        if nodes.len() == 1 {
            serialize_yaml_node(nodes[0], buf, depth + 1, false);
        } else {
            for n in nodes {
                let inner = "  ".repeat(depth + 1);
                let _ = write!(buf, "{inner}- ");
                serialize_yaml_node(n, buf, depth + 2, true);
                if !buf.ends_with('\n') {
                    buf.push('\n');
                }
            }
        }
    }
}

// --- SQL INSERT ---

fn serialize_sql(node: &TreeNode, buf: &mut String) {
    let TreeNode::Element {
        mark: Mark::None | Mark::Element,
        name: table,
        children,
    } = node
    else {
        return;
    };

    let elements = non_attr_children(children);
    let visible = collect_visible_elements(&elements);
    let groups = group_by_name(&visible);

    // Repeated elements with sub-structure -> multiple rows.
    if groups.len() == 1 && groups[0].1.len() > 1 && !is_text_like(groups[0].1[0]) {
        for row_node in &groups[0].1 {
            emit_insert(table, row_node, buf);
        }
        return;
    }

    // Single row from root's children.
    emit_insert_from_children(table, children, buf);
}

fn emit_insert(table: &str, node: &TreeNode, buf: &mut String) {
    let TreeNode::Element { children, .. } = node else {
        return;
    };
    emit_insert_from_children(table, children, buf);
}

fn emit_insert_from_children(table: &str, children: &[TreeNode], buf: &mut String) {
    let attrs = collect_attributes(children);
    let elements = non_attr_children(children);
    let visible = collect_visible_elements(&elements);

    let mut cols: Vec<String> = Vec::new();
    let mut vals: Vec<String> = Vec::new();

    for (aname, aval) in &attrs {
        cols.push(sql_identifier(aname));
        vals.push(sql_quote(aval));
    }

    for node in &visible {
        if let TreeNode::Element { name, .. } = node {
            let text = collect_text(node);
            cols.push(sql_identifier(name));
            vals.push(sql_quote(&text));
        }
    }

    if cols.is_empty() {
        return;
    }

    let _ = writeln!(
        buf,
        "INSERT INTO {} ({}) VALUES ({});",
        sql_identifier(table),
        cols.join(", "),
        vals.join(", ")
    );
}

fn non_attr_children(children: &[TreeNode]) -> Vec<&TreeNode> {
    children
        .iter()
        .filter(|c| {
            !matches!(
                c,
                TreeNode::Element {
                    mark: Mark::Attribute,
                    ..
                }
            )
        })
        .collect()
}

fn collect_attributes(children: &[TreeNode]) -> Vec<(String, String)> {
    let mut attrs = Vec::new();
    for child in children {
        if let TreeNode::Element {
            mark: Mark::Attribute,
            name,
            ..
        } = child
        {
            let text = collect_text(child);
            attrs.push((name.clone(), text));
        }
        if let TreeNode::Element {
            mark: Mark::Hidden,
            children: hchildren,
            ..
        } = child
        {
            attrs.extend(collect_attributes(hchildren));
        }
    }
    attrs
}

fn collect_text(node: &TreeNode) -> String {
    let mut buf = String::new();
    collect_text_into(node, &mut buf);
    buf
}

fn collect_text_into(node: &TreeNode, buf: &mut String) {
    match node {
        TreeNode::Text { mark, value } => {
            if *mark != Mark::Hidden {
                buf.push_str(value);
            }
        }
        TreeNode::Insertion { value } => {
            buf.push_str(value);
        }
        TreeNode::Element { children, .. } => {
            for child in children {
                collect_text_into(child, buf);
            }
        }
    }
}

/// Check if any node in a list has visible text (not element) content.
fn has_visible_text_content(nodes: &[&TreeNode]) -> bool {
    nodes.iter().any(|n| has_visible_text_in_node(n))
}

fn has_visible_text_in_node(node: &TreeNode) -> bool {
    match node {
        TreeNode::Text { mark, value } => *mark != Mark::Hidden && !value.is_empty(),
        TreeNode::Insertion { value } => !value.is_empty(),
        TreeNode::Element {
            mark: Mark::Hidden,
            children,
            ..
        } => {
            let refs: Vec<&TreeNode> = children.iter().collect();
            has_visible_text_content(&refs)
        }
        TreeNode::Element { .. } => false,
    }
}

/// Check if node has visible content.
fn has_visible_content(node: &TreeNode) -> bool {
    match node {
        TreeNode::Text { mark, value } => *mark != Mark::Hidden && !value.is_empty(),
        TreeNode::Insertion { value } => !value.is_empty(),
        TreeNode::Element { mark, children, .. } => match mark {
            Mark::Attribute => false,
            Mark::Hidden => children.iter().any(has_visible_content),
            Mark::None | Mark::Element => true,
        },
    }
}

/// Flatten hidden elements to collect visible children.
fn collect_visible_elements<'a>(nodes: &[&'a TreeNode]) -> Vec<&'a TreeNode> {
    let mut result = Vec::new();
    for node in nodes {
        match node {
            TreeNode::Element {
                mark: Mark::Hidden,
                children,
                ..
            } => {
                let refs: Vec<&TreeNode> = children.iter().collect();
                result.extend(collect_visible_elements(&refs));
            }
            TreeNode::Element {
                mark: Mark::None | Mark::Element,
                ..
            } => {
                result.push(node);
            }
            _ => {}
        }
    }
    result
}

/// Check if node is purely text (no nested elements).
fn group_by_name<'a>(nodes: &[&'a TreeNode]) -> Vec<(String, Vec<&'a TreeNode>)> {
    let mut groups: Vec<(String, Vec<&TreeNode>)> = Vec::new();
    for node in nodes {
        if let TreeNode::Element { name, .. } = node {
            if let Some(group) = groups.iter_mut().find(|(n, _)| n == name) {
                group.1.push(node);
            } else {
                groups.push((name.clone(), vec![node]));
            }
        }
    }
    groups
}

fn is_text_like(node: &TreeNode) -> bool {
    match node {
        TreeNode::Text { .. } | TreeNode::Insertion { .. } => true,
        TreeNode::Element {
            mark: Mark::Hidden,
            children,
            ..
        } => children.iter().all(is_text_like),
        TreeNode::Element { .. } => false,
    }
}

fn xml_escape(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => buf.push_str("&amp;"),
            '<' => buf.push_str("&lt;"),
            _ => buf.push(ch),
        }
    }
    buf
}

fn xml_escape_attr(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => buf.push_str("&amp;"),
            '<' => buf.push_str("&lt;"),
            '"' => buf.push_str("&quot;"),
            _ => buf.push(ch),
        }
    }
    buf
}

fn json_escape(s: &str) -> String {
    use std::fmt::Write;
    let mut buf = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => buf.push_str("\\\\"),
            '"' => buf.push_str("\\\""),
            '\n' => buf.push_str("\\n"),
            '\r' => buf.push_str("\\r"),
            '\t' => buf.push_str("\\t"),
            c if c.is_control() => {
                let _ = write!(buf, "\\u{:04x}", c as u32);
            }
            _ => buf.push(ch),
        }
    }
    buf
}

fn yaml_escape(s: &str) -> String {
    let needs_quote = s.is_empty()
        || s.contains(':')
        || s.contains('#')
        || s.contains('\n')
        || s.contains('"')
        || s.contains('\'')
        || s.contains('{')
        || s.contains('}')
        || s.contains('[')
        || s.contains(']')
        || s.contains(',')
        || s.contains('&')
        || s.contains('*')
        || s.contains('!')
        || s.contains('|')
        || s.contains('>')
        || s.contains('%')
        || s.contains('@')
        || s.contains('?')
        || s.starts_with(' ')
        || s.starts_with('-')
        || s.ends_with(' ')
        || s.starts_with("---")
        || s.starts_with("...")
        || s == "true"
        || s == "false"
        || s == "null"
        || s == "~";
    if needs_quote {
        format!("\"{}\"", json_escape(s))
    } else {
        s.to_string()
    }
}

fn sql_identifier(s: &str) -> String {
    let escaped = s.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

fn sql_quote(s: &str) -> String {
    let escaped = s.replace('\'', "''");
    format!("'{escaped}'")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tree() -> ParseTree {
        ParseTree {
            root: TreeNode::Element {
                mark: Mark::None,
                name: "greeting".into(),
                children: vec![
                    TreeNode::Text {
                        mark: Mark::None,
                        value: "Hello, ".into(),
                    },
                    TreeNode::Element {
                        mark: Mark::None,
                        name: "name".into(),
                        children: vec![TreeNode::Text {
                            mark: Mark::None,
                            value: "World".into(),
                        }],
                    },
                    TreeNode::Text {
                        mark: Mark::None,
                        value: "!".into(),
                    },
                ],
            },
        }
    }

    #[test]
    fn xml_output() {
        let xml = to_xml(&sample_tree());
        assert!(xml.contains("<greeting>"));
        assert!(xml.contains("<name>World</name>"));
        assert!(xml.contains("</greeting>"));
    }

    #[test]
    fn json_output() {
        let json = to_json(&sample_tree());
        assert!(json.contains("\"name\""));
        assert!(json.contains("World"));
    }

    #[test]
    fn yaml_output() {
        let yaml = to_yaml(&sample_tree());
        assert!(yaml.starts_with("---\n"), "Missing YAML header: {yaml}");
        assert!(yaml.contains("name:"), "Missing name key: {yaml}");
        assert!(yaml.contains("World"), "Missing World value: {yaml}");
    }

    #[test]
    fn sql_output() {
        let sql = to_sql(&sample_tree());
        assert!(sql.contains("INSERT INTO"), "Missing INSERT: {sql}");
        assert!(sql.contains("greeting"), "Missing table name: {sql}");
        assert!(sql.contains("'World'"), "Missing value: {sql}");
    }
}
