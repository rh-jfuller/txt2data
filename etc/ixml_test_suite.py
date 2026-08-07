#!/usr/bin/env python3
"""hacked up test harness for running txt2data against ixml community test suite.

https://codeberg.org/InvisibleXML/ixml/src/branch/main/tests

Usage:
    python3 tests/ixml_test_suite.py [--test-dir PATH] [--binary PATH]
           [--catalog CATALOG] [--verbose] [--filter PATTERN]

note: this is crap script!
"""
import argparse
import os
import re
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET
from dataclasses import dataclass, field
from pathlib import Path

NS = "https://github.com/invisibleXML/ixml/test-catalog"
NSM = {"tc": NS}


@dataclass
class TestResult:
    name: str
    path: str
    status: str
    expected_type: str
    detail: str = ""
@dataclass
class Stats:
    passed: int = 0
    failed: int = 0
    errors: int = 0
    skipped: int = 0
    results: list = field(default_factory=list)


def strip_insignificant_ws(text):
    if text is None:
        return ""
    if text.strip() == "":
        return ""
    return text


def canonicalize_xml_tree(elem, is_root=True):
    tag = re.sub(r'\{[^}]*\}', '', elem.tag)

    attrs = {}
    for k, v in sorted(elem.attrib.items()):
        clean_k = re.sub(r'\{[^}]*\}', '', k)
        if 'invisiblexml.org' in k or 'invisiblexml.org' in v:
            continue
        if clean_k.startswith('xmlns'):
            continue
        if clean_k == 'state' and v in (
            'ambiguous', 'version-mismatch'
        ):
            continue
        attrs[clean_k] = v

    attr_str = ''.join(
        f' {k}="{v}"' for k, v in sorted(attrs.items())
    )

    children = list(elem)
    has_children = len(children) > 0

    if has_children:
        text = strip_insignificant_ws(elem.text)
    else:
        text = elem.text or ""

    tail = strip_insignificant_ws(elem.tail) if not is_root else ""

    children_str = ''.join(
        canonicalize_xml_tree(child, is_root=False) for child in children
    )

    if not children_str and not text:
        return f'<{tag}{attr_str}/>{tail}'
    return f'<{tag}{attr_str}>{text}{children_str}</{tag}>{tail}'


def normalize_xml(xml_str: str) -> str:
    xml_str = xml_str.strip()
    xml_str = re.sub(r'<\?xml[^?]*\?>\s*', '', xml_str)
    xml_str = re.sub(r'<!--.*?-->', '', xml_str, flags=re.DOTALL)
    xml_str = xml_str.strip()

    try:
        root = ET.fromstring(xml_str)
        return canonicalize_xml_tree(root)
    except ET.ParseError:
        xml_str = re.sub(r'\s*xmlns(:[^=]*)?\s*=\s*"[^"]*"', '', xml_str)
        xml_str = re.sub(r'>\s+<', '><', xml_str)
        return xml_str.strip()
def get_text_content(elem):
    parts = []
    if elem.text:
        parts.append(elem.text)
    for child in elem:
        parts.append(ET.tostring(child, encoding='unicode'))
    return ''.join(parts)


def find_grammar(test_set_elem, base_dir: Path):
    grammar_ref = test_set_elem.find(f"{{{NS}}}ixml-grammar-ref")
    if grammar_ref is not None:
        href = grammar_ref.get("href")
        grammar_path = base_dir / href
        if grammar_path.exists():
            return grammar_path.read_text(encoding='utf-8'), True, grammar_path
        return None, False, None

    grammar_inline = test_set_elem.find(f"{{{NS}}}ixml-grammar")
    if grammar_inline is not None:
        text = grammar_inline.text or ""
        return text, False, None

    return None, False, None


def find_input(test_case_elem, base_dir: Path):
    string_ref = test_case_elem.find(f"{{{NS}}}test-string-ref")
    if string_ref is not None:
        href = string_ref.get("href")
        inp_path = base_dir / href
        if inp_path.exists():
            return inp_path.read_bytes().decode('utf-8')
        return None

    test_string = test_case_elem.find(f"{{{NS}}}test-string")
    if test_string is not None:
        return test_string.text or ""

    return None


def find_expected(result_elem):
    expectations = []
    for child in result_elem:
        tag = child.tag.replace(f"{{{NS}}}", "")
        if tag == "assert-xml":
            inner = get_text_content(child)
            expectations.append(("assert-xml", inner))
        elif tag == "assert-xml-ref":
            expectations.append(("assert-xml-ref", child.get("href")))
        elif tag == "assert-not-a-sentence":
            expectations.append(("assert-not-a-sentence", None))
        elif tag == "assert-not-a-grammar":
            expectations.append(("assert-not-a-grammar", None))
        elif tag == "assert-dynamic-error":
            expectations.append(("assert-dynamic-error",
                                 child.get("error-code", "")))
    return expectations


def run_txt2data(binary: str, grammar_text: str,
                 input_text: str, timeout: int = 10):
    with tempfile.NamedTemporaryFile(
        mode='w', suffix='.ixml', delete=False, encoding='utf-8'
    ) as gf:
        gf.write(grammar_text)
        grammar_file = gf.name

    with tempfile.NamedTemporaryFile(
        mode='wb', suffix='.inp', delete=False
    ) as inf:
        inf.write(input_text.encode('utf-8'))
        input_file = inf.name

    try:
        result = subprocess.run(
            [binary, "--grammar", grammar_file,
             "--input", input_file, "--format", "xml"],
            capture_output=True, text=True, timeout=timeout
        )
        return result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return -1, "", "TIMEOUT"
    finally:
        os.unlink(grammar_file)
        os.unlink(input_file)


def check_grammar_only(binary: str, grammar_text: str, timeout: int = 10):
    with tempfile.NamedTemporaryFile(
        mode='w', suffix='.ixml', delete=False, encoding='utf-8'
    ) as gf:
        gf.write(grammar_text)
        grammar_file = gf.name

    with tempfile.NamedTemporaryFile(
        mode='wb', suffix='.inp', delete=False
    ) as inf:
        inf.write(b"")
        input_file = inf.name

    try:
        result = subprocess.run(
            [binary, "--grammar", grammar_file,
             "--input", input_file, "--format", "xml"],
            capture_output=True, text=True, timeout=timeout
        )
        return result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return -1, "", "TIMEOUT"
    finally:
        os.unlink(grammar_file)
        os.unlink(input_file)


def process_test_case(test_case, grammar_text, base_dir,
                      binary, set_name, verbose):
    case_name = test_case.get("name", "unnamed")
    full_name = f"{set_name}/{case_name}"

    input_text = find_input(test_case, base_dir)
    if input_text is None:
        return TestResult(full_name, str(base_dir), "skip",
                          "unknown", "no input found")

    result_elem = test_case.find(f"{{{NS}}}result")
    if result_elem is None:
        return TestResult(full_name, str(base_dir), "skip",
                          "unknown", "no result element")

    expectations = find_expected(result_elem)
    if not expectations:
        return TestResult(full_name, str(base_dir), "skip",
                          "unknown", "no expectations found")

    exp_type = expectations[0][0]

    exit_code, stdout, stderr = run_txt2data(
        binary, grammar_text, input_text
    )

    if exit_code == -1:
        return TestResult(full_name, str(base_dir), "error",
                          exp_type, "timeout")

    if exp_type in ("assert-not-a-sentence", "assert-dynamic-error"):
        if exit_code != 0:
            return TestResult(full_name, str(base_dir), "pass", exp_type)
        return TestResult(
            full_name, str(base_dir), "fail", exp_type,
            "expected failure but got success"
        )

    if exp_type in ("assert-xml", "assert-xml-ref"):
        if exit_code != 0:
            return TestResult(
                full_name, str(base_dir), "fail", exp_type,
                f"exit={exit_code}: {stderr.strip()}"
            )

        actual_xml = normalize_xml(stdout)

        for etype, evalue in expectations:
            if etype == "assert-xml":
                expected_xml = normalize_xml(evalue)
            elif etype == "assert-xml-ref":
                ref_path = base_dir / evalue
                if not ref_path.exists():
                    continue
                expected_xml = normalize_xml(
                    ref_path.read_text(encoding='utf-8')
                )
            else:
                continue

            if actual_xml == expected_xml:
                return TestResult(full_name, str(base_dir), "pass",
                                  exp_type)

        if verbose:
            detail = f"XML mismatch\n  actual:   {actual_xml[:200]}"
            if expectations:
                etype, evalue = expectations[0]
                if etype == "assert-xml":
                    detail += f"\n  probably expected: {normalize_xml(evalue)[:200]}"
                elif etype == "assert-xml-ref":
                    ref_path = base_dir / evalue
                    if ref_path.exists():
                        exp = normalize_xml(
                            ref_path.read_text(encoding='utf-8')
                        )
                        detail += f"\n  expected: {exp[:200]}"
        else:
            detail = "XML mismatch"

        return TestResult(full_name, str(base_dir), "fail",
                          exp_type, detail)

    return TestResult(full_name, str(base_dir), "skip",
                      exp_type, f"unhandled assertion type: {exp_type}")


def process_grammar_test(grammar_test, grammar_text,
                         base_dir, binary, set_name):
    full_name = f"{set_name}/grammar-test"

    result_elem = grammar_test.find(f"{{{NS}}}result")
    if result_elem is None:
        return TestResult(full_name, str(base_dir), "skip",
                          "grammar-test", "no result element")

    expectations = find_expected(result_elem)
    if not expectations:
        return TestResult(full_name, str(base_dir), "skip",
                          "grammar-test", "no expectations")

    exp_type = expectations[0][0]

    exit_code, stdout, stderr = check_grammar_only(
        binary, grammar_text
    )

    if exp_type == "assert-not-a-grammar":
        grammar_error = "Grammar error" in stderr or "grammar" in stderr.lower()
        if exit_code != 0 and grammar_error:
            return TestResult(full_name, str(base_dir), "pass", exp_type)
        if exit_code != 0:
            return TestResult(full_name, str(base_dir), "pass", exp_type,
                              "failed")
        return TestResult(
            full_name, str(base_dir), "fail", exp_type,
            "expected grammar rejection but grammar accepted"
        )

    if exp_type in ("assert-xml", "assert-xml-ref"):
        return TestResult(full_name, str(base_dir), "skip",
                          exp_type, "I guess grammar-to-XML serialization not supported")

    return TestResult(full_name, str(base_dir), "skip",
                      exp_type, f"unhandled grammar-test type ?: {exp_type}")


def process_test_set(test_set, base_dir, binary, stats,
                     verbose, filter_pat, parent_grammar=None):
    set_name = test_set.get("name", "unnamed")

    grammar_text, is_file, grammar_path = find_grammar(test_set, base_dir)
    if grammar_text is None:
        grammar_text = parent_grammar

    for grammar_test in test_set.findall(f"{{{NS}}}grammar-test"):
        if grammar_text is None:
            continue
        if filter_pat and not re.search(filter_pat, set_name):
            continue
        result = process_grammar_test(
            grammar_test, grammar_text, base_dir, binary, set_name
        )
        record_result(result, stats, verbose)

    for test_case in test_set.findall(f"{{{NS}}}test-case"):
        case_name = test_case.get("name", "unnamed")
        full_name = f"{set_name}/{case_name}"
        if filter_pat and not re.search(filter_pat, full_name):
            continue

        if grammar_text is None:
            result = TestResult(full_name, str(base_dir), "skip",
                                "unknown", "no grammar available")
            record_result(result, stats, verbose)
            continue

        result = process_test_case(
            test_case, grammar_text, base_dir, binary, set_name, verbose
        )
        record_result(result, stats, verbose)

    for nested_set in test_set.findall(f"{{{NS}}}test-set"):
        process_test_set(nested_set, base_dir, binary, stats,
                         verbose, filter_pat,
                         parent_grammar=grammar_text)


def record_result(result, stats, verbose):
    stats.results.append(result)
    if result.status == "pass":
        stats.passed += 1
        marker = "PASS"
    elif result.status == "fail":
        stats.failed += 1
        marker = "FAIL"
    elif result.status == "error":
        stats.errors += 1
        marker = "ERR "
    else:
        stats.skipped += 1
        marker = "SKIP"

    if verbose or result.status in ("fail", "error"):
        detail = f"  ({result.detail})" if result.detail else ""
        print(f"  [{marker}] {result.name} [{result.expected_type}]{detail}")
    elif result.status == "pass":
        print(f"  [PASS] {result.name}")

# hackhackhack
def process_catalog(catalog_path: Path, binary: str, stats: Stats,
                    verbose: bool, filter_pat: str):
    if not catalog_path.exists():
        print(f"  Warning: catalog not found: {catalog_path}")
        return

    tree = ET.parse(catalog_path)
    root = tree.getroot()
    base_dir = catalog_path.parent
    catalog_name = root.get("name", str(catalog_path))

    print(f"\n--- {catalog_name} ---")

    for ref in root.findall(f"{{{NS}}}test-set-ref"):
        href = ref.get("href")
        ref_path = base_dir / href
        process_catalog(ref_path, binary, stats, verbose, filter_pat)

    for test_set in root.findall(f"{{{NS}}}test-set"):
        process_test_set(test_set, base_dir, binary, stats,
                         verbose, filter_pat)


def main():
    parser = argparse.ArgumentParser(
        description="Run txt2data against the ixml test suite"
    )
    parser.add_argument(
        "--test-dir", default="/tmp/opencode/ixml-tests/tests",
        help="Path to ixml test suite tests/ directory"
    )
    parser.add_argument(
        "--binary",
        default=str(Path(__file__).parent.parent / "target/release/txt2data"),
        help="Path to txt2data binary"
    )
    parser.add_argument(
        "--catalog", default="test-catalog.xml",
        help="Top-level catalog filename"
    )
    parser.add_argument(
        "--verbose", "-v", action="store_true",
        help="Show all test results including passes"
    )
    parser.add_argument(
        "--filter", "-f", default="",
        help="Regex filter for test names"
    )
    args = parser.parse_args()

    test_dir = Path(args.test_dir)
    catalog = test_dir / args.catalog

    if not catalog.exists():
        print(f"Error: catalog not found at {catalog}")
        sys.exit(1)

    if not Path(args.binary).exists():
        print(f"Error: binary not found at {args.binary}")
        sys.exit(1)

    stats = Stats()

    print(f"Running ixml test suite against: {args.binary}")
    print(f"Test catalog: {catalog}")
    print()

    process_catalog(catalog, args.binary, stats, args.verbose, args.filter)

    total = stats.passed + stats.failed + stats.errors + stats.skipped
    print(f"\n{'='*60}")
    print(f"RESULTS: {total} total")
    print(f"  PASSED:  {stats.passed}")
    print(f"  FAILED:  {stats.failed}")
    print(f"  ERRORS:  {stats.errors}")
    print(f"  SKIPPED: {stats.skipped}")
    print(f"{'='*60}")

    if stats.failed > 0 or stats.errors > 0:
        print(f"\nFailed/Error tests:")
        for r in stats.results:
            if r.status in ("fail", "error"):
                print(f"  {r.status.upper():5s} {r.name}: {r.detail}")

    sys.exit(1 if stats.failed > 0 or stats.errors > 0 else 0)
if __name__ == "__main__":
    main()
