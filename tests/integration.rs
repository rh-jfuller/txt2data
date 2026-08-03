#![expect(clippy::unwrap_used)]

use txt2data::{parse_grammar, parse_to_json, parse_to_sql, parse_to_xml, parse_to_yaml};

#[test]
fn greeting_to_xml() {
    let grammar = r#"
        greeting: "Hello, ", name, "!".
        name: letter+.
        -letter: ["a"-"z"; "A"-"Z"].
    "#;
    let xml = parse_to_xml(grammar, "Hello, World!").unwrap();
    assert!(
        xml.contains("<greeting>"),
        "Missing greeting element: {xml}"
    );
    assert!(
        xml.contains("<name>World</name>"),
        "Missing name element: {xml}"
    );
}

#[test]
fn greeting_to_json() {
    let grammar = r#"
        greeting: "Hello, ", name, "!".
        name: letter+.
        -letter: ["a"-"z"; "A"-"Z"].
    "#;
    let json = parse_to_json(grammar, "Hello, World!").unwrap();
    assert!(json.contains("\"name\""), "Missing name key: {json}");
}

#[test]
fn attribute_mark() {
    let grammar = r#"
        item: @name.
        @name: ["a"-"z"]+.
    "#;
    let xml = parse_to_xml(grammar, "hello").unwrap();
    assert!(xml.contains("name=\"hello\""), "Missing attribute: {xml}");
}

#[test]
fn hidden_terminals() {
    let grammar = r#"
        pair: key, -"=", value.
        key: letter+.
        value: letter+.
        -letter: ["a"-"z"].
    "#;
    let xml = parse_to_xml(grammar, "foo=bar").unwrap();
    assert!(xml.contains("<key>foo</key>"), "Missing key: {xml}");
    assert!(xml.contains("<value>bar</value>"), "Missing value: {xml}");
    assert!(!xml.contains("=</"), "Equals should be hidden: {xml}");
}

#[test]
fn character_range() {
    let grammar = r#"
        digits: digit+.
        -digit: ["0"-"9"].
    "#;
    let xml = parse_to_xml(grammar, "42").unwrap();
    assert!(
        xml.contains("<digits>42</digits>"),
        "Wrong digits output: {xml}"
    );
}

#[test]
fn optional_term() {
    let grammar = r#"
        maybe: "a", letter?.
        letter: ["b"-"z"].
    "#;
    let r1 = parse_to_xml(grammar, "ab");
    assert!(r1.is_ok(), "Failed with 'ab': {r1:?}");

    let r2 = parse_to_xml(grammar, "a");
    assert!(r2.is_ok(), "Failed with 'a': {r2:?}");
}

#[test]
fn exclusion_charset() {
    let grammar = r#"
        word: char+.
        -char: ~[" "].
    "#;
    let xml = parse_to_xml(grammar, "hello").unwrap();
    assert!(xml.contains("<word>hello</word>"), "Wrong output: {xml}");
}

#[test]
fn parse_error_reported() {
    let grammar = r#"
        yes: "yes".
    "#;
    let result = parse_to_xml(grammar, "no");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Parse error"), "Bad error message: {err}");
}

#[test]
fn grammar_error_undefined_rule() {
    let grammar = r"
        root: missing.
    ";
    let result = parse_grammar(grammar);
    assert!(result.is_err());
}

#[test]
fn alternatives() {
    let grammar = r#"
        bool: true; false.
        true: "true".
        false: "false".
    "#;
    let r1 = parse_to_xml(grammar, "true");
    assert!(r1.is_ok(), "Failed on 'true': {r1:?}");

    let r2 = parse_to_xml(grammar, "false");
    assert!(r2.is_ok(), "Failed on 'false': {r2:?}");
}

#[test]
fn repeat_zero_or_more() {
    let grammar = r#"
        list: item*.
        item: ["a"-"z"].
    "#;
    let r1 = parse_to_xml(grammar, "");
    assert!(r1.is_ok(), "Failed on empty: {r1:?}");

    let r2 = parse_to_xml(grammar, "abc");
    assert!(r2.is_ok(), "Failed on 'abc': {r2:?}");
}

#[test]
fn insertion() {
    let grammar = r#"
        greeting: +"Hello ", name.
        name: ["A"-"Z"], ["a"-"z"]*.
    "#;
    let xml = parse_to_xml(grammar, "World").unwrap();
    assert!(xml.contains("Hello "), "Missing insertion: {xml}");
}

#[test]
fn json_attributes_prefixed() {
    let grammar = r#"
        item: @id, value.
        @id: ["0"-"9"]+.
        value: ["a"-"z"]+.
    "#;
    let json = parse_to_json(grammar, "42abc").unwrap();
    assert!(json.contains("\"@id\""), "Missing @id in JSON: {json}");
}

#[test]
fn json_repeated_elements_as_array() {
    let grammar = r#"
        csv: row+.
        row: name, -",", age, -nl?.
        name: char+.
        age: digit+.
        -char: ["a"-"z"].
        -digit: ["0"-"9"].
        -nl: #0A.
    "#;
    let json = parse_to_json(grammar, "alice,30\nbob,42").unwrap();
    assert!(json.contains("\"row\": ["), "Expected row array: {json}");
    assert!(json.contains("\"alice\""), "Missing alice: {json}");
    assert!(json.contains("\"bob\""), "Missing bob: {json}");
}

#[test]
fn unicode_letter_class() {
    let grammar = r"
        word: [L]+.
    ";
    let xml = parse_to_xml(grammar, "Caf\u{00e9}").unwrap();
    assert!(
        xml.contains("Caf\u{00e9}"),
        "Unicode letters not matched: {xml}"
    );
}

#[test]
fn unicode_digit_class() {
    let grammar = r"
        num: [Nd]+.
    ";
    let json = parse_to_json(grammar, "42").unwrap();
    assert!(json.contains("42"), "Nd class didn't match digits: {json}");
}

#[test]
fn hex_literal_in_grammar() {
    let grammar = r#"
        line: word, -#20, word.
        word: ["a"-"z"]+.
    "#;
    let xml = parse_to_xml(grammar, "hello world").unwrap();
    assert!(
        xml.contains("<word>hello</word>"),
        "Missing first word: {xml}"
    );
    assert!(
        xml.contains("<word>world</word>"),
        "Missing second word: {xml}"
    );
}

#[test]
fn grammar_comments_ignored() {
    let grammar = r#"
        {this is a comment}
        greeting: "hi", -" ", name.
        {another comment}
        name: ["a"-"z"]+.
    "#;
    let json = parse_to_json(grammar, "hi world").unwrap();
    assert!(json.contains("world"), "Comment broke parsing: {json}");
}

#[test]
fn grammar_error_duplicate_rule() {
    let grammar = r"
        foo: 'a'.
        foo: 'b'.
    ";
    let result = parse_grammar(grammar);
    assert!(result.is_err());
    let err = format!("{}", result.unwrap_err());
    assert!(
        err.contains("foo"),
        "Should mention duplicate rule name: {err}"
    );
}

#[test]
fn empty_input_with_star() {
    let grammar = r#"
        root: item*.
        item: ["a"-"z"].
    "#;
    let json = parse_to_json(grammar, "").unwrap();
    assert!(
        json.contains("#text"),
        "Empty star should produce output: {json}"
    );
    let xml = parse_to_xml(grammar, "").unwrap();
    assert!(
        xml.contains("<root"),
        "Empty star should produce root: {xml}"
    );
}

#[test]
fn single_element_not_array() {
    let grammar = r#"
        doc: item.
        item: ["a"-"z"]+.
    "#;
    let json = parse_to_json(grammar, "hello").unwrap();
    assert!(
        !json.contains('['),
        "Single element should not be array: {json}"
    );
}

#[test]
fn mixed_marks_json() {
    let grammar = r#"
        tag: @id, -":", label.
        @id: ["0"-"9"]+.
        label: ["a"-"z"]+.
    "#;
    let json = parse_to_json(grammar, "42:hello").unwrap();
    assert!(
        json.contains("\"@id\": \"42\""),
        "Missing attribute: {json}"
    );
    assert!(json.contains("\"label\""), "Missing element: {json}");
    assert!(json.contains("hello"), "Missing label text: {json}");
}

#[test]
fn long_repeated_input() {
    let grammar = r#"
        list: item+.
        item: ["a"-"z"].
    "#;
    let input: String = (0..20_u32)
        .map(|i| (b'a' + (i % 26) as u8) as char)
        .collect();
    let json = parse_to_json(grammar, &input).unwrap();
    assert!(
        json.contains("\"item\": ["),
        "Repeated items should produce array: {json}"
    );
}

#[test]
fn nested_elements_json() {
    let grammar = r#"
        doc: section.
        section: title, body.
        title: ["A"-"Z"]+.
        body: ["a"-"z"]+.
    "#;
    let json = parse_to_json(grammar, "HELLOworld").unwrap();
    assert!(json.contains("\"section\""), "Missing section: {json}");
    assert!(json.contains("\"title\""), "Missing title: {json}");
    assert!(json.contains("\"body\""), "Missing body: {json}");
}

#[test]
fn equals_sign_separator() {
    let grammar = r#"
        pair: key, -"=", value.
        key: letter+.
        value: vchar+.
        -letter: ["a"-"z"].
        -vchar: ["a"-"z"; "0"-"9"].
    "#;
    let json = parse_to_json(grammar, "width=100px").unwrap();
    let xml = parse_to_xml(grammar, "width=100px").unwrap();
    assert!(json.contains("\"key\""), "Missing key in JSON: {json}");
    assert!(!json.contains('='), "Separator leaked into JSON: {json}");
    assert!(
        xml.contains("<key>width</key>"),
        "Missing key in XML: {xml}"
    );
}

#[test]
fn yaml_simple() {
    let grammar = r#"
        row: name, -",", age.
        name: ["a"-"z"]+.
        age: ["0"-"9"]+.
    "#;
    let yaml = parse_to_yaml(grammar, "alice,30").unwrap();
    assert!(yaml.starts_with("---\n"), "Missing YAML header: {yaml}");
    assert!(yaml.contains("name: alice"), "Missing name: {yaml}");
    assert!(yaml.contains("age: 30"), "Missing age: {yaml}");
}

#[test]
fn yaml_repeated_as_list() {
    let grammar = r#"
        csv: row+.
        row: name, -",", age, -nl?.
        name: ["a"-"z"]+.
        age: ["0"-"9"]+.
        -nl: #0A.
    "#;
    let yaml = parse_to_yaml(grammar, "alice,30\nbob,42").unwrap();
    assert!(yaml.contains("- "), "Repeated rows should use list: {yaml}");
    assert!(yaml.contains("alice"), "Missing alice: {yaml}");
    assert!(yaml.contains("bob"), "Missing bob: {yaml}");
}

#[test]
fn sql_single_row() {
    let grammar = r#"
        row: name, -",", age.
        name: ["a"-"z"]+.
        age: ["0"-"9"]+.
    "#;
    let sql = parse_to_sql(grammar, "alice,30").unwrap();
    assert!(sql.contains("INSERT INTO row"), "Missing INSERT: {sql}");
    assert!(sql.contains("(name, age)"), "Missing columns: {sql}");
    assert!(sql.contains("('alice', '30')"), "Missing values: {sql}");
}

#[test]
fn sql_multiple_rows() {
    let grammar = r#"
        csv: row+.
        row: name, -",", age, -nl?.
        name: ["a"-"z"]+.
        age: ["0"-"9"]+.
        -nl: #0A.
    "#;
    let sql = parse_to_sql(grammar, "alice,30\nbob,42").unwrap();
    let lines: Vec<&str> = sql.lines().collect();
    assert_eq!(lines.len(), 2, "Expected 2 INSERT statements: {sql}");
    assert!(lines[0].contains("INSERT INTO csv"), "Wrong table: {sql}");
    assert!(lines[0].contains("'alice'"), "Missing alice: {sql}");
    assert!(lines[1].contains("'bob'"), "Missing bob: {sql}");
}

#[test]
fn sql_escapes_quotes() {
    let grammar = r#"
        row: name, -",", value.
        name: ["a"-"z"]+.
        value: ~[","]+.
    "#;
    let sql = parse_to_sql(grammar, "key,it's").unwrap();
    assert!(
        sql.contains("''s"),
        "Single quotes should be escaped: {sql}"
    );
}

#[test]
fn sql_with_attributes() {
    let grammar = r#"
        decl: @prop, -":", @val.
        @prop: ["a"-"z"]+.
        @val: ["a"-"z"; "0"-"9"]+.
    "#;
    let sql = parse_to_sql(grammar, "width:100px").unwrap();
    assert!(
        sql.contains("(prop, val)"),
        "Attributes should become columns: {sql}"
    );
    assert!(sql.contains("('width', '100px')"), "Wrong values: {sql}");
}
