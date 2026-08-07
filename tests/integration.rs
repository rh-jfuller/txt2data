#![expect(clippy::unwrap_used)]

use txt2data::{
    parse_grammar, parse_to_json, parse_to_sexp, parse_to_sql, parse_to_xml, parse_to_yaml,
};

// ── Grammar parsing edge cases ──────────────────────────────────────

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
    assert!(sql.contains("INSERT INTO \"row\""), "Missing INSERT: {sql}");
    assert!(
        sql.contains("(\"name\", \"age\")"),
        "Missing columns: {sql}"
    );
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
    assert!(
        lines[0].contains("INSERT INTO \"csv\""),
        "Wrong table: {sql}"
    );
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
        sql.contains("(\"prop\", \"val\")"),
        "Attributes should become columns: {sql}"
    );
    assert!(sql.contains("('width', '100px')"), "Wrong values: {sql}");
}

// ── Grammar parsing edge cases ──────────────────────────────────────

#[test]
fn single_quoted_literal() {
    let grammar = "root: 'hello'.";
    let xml = parse_to_xml(grammar, "hello").unwrap();
    assert!(xml.contains("<root>hello</root>"), "Bad output: {xml}");
}

#[test]
fn doubled_double_quote_escape() {
    let grammar = r#"root: "he""llo"."#;
    let xml = parse_to_xml(grammar, "he\"llo").unwrap();
    assert!(
        xml.contains("he\"llo"),
        "Doubled quote not unescaped: {xml}"
    );
}

#[test]
fn doubled_single_quote_escape() {
    let grammar = "root: 'it''s'.";
    let xml = parse_to_xml(grammar, "it's").unwrap();
    assert!(xml.contains("it's"), "Doubled quote not unescaped: {xml}");
}

#[test]
fn ixml_version_prolog() {
    let grammar = r#"ixml version "1.0" . root: "ok"."#;
    let json = parse_to_json(grammar, "ok").unwrap();
    assert!(json.contains("ok"), "Prolog broke parsing: {json}");
}

#[test]
fn ixml_version_prolog_single_quoted() {
    let grammar = "ixml version '1.1' . root: 'ok'.";
    let json = parse_to_json(grammar, "ok").unwrap();
    assert!(json.contains("ok"), "Prolog broke parsing: {json}");
}

#[test]
fn rule_name_starting_with_ixml() {
    let grammar = "ixmldata: 'x'.";
    let json = parse_to_json(grammar, "x").unwrap();
    // Root element name doesn't appear as key in JSON, but parse succeeds
    // (i.e., "ixmldata" wasn't consumed as an ixml prolog).
    assert!(json.contains('x'), "Rule name swallowed as prolog: {json}");
}

#[test]
fn equals_sign_rule_separator() {
    let grammar = "root = 'ok'.";
    let json = parse_to_json(grammar, "ok").unwrap();
    assert!(json.contains("ok"), "= separator failed: {json}");
}

#[test]
fn pipe_alternative_separator() {
    let grammar = "root: 'a' | 'b'.";
    let r1 = parse_to_xml(grammar, "a");
    assert!(r1.is_ok(), "Failed on 'a': {r1:?}");
    let r2 = parse_to_xml(grammar, "b");
    assert!(r2.is_ok(), "Failed on 'b': {r2:?}");
}

#[test]
fn nested_comments() {
    let grammar = "{outer {inner} still outer} root: 'ok'.";
    let json = parse_to_json(grammar, "ok").unwrap();
    assert!(json.contains("ok"), "Nested comment broke parsing: {json}");
}

#[test]
fn comment_inside_rule_body() {
    let grammar = "root: 'a' {mid-rule comment} , 'b'.";
    let xml = parse_to_xml(grammar, "ab").unwrap();
    assert!(xml.contains("ab"), "Mid-rule comment broke parse: {xml}");
}

#[test]
fn implicit_sequence_no_commas() {
    let grammar = "root: 'a' 'b' 'c'.";
    let xml = parse_to_xml(grammar, "abc").unwrap();
    assert!(xml.contains("abc"), "Implicit sequence failed: {xml}");
}

#[test]
fn element_mark_caret() {
    let grammar = r#"
        root: ^item.
        ^item: ["a"-"z"]+.
    "#;
    let xml = parse_to_xml(grammar, "hello").unwrap();
    assert!(
        xml.contains("<item>hello</item>"),
        "^ mark should produce element: {xml}"
    );
    let json = parse_to_json(grammar, "hello").unwrap();
    assert!(json.contains("\"item\""), "^ mark missing in JSON: {json}");
}

#[test]
fn charset_multiple_members() {
    let grammar = r#"root: ["abc"; "0"-"9"]+."#;
    let xml = parse_to_xml(grammar, "a3b7c").unwrap();
    assert!(xml.contains("a3b7c"), "Multi-member charset failed: {xml}");
}

#[test]
fn multi_char_charset_inclusion() {
    let grammar = r#"root: ["aeiou"]+."#;
    let xml = parse_to_xml(grammar, "aeiou").unwrap();
    assert!(xml.contains("aeiou"), "Multi-char chars failed: {xml}");
}

// ── Earley parser edge cases ────────────────────────────────────────

#[test]
fn deeply_nested_rules() {
    let grammar = r#"
        a: b.
        b: c.
        c: d.
        d: ["x"].
    "#;
    let json = parse_to_json(grammar, "x").unwrap();
    assert!(json.contains("\"d\""), "Deep nesting missing leaf: {json}");
    assert!(json.contains("\"c\""), "Deep nesting missing c: {json}");
    assert!(json.contains("\"b\""), "Deep nesting missing b: {json}");
}

#[test]
fn emoji_in_literal() {
    let grammar = "root: \"\u{1F389}\".";
    let xml = parse_to_xml(grammar, "\u{1F389}").unwrap();
    assert!(xml.contains("\u{1F389}"), "Emoji not matched: {xml}");
}

#[test]
fn multibyte_unicode_input() {
    let grammar = r"root: [L]+.";
    let xml = parse_to_xml(grammar, "\u{00FC}ber").unwrap();
    assert!(
        xml.contains("\u{00FC}ber"),
        "Multibyte unicode failed: {xml}"
    );
}

#[test]
fn partial_match_error_message() {
    let grammar = r#"root: "abc"."#;
    let result = parse_to_json(grammar, "abd");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Parse error"), "Bad error format: {err}");
}

#[test]
fn single_char_vs_multichar_literal() {
    let grammar = "root: 'hello'.";
    let result = parse_to_json(grammar, "h");
    assert!(result.is_err(), "Should fail on partial literal");
}

#[test]
fn group_single_term_inlined() {
    // Single-alt, single-term groups are inlined. Multi-term or
    // multi-alt groups produce synthetic nonterminals (known limitation).
    let grammar = "root: ('a'), 'b'.";
    let xml = parse_to_xml(grammar, "ab").unwrap();
    assert!(xml.contains("ab"), "Single-term group failed: {xml}");
}

// ── Serialization edge cases ────────────────────────────────────────

#[test]
fn xml_escapes_special_chars() {
    let grammar = r"root: val.
        val: ~[]+.";
    let xml = parse_to_xml(grammar, "<b>A&B</b>").unwrap();
    assert!(
        xml.contains("&lt;b>A&amp;B&lt;/b>"),
        "XML not escaped: {xml}"
    );
}

#[test]
fn xml_attribute_escapes_quotes() {
    let grammar = r"
        root: @val.
        @val: ~[]+.
    ";
    let xml = parse_to_xml(grammar, "say \"hi\"").unwrap();
    assert!(
        xml.contains("&quot;"),
        "Attribute quotes not escaped: {xml}"
    );
}

#[test]
fn json_escapes_special_chars() {
    let grammar = "root: val.\nval: ~[]+.";
    let json = parse_to_json(grammar, "line1\nline2").unwrap();
    assert!(
        json.contains("line1\\nline2"),
        "JSON newline not escaped: {json}"
    );
}

#[test]
fn json_escapes_backslash() {
    let grammar = "root: val.\nval: ~[]+.";
    let json = parse_to_json(grammar, "a\\b").unwrap();
    assert!(
        json.contains("a\\\\b"),
        "JSON backslash not escaped: {json}"
    );
}

#[test]
fn yaml_quotes_special_values() {
    let grammar = "root: val.\nval: ~[]+.";

    let yaml_true = parse_to_yaml(grammar, "true").unwrap();
    assert!(
        yaml_true.contains("\"true\""),
        "YAML 'true' should be quoted: {yaml_true}"
    );

    let yaml_null = parse_to_yaml(grammar, "null").unwrap();
    assert!(
        yaml_null.contains("\"null\""),
        "YAML 'null' should be quoted: {yaml_null}"
    );
}

#[test]
fn yaml_quotes_colon_value() {
    let grammar = "root: val.\nval: ~[]+.";
    let yaml = parse_to_yaml(grammar, "key: value").unwrap();
    assert!(
        yaml.contains("\"key: value\""),
        "YAML colon value should be quoted: {yaml}"
    );
}

#[test]
fn insertion_in_json() {
    // Insertions appear in XML output but JSON only captures
    // element/text structure from matched input.
    let grammar = r#"
        greeting: +"Hello ", name.
        name: ["A"-"Z"], ["a"-"z"]*.
    "#;
    let json = parse_to_json(grammar, "World").unwrap();
    assert!(
        json.contains("World"),
        "Insertion test: name missing in JSON: {json}"
    );
}

#[test]
fn insertion_in_yaml() {
    let grammar = r#"
        greeting: +"hello ", name.
        name: ["A"-"Z"], ["a"-"z"]*.
    "#;
    let yaml = parse_to_yaml(grammar, "World").unwrap();
    assert!(
        yaml.contains("World"),
        "Insertion test: name missing in YAML: {yaml}"
    );
}

#[test]
fn insertion_in_sql() {
    let grammar = r#"
        root: name.
        name: +"Dr. ", raw.
        raw: ["a"-"z"]+.
    "#;
    let sql = parse_to_sql(grammar, "smith").unwrap();
    assert!(sql.contains("Dr. smith"), "Insertion missing in SQL: {sql}");
}

#[test]
fn hex_insertion() {
    // +#20 inserts a space into output (not matched in input).
    // Input has no space; the two words are distinguished by case.
    let grammar = r#"
        root: upper, +#20, lower.
        upper: ["A"-"Z"]+.
        lower: ["a"-"z"]+.
    "#;
    let xml = parse_to_xml(grammar, "ABCdef").unwrap();
    assert!(xml.contains(' '), "Hex insertion missing space: {xml}");
}

#[test]
fn xml_self_closing_empty_element() {
    let grammar = "root: -'x'.";
    let xml = parse_to_xml(grammar, "x").unwrap();
    assert!(
        xml.contains("<root/>"),
        "Empty element should self-close: {xml}"
    );
}

#[test]
fn deeply_nested_json_structure() {
    let grammar = r#"
        a: b.
        b: c.
        c: d.
        d: ["x"].
    "#;
    let json = parse_to_json(grammar, "x").unwrap();
    // Root element "a" is implicit (not a key); children are nested.
    assert!(json.contains("\"b\""), "Missing b: {json}");
    assert!(json.contains("\"c\""), "Missing c: {json}");
    assert!(json.contains("\"d\""), "Missing d: {json}");
}

#[test]
fn sql_attributes_in_repeated_rows() {
    let grammar = r#"
        csv: row+.
        row: @id, name, -nl?.
        @id: ["0"-"9"]+.
        name: ["a"-"z"]+.
        -nl: #0A.
    "#;
    let sql = parse_to_sql(grammar, "1alice\n2bob").unwrap();
    let lines: Vec<&str> = sql.lines().collect();
    assert_eq!(lines.len(), 2, "Expected 2 INSERT statements: {sql}");
    assert!(sql.contains("(\"id\", \"name\")"), "Missing columns: {sql}");
    assert!(lines[0].contains("'1'"), "Missing id 1: {sql}");
    assert!(lines[0].contains("'alice'"), "Missing alice: {sql}");
    assert!(lines[1].contains("'2'"), "Missing id 2: {sql}");
    assert!(lines[1].contains("'bob'"), "Missing bob: {sql}");
}

// ── Error cases ─────────────────────────────────────────────────────

#[test]
fn error_unclosed_double_quote() {
    let result = parse_grammar("root: \"hello.");
    assert!(result.is_err(), "Unclosed double quote should fail");
}

#[test]
fn error_unclosed_single_quote() {
    let result = parse_grammar("root: 'hello.");
    assert!(result.is_err(), "Unclosed single quote should fail");
}

#[test]
fn error_missing_period() {
    let result = parse_grammar("root: 'a'");
    assert!(result.is_err(), "Missing period should fail");
}

#[test]
fn error_bad_hex_code() {
    let result = parse_grammar("root: #GG.");
    assert!(result.is_err(), "Bad hex should fail");
}

#[test]
fn error_invalid_unicode_codepoint() {
    let result = parse_grammar("root: #D800.");
    assert!(result.is_err(), "Surrogate codepoint should fail");
}

#[test]
fn error_empty_grammar() {
    let grammar = parse_grammar("");
    assert!(grammar.is_ok(), "Empty grammar should parse");
    let result = parse_to_json("", "anything");
    assert!(result.is_err(), "Empty grammar should fail to parse input");
}

#[test]
fn error_whitespace_only_grammar() {
    let grammar = parse_grammar("   \n\t  ");
    assert!(grammar.is_ok(), "Whitespace grammar should parse");
}

#[test]
fn error_unclosed_charset() {
    let result = parse_grammar("root: ['a'-'z'.");
    assert!(result.is_err(), "Unclosed charset should fail");
}

#[test]
fn error_unclosed_group() {
    let result = parse_grammar("root: ('a'.");
    assert!(result.is_err(), "Unclosed group should fail");
}

// ── Character class edge cases ──────────────────────────────────────

#[test]
fn unicode_lowercase_letter_class() {
    let grammar = "root: [Ll]+.";
    let xml = parse_to_xml(grammar, "abc").unwrap();
    assert!(xml.contains("abc"), "Ll class failed: {xml}");
}

#[test]
fn unicode_uppercase_letter_class() {
    let grammar = "root: [Lu]+.";
    let xml = parse_to_xml(grammar, "ABC").unwrap();
    assert!(xml.contains("ABC"), "Lu class failed: {xml}");
}

#[test]
fn unicode_space_separator_class() {
    let grammar = "root: [Zs]+.";
    let xml = parse_to_xml(grammar, "   ").unwrap();
    assert!(xml.contains("   "), "Zs class failed: {xml}");
}

#[test]
fn unicode_currency_symbol_class() {
    let grammar = "root: [Sc]+.";
    let xml = parse_to_xml(grammar, "$").unwrap();
    assert!(xml.contains('$'), "Sc class failed: {xml}");
}

#[test]
fn unicode_connector_punctuation_class() {
    let grammar = "root: [Pc]+.";
    let xml = parse_to_xml(grammar, "_").unwrap();
    assert!(xml.contains('_'), "Pc class failed: {xml}");
}

#[test]
fn unicode_math_symbol_class() {
    let grammar = "root: [Sm]+.";
    let xml = parse_to_xml(grammar, "+").unwrap();
    assert!(xml.contains('+'), "Sm class failed: {xml}");
}

#[test]
fn unicode_punctuation_class() {
    let grammar = "root: [P]+.";
    let json = parse_to_json(grammar, ".,!?").unwrap();
    assert!(json.contains(".,!?"), "P class failed: {json}");
}

#[test]
fn hex_range_in_charset() {
    let grammar = "root: [#41-#5A]+.";
    let xml = parse_to_xml(grammar, "HELLO").unwrap();
    assert!(xml.contains("HELLO"), "Hex range failed: {xml}");
}

#[test]
fn hex_range_digits_in_charset() {
    let grammar = "root: [#30-#39]+.";
    let xml = parse_to_xml(grammar, "12345").unwrap();
    assert!(xml.contains("12345"), "Hex digit range failed: {xml}");
}

#[test]
fn exclusion_with_unicode_class() {
    let grammar = "root: ~[L]+.";
    let json = parse_to_json(grammar, "123!@#").unwrap();
    assert!(json.contains("123!@#"), "Exclusion ~[L] failed: {json}");
}

#[test]
fn unknown_unicode_category_no_match() {
    let grammar = "root: [Xx]+.";
    let result = parse_to_json(grammar, "a");
    assert!(result.is_err(), "Unknown category Xx should match nothing");
}

#[test]
fn charset_mixed_class_and_range() {
    let grammar = r#"root: [Ll; "0"-"9"]+."#;
    let xml = parse_to_xml(grammar, "abc123").unwrap();
    assert!(
        xml.contains("abc123"),
        "Mixed class+range charset failed: {xml}"
    );
}

#[test]
fn marked_hex_literal_hidden() {
    let grammar = "root: word, -#20, word.\nword: ['a'-'z']+.";
    let xml = parse_to_xml(grammar, "hello world").unwrap();
    assert!(
        xml.contains("<word>hello</word>"),
        "First word missing: {xml}"
    );
    assert!(
        xml.contains("<word>world</word>"),
        "Second word missing: {xml}"
    );
    assert!(!xml.contains(" </"), "Space should be hidden: {xml}");
}

// ── S-expression output ─────────────────────────────────────────────

#[test]
fn sexp_simple() {
    let grammar = r#"
        row: name, -",", age.
        name: ["a"-"z"]+.
        age: ["0"-"9"]+.
    "#;
    let sexp = parse_to_sexp(grammar, "alice,30").unwrap();
    assert!(sexp.contains("(name alice)"), "Missing name: {sexp}");
    assert!(sexp.contains("(age 30)"), "Missing age: {sexp}");
}

#[test]
fn sexp_nested() {
    let grammar = r#"
        doc: section.
        section: title, body.
        title: ["A"-"Z"]+.
        body: ["a"-"z"]+.
    "#;
    let sexp = parse_to_sexp(grammar, "HELLOworld").unwrap();
    assert!(sexp.contains("(section"), "Missing section: {sexp}");
    assert!(sexp.contains("(title HELLO)"), "Missing title: {sexp}");
    assert!(sexp.contains("(body world)"), "Missing body: {sexp}");
}

#[test]
fn sexp_attributes() {
    let grammar = r#"
        decl: @prop, -":", @val.
        @prop: ["a"-"z"]+.
        @val: ["a"-"z"; "0"-"9"]+.
    "#;
    let sexp = parse_to_sexp(grammar, "width:100px").unwrap();
    assert!(
        sexp.contains("(@prop width)"),
        "Missing attribute prop: {sexp}"
    );
    assert!(
        sexp.contains("(@val 100px)"),
        "Missing attribute val: {sexp}"
    );
}

#[test]
fn sexp_repeated() {
    let grammar = r#"
        csv: row+.
        row: name, -",", age, -nl?.
        name: ["a"-"z"]+.
        age: ["0"-"9"]+.
        -nl: #0A.
    "#;
    let sexp = parse_to_sexp(grammar, "alice,30\nbob,42").unwrap();
    assert!(sexp.contains("(csv"), "Missing csv: {sexp}");
    assert!(sexp.contains("(name alice)"), "Missing alice: {sexp}");
    assert!(sexp.contains("(name bob)"), "Missing bob: {sexp}");
    // Hidden newline should not leak.
    assert!(!sexp.contains("\\n"), "Hidden newline leaked: {sexp}");
}

#[test]
fn sexp_quoting_special_chars() {
    let grammar = "root: val.\nval: ~[]+.";
    let sexp = parse_to_sexp(grammar, "hello world").unwrap();
    assert!(
        sexp.contains("\"hello world\""),
        "Space should trigger quoting: {sexp}"
    );
}

#[test]
fn sexp_hidden_separator() {
    let grammar = r#"
        pair: key, -"=", value.
        key: ["a"-"z"]+.
        value: ["a"-"z"]+.
    "#;
    let sexp = parse_to_sexp(grammar, "foo=bar").unwrap();
    assert!(sexp.contains("(key foo)"), "Missing key: {sexp}");
    assert!(sexp.contains("(value bar)"), "Missing value: {sexp}");
    assert!(!sexp.contains('='), "Separator leaked: {sexp}");
}
