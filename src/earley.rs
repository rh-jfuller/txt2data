//! Earley parser: predict/scan/complete loop over normalized
//! BNF rules, then backtrack to extract a parse tree.
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use crate::ast::{Alt, CharMember, Grammar, Mark, Rule, Term};
use crate::charclass::{matches_exclusion, matches_inclusion};
use crate::serialize::{ParseTree, TreeNode};
use thiserror::Error;

type TreeCache = RefCell<HashMap<(usize, usize, usize), Option<ParseTree>>>;
type CompletionIndex = HashMap<(usize, usize), Vec<usize>>;

/// Shared context for tree extraction.
struct ExtractCtx<'a> {
    chart: &'a [ChartSet],
    chars: &'a [char],
    completions: &'a CompletionIndex,
    memo: &'a TreeCache,
    /// rule name -> unique index for cache keys.
    name_to_idx: &'a HashMap<String, usize>,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error(
        "parse failed at position {pos}: \
         expected {expected}"
    )]
    Failed { pos: usize, expected: String },

    #[error("input not fully consumed, stopped at {0}")]
    Incomplete(usize),
}

pub struct Parser {
    rules: Vec<NormRule>,
    root: String,
    /// name -> rule indices for O(1) predict.
    rule_index: HashMap<String, Vec<usize>>,
}

/// Rule after desugaring `+`/`*`/`?` into plain BNF.
#[derive(Debug, Clone)]
struct NormRule {
    mark: Mark,
    name: String,
    output_name: String,
    alts: Vec<Vec<Symbol>>,
}

#[derive(Debug, Clone)]
enum Symbol {
    Nonterminal {
        mark: Mark,
        name: String,
        output_name: Option<String>,
    },
    Literal {
        mark: Mark,
        value: String,
        char_len: usize,
    },
    Inclusion {
        mark: Mark,
        members: Vec<CharMember>,
    },
    Exclusion {
        mark: Mark,
        members: Vec<CharMember>,
    },
    Insertion {
        value: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EarleyItem {
    rule_idx: usize,
    alt_idx: usize,
    dot: usize,
    start: usize,
}

/// Ordered items with O(1) dedup via `HashSet`.
struct ChartSet {
    items: Vec<EarleyItem>,
    seen: HashSet<EarleyItem>,
}

impl ChartSet {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            seen: HashSet::new(),
        }
    }

    fn add(&mut self, item: EarleyItem) {
        if self.seen.insert(item.clone()) {
            self.items.push(item);
        }
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn get(&self, idx: usize) -> &EarleyItem {
        &self.items[idx]
    }

    fn iter(&self) -> std::slice::Iter<'_, EarleyItem> {
        self.items.iter()
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Parser {
    #[must_use]
    pub fn new(grammar: &Grammar) -> Self {
        let mut normalizer = Normalizer::new();
        for rule in &grammar.rules {
            normalizer.normalize_rule(rule);
        }

        let mut rule_index: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, rule) in normalizer.rules.iter().enumerate() {
            rule_index.entry(rule.name.clone()).or_default().push(idx);
        }

        Self {
            root: grammar.root_name().to_string(),
            rules: normalizer.rules,
            rule_index,
        }
    }

    /// # Errors
    ///
    /// Returns `ParseError` if the input doesn't match.
    pub fn parse(&self, input: &str) -> Result<ParseTree, ParseError> {
        let chars: Vec<char> = input.chars().collect();
        let n = chars.len();
        let chart = self.build_chart(&chars, n);

        // Build name -> integer index for allocation-free lookups.
        let mut name_to_idx: HashMap<String, usize> = HashMap::new();
        for rule in &self.rules {
            let len = name_to_idx.len();
            name_to_idx.entry(rule.name.clone()).or_insert(len);
        }

        let completions = self.build_completion_index(&chart, &name_to_idx);
        let memo: TreeCache = RefCell::new(HashMap::new());
        let ctx = ExtractCtx {
            chart: &chart,
            chars: &chars,
            completions: &completions,
            memo: &memo,
            name_to_idx: &name_to_idx,
        };
        self.find_parse(&ctx, n)
    }

    fn build_completion_index(
        &self,
        chart: &[ChartSet],
        name_to_idx: &HashMap<String, usize>,
    ) -> CompletionIndex {
        let mut index = CompletionIndex::new();
        for (end, set) in chart.iter().enumerate() {
            for item in set.iter() {
                let rule = &self.rules[item.rule_idx];
                let alt = &rule.alts[item.alt_idx];
                if item.dot == alt.len()
                    && let Some(&idx) = name_to_idx.get(&rule.name)
                {
                    index.entry((idx, item.start)).or_default().push(end);
                }
            }
        }
        index
    }

    fn build_chart(&self, chars: &[char], n: usize) -> Vec<ChartSet> {
        let mut chart: Vec<ChartSet> = (0..=n).map(|_| ChartSet::new()).collect();

        if let Some(indices) = self.rule_index.get(&self.root) {
            for &rule_idx in indices {
                let rule = &self.rules[rule_idx];
                for alt_idx in 0..rule.alts.len() {
                    chart[0].add(EarleyItem {
                        rule_idx,
                        alt_idx,
                        dot: 0,
                        start: 0,
                    });
                }
            }
        }

        for i in 0..=n {
            let mut j = 0;
            while j < chart[i].len() {
                let item = chart[i].get(j).clone();
                let rule = &self.rules[item.rule_idx];
                let alt = &rule.alts[item.alt_idx];

                if item.dot >= alt.len() {
                    self.complete(&mut chart, i, &item);
                } else {
                    self.process_symbol(&mut chart, i, chars, &item, &alt[item.dot]);
                }
                j += 1;
            }
        }
        chart
    }

    fn process_symbol(
        &self,
        chart: &mut [ChartSet],
        i: usize,
        chars: &[char],
        item: &EarleyItem,
        sym: &Symbol,
    ) {
        match sym {
            Symbol::Nonterminal { name, .. } => {
                self.predict(chart, i, name);
                // Handle nullable: if predicted nonterminal has completed at
                // this position (epsilon production) advance the dot immediately
                self.complete_nullable(chart, i, item, name);
            }
            Symbol::Literal {
                value, char_len, ..
            } => {
                scan_literal(chart, i, chars, item, value, *char_len);
            }
            Symbol::Inclusion { members, .. } => {
                scan_charset(chart, i, chars, item, members, false);
            }
            Symbol::Exclusion { members, .. } => {
                scan_charset(chart, i, chars, item, members, true);
            }
            Symbol::Insertion { .. } => {
                chart[i].add(EarleyItem {
                    dot: item.dot + 1,
                    ..*item
                });
            }
        }
    }

    fn find_parse(&self, ctx: &ExtractCtx<'_>, n: usize) -> Result<ParseTree, ParseError> {
        let root_idx = ctx
            .name_to_idx
            .get(&self.root)
            .copied()
            .unwrap_or(usize::MAX);
        let has_completed = ctx
            .completions
            .get(&(root_idx, 0))
            .is_some_and(|ends| ends.contains(&n));

        if !has_completed {
            let furthest = ctx
                .chart
                .iter()
                .enumerate()
                .rev()
                .find(|(_, set)| !set.is_empty())
                .map_or(0, |(i, _)| i);
            let expected = self.expected_at(ctx.chart, furthest);
            return Err(ParseError::Failed {
                pos: furthest,
                expected,
            });
        }

        self.build_tree(ctx, &self.root, 0, n)
            .ok_or_else(|| ParseError::Failed {
                pos: 0,
                expected: "valid parse".into(),
            })
    }

    fn predict(&self, chart: &mut [ChartSet], pos: usize, name: &str) {
        if let Some(indices) = self.rule_index.get(name) {
            for &rule_idx in indices {
                let rule = &self.rules[rule_idx];
                for alt_idx in 0..rule.alts.len() {
                    chart[pos].add(EarleyItem {
                        rule_idx,
                        alt_idx,
                        dot: 0,
                        start: pos,
                    });
                }
            }
        }
    }

    fn complete_nullable(&self, chart: &mut [ChartSet], pos: usize, item: &EarleyItem, name: &str) {
        // Check `name` already has completion at (pos, pos).
        let has_nullable_completion = chart[pos].iter().any(|it| {
            let r = &self.rules[it.rule_idx];
            r.name == name && it.start == pos && it.dot == r.alts[it.alt_idx].len()
        });
        if has_nullable_completion {
            chart[pos].add(EarleyItem {
                dot: item.dot + 1,
                ..*item
            });
        }
    }

    fn complete(&self, chart: &mut [ChartSet], pos: usize, completed: &EarleyItem) {
        let completed_name = &self.rules[completed.rule_idx].name;
        let start = completed.start;

        let mut to_add = Vec::new();
        for item in chart[start].iter() {
            let rule = &self.rules[item.rule_idx];
            let alt = &rule.alts[item.alt_idx];
            if item.dot < alt.len()
                && let Symbol::Nonterminal { name, .. } = &alt[item.dot]
                && name == completed_name
            {
                to_add.push(EarleyItem {
                    dot: item.dot + 1,
                    ..*item
                });
            }
        }
        for new_item in to_add {
            chart[pos].add(new_item);
        }
    }

    fn expected_at(&self, chart: &[ChartSet], pos: usize) -> String {
        let mut expected = Vec::new();
        for item in chart[pos].iter() {
            let rule = &self.rules[item.rule_idx];
            let alt = &rule.alts[item.alt_idx];
            if item.dot < alt.len() {
                let desc = match &alt[item.dot] {
                    Symbol::Nonterminal { name, .. } => name.clone(),
                    Symbol::Literal { value, .. } => {
                        format!("\"{value}\"")
                    }
                    Symbol::Inclusion { .. } => "[charset]".to_string(),
                    Symbol::Exclusion { .. } => "~[charset]".to_string(),
                    Symbol::Insertion { .. } => continue,
                };
                if !expected.contains(&desc) {
                    expected.push(desc);
                }
            }
        }
        if expected.is_empty() {
            "end of input".to_string()
        } else {
            expected.join(" or ")
        }
    }

    fn build_tree(
        &self,
        ctx: &ExtractCtx<'_>,
        name: &str,
        start: usize,
        end: usize,
    ) -> Option<ParseTree> {
        let name_idx = ctx.name_to_idx.get(name).copied().unwrap_or(usize::MAX);
        let key = (name_idx, start, end);
        if let Some(cached) = ctx.memo.borrow().get(&key) {
            return cached.clone();
        }

        let result = self.build_tree_inner(ctx, name, start, end);
        ctx.memo.borrow_mut().insert(key, result.clone());
        result
    }

    fn build_tree_inner(
        &self,
        ctx: &ExtractCtx<'_>,
        name: &str,
        start: usize,
        end: usize,
    ) -> Option<ParseTree> {
        if name.starts_with("__star_") || name.starts_with("__plus_") {
            return self.build_repeat_tree(ctx, name, start, end);
        }

        for item in ctx.chart[end].iter() {
            let rule = &self.rules[item.rule_idx];
            if rule.name != name || item.start != start {
                continue;
            }
            let alt = &rule.alts[item.alt_idx];
            if item.dot != alt.len() {
                continue;
            }

            let mut children = Vec::new();
            if self.match_symbols(ctx, alt, start, end, 0, &mut children) {
                return Some(ParseTree {
                    root: TreeNode::Element {
                        mark: rule.mark,
                        name: rule.output_name.clone(),
                        children,
                    },
                });
            }
        }
        None
    }

    /// Iterative extraction for `__star`/`__plus` rules.
    /// Walks forward greedily, longest match first.
    fn build_repeat_tree(
        &self,
        ctx: &ExtractCtx<'_>,
        name: &str,
        start: usize,
        end: usize,
    ) -> Option<ParseTree> {
        let rule_idx = self.rule_index.get(name)?.first().copied()?;
        let rule = &self.rules[rule_idx];

        if start == end {
            if name.starts_with("__plus_") {
                let base_alt = rule.alts.iter().find(|a| a.len() == 1);
                if let Some(base) = base_alt {
                    let mut children = Vec::new();
                    if self.match_symbols(ctx, base, start, end, 0, &mut children) {
                        return Some(ParseTree {
                            root: TreeNode::Element {
                                mark: rule.mark,
                                name: name.to_string(),
                                children,
                            },
                        });
                    }
                }
            }
            if rule.alts.iter().any(Vec::is_empty) {
                return Some(ParseTree {
                    root: TreeNode::Element {
                        mark: rule.mark,
                        name: name.to_string(),
                        children: vec![],
                    },
                });
            }
        }

        let rec_alt = rule.alts.iter().find(|a| {
            a.len() == 2
                && matches!(&a[1], Symbol::Nonterminal { name: n, .. } if n.as_str() == name)
        });

        let elem_sym = if let Some(alt) = rec_alt {
            &alt[0]
        } else {
            let base_alt = rule.alts.iter().find(|a| a.len() == 1)?;
            let mut children = Vec::new();
            if self.match_symbols(ctx, base_alt, start, end, 0, &mut children) {
                return Some(ParseTree {
                    root: TreeNode::Element {
                        mark: rule.mark,
                        name: name.to_string(),
                        children,
                    },
                });
            }
            return None;
        };

        let mut all_children = Vec::new();
        let mut pos = start;

        while pos < end {
            let splits = Self::completion_ends(ctx, elem_sym, pos);
            let mut found_split = false;
            for &split in splits.iter().rev() {
                if split <= pos || split > end {
                    continue;
                }
                let mut elem_children = Vec::new();
                if self.match_symbols(
                    ctx,
                    std::slice::from_ref(elem_sym),
                    pos,
                    split,
                    0,
                    &mut elem_children,
                ) {
                    all_children.extend(elem_children);
                    pos = split;
                    found_split = true;
                    break;
                }
            }
            if !found_split {
                return None;
            }
        }

        Some(ParseTree {
            root: TreeNode::Element {
                mark: rule.mark,
                name: name.to_string(),
                children: all_children,
            },
        })
    }

    /// End positions where symbol completes from `start`.
    fn completion_ends(ctx: &ExtractCtx<'_>, sym: &Symbol, start: usize) -> Vec<usize> {
        match sym {
            Symbol::Nonterminal { name, .. } => {
                let idx = ctx.name_to_idx.get(name).copied().unwrap_or(usize::MAX);
                ctx.completions
                    .get(&(idx, start))
                    .cloned()
                    .unwrap_or_default()
            }
            Symbol::Literal { char_len, .. } => vec![start + char_len],
            Symbol::Inclusion { .. } | Symbol::Exclusion { .. } => vec![start + 1],
            Symbol::Insertion { .. } => vec![start],
        }
    }

    fn match_symbols(
        &self,
        ctx: &ExtractCtx<'_>,
        alt: &[Symbol],
        pos: usize,
        end: usize,
        sym_idx: usize,
        children: &mut Vec<TreeNode>,
    ) -> bool {
        if sym_idx == alt.len() {
            return pos == end;
        }
        match &alt[sym_idx] {
            Symbol::Insertion { value } => {
                children.push(TreeNode::Insertion {
                    value: value.clone(),
                });
                self.match_symbols(ctx, alt, pos, end, sym_idx + 1, children)
            }
            Symbol::Literal {
                mark,
                value,
                char_len,
            } => {
                let next = pos + char_len;
                if next > end {
                    return false;
                }
                for (k, lc) in value.chars().enumerate() {
                    if ctx.chars[pos + k] != lc {
                        return false;
                    }
                }
                children.push(TreeNode::Text {
                    mark: *mark,
                    value: value.clone(),
                });
                self.match_symbols(ctx, alt, next, end, sym_idx + 1, children)
            }
            Symbol::Inclusion { mark, members } => {
                self.match_charset(ctx, alt, pos, end, sym_idx, children, *mark, members, false)
            }
            Symbol::Exclusion { mark, members } => {
                self.match_charset(ctx, alt, pos, end, sym_idx, children, *mark, members, true)
            }
            Symbol::Nonterminal {
                mark,
                name,
                output_name,
            } => self.match_nonterminal(
                ctx,
                alt,
                pos,
                end,
                sym_idx,
                children,
                *mark,
                name,
                output_name.as_deref(),
            ),
        }
    }

    #[expect(clippy::too_many_arguments)]
    fn match_charset(
        &self,
        ctx: &ExtractCtx<'_>,
        alt: &[Symbol],
        pos: usize,
        end: usize,
        sym_idx: usize,
        children: &mut Vec<TreeNode>,
        mark: Mark,
        members: &[CharMember],
        is_exclusion: bool,
    ) -> bool {
        if pos >= end {
            return false;
        }
        let ch = ctx.chars[pos];
        let matched = if is_exclusion {
            matches_exclusion(ch, members)
        } else {
            matches_inclusion(ch, members)
        };
        if !matched {
            return false;
        }
        children.push(TreeNode::Text {
            mark,
            value: ch.to_string(),
        });
        self.match_symbols(ctx, alt, pos + 1, end, sym_idx + 1, children)
    }

    #[expect(clippy::too_many_arguments)]
    fn match_nonterminal(
        &self,
        ctx: &ExtractCtx<'_>,
        alt: &[Symbol],
        pos: usize,
        end: usize,
        sym_idx: usize,
        children: &mut Vec<TreeNode>,
        mark: Mark,
        name: &str,
        output_name: Option<&str>,
    ) -> bool {
        let name_idx = ctx.name_to_idx.get(name).copied().unwrap_or(usize::MAX);
        let ends = ctx
            .completions
            .get(&(name_idx, pos))
            .cloned()
            .unwrap_or_default();

        for split in ends {
            if split > end {
                continue;
            }
            if let Some(subtree) = self.build_tree(ctx, name, pos, split) {
                let mut child_node = apply_mark(&subtree.root, mark);
                if let Some(alias) = output_name {
                    apply_rename(&mut child_node, alias);
                }
                let prev_len = children.len();
                children.push(child_node);
                if self.match_symbols(ctx, alt, split, end, sym_idx + 1, children) {
                    return true;
                }
                children.truncate(prev_len);
            }
        }
        false
    }
}

fn apply_rename(node: &mut TreeNode, alias: &str) {
    if let TreeNode::Element { name, .. } = node {
        *name = alias.to_string();
    }
}

fn apply_mark(node: &TreeNode, use_mark: Mark) -> TreeNode {
    match node {
        TreeNode::Element {
            mark: rule_mark,
            name,
            children,
        } => {
            let effective = if use_mark == Mark::None {
                *rule_mark
            } else {
                use_mark
            };
            TreeNode::Element {
                mark: effective,
                name: name.clone(),
                children: children.clone(),
            }
        }
        other => other.clone(),
    }
}

fn scan_literal(
    chart: &mut [ChartSet],
    pos: usize,
    chars: &[char],
    item: &EarleyItem,
    value: &str,
    char_len: usize,
) {
    let next_pos = pos + char_len;
    if next_pos > chars.len() {
        return;
    }
    for (k, lc) in value.chars().enumerate() {
        if chars[pos + k] != lc {
            return;
        }
    }
    chart[next_pos].add(EarleyItem {
        dot: item.dot + 1,
        ..*item
    });
}

fn scan_charset(
    chart: &mut [ChartSet],
    pos: usize,
    chars: &[char],
    item: &EarleyItem,
    members: &[CharMember],
    is_exclusion: bool,
) {
    if pos >= chars.len() {
        return;
    }
    let ch = chars[pos];
    let matched = if is_exclusion {
        matches_exclusion(ch, members)
    } else {
        matches_inclusion(ch, members)
    };
    if matched {
        chart[pos + 1].add(EarleyItem {
            dot: item.dot + 1,
            ..*item
        });
    }
}

/// Desugars `+`/`*`/`?` into synthetic BNF rules.
struct Normalizer {
    rules: Vec<NormRule>,
    counter: usize,
}

impl Normalizer {
    fn new() -> Self {
        Self {
            rules: Vec::new(),
            counter: 0,
        }
    }

    fn fresh_name(&mut self, base: &str) -> String {
        self.counter += 1;
        format!("__{base}_{}", self.counter)
    }

    fn normalize_rule(&mut self, rule: &Rule) {
        let norm_alts: Vec<Vec<Symbol>> = rule
            .alts
            .iter()
            .map(|alt| self.normalize_alt(alt))
            .collect();
        self.rules.push(NormRule {
            mark: rule.mark,
            output_name: rule.alias.clone().unwrap_or_else(|| rule.name.clone()),
            name: rule.name.clone(),
            alts: norm_alts,
        });
    }

    fn normalize_alt(&mut self, alt: &Alt) -> Vec<Symbol> {
        let mut syms = Vec::new();
        for term in alt {
            match term {
                Term::Nonterminal { name, .. } if name.starts_with("__repeat0_") => {
                    if let Some(prev) = syms.pop() {
                        let rep_name = self.make_repeat0(prev);
                        syms.push(Symbol::Nonterminal {
                            mark: Mark::Hidden,
                            name: rep_name,
                            output_name: None,
                        });
                    }
                }
                Term::Nonterminal { name, .. } if name.starts_with("__repeat1_") => {
                    if let Some(prev) = syms.pop() {
                        let rep_name = self.make_repeat1(prev);
                        syms.push(Symbol::Nonterminal {
                            mark: Mark::Hidden,
                            name: rep_name,
                            output_name: None,
                        });
                    }
                }
                Term::Nonterminal { name, .. } if name.starts_with("__option_") => {
                    if let Some(prev) = syms.pop() {
                        let opt_name = self.make_option(prev);
                        syms.push(Symbol::Nonterminal {
                            mark: Mark::Hidden,
                            name: opt_name,
                            output_name: None,
                        });
                    }
                }
                _ => {
                    syms.push(normalize_term(term));
                }
            }
        }
        syms
    }

    /// `f*` => `__star_N: | f, __star_N .`
    fn make_repeat0(&mut self, sym: Symbol) -> String {
        let name = self.fresh_name("star");
        self.rules.push(NormRule {
            mark: Mark::Hidden,
            output_name: name.clone(),
            name: name.clone(),
            alts: vec![
                vec![],
                vec![
                    sym,
                    Symbol::Nonterminal {
                        mark: Mark::Hidden,
                        name: name.clone(),
                        output_name: None,
                    },
                ],
            ],
        });
        name
    }

    fn make_repeat1(&mut self, sym: Symbol) -> String {
        let name = self.fresh_name("plus");
        self.rules.push(NormRule {
            mark: Mark::Hidden,
            output_name: name.clone(),
            name: name.clone(),
            alts: vec![
                vec![sym.clone()],
                vec![
                    sym,
                    Symbol::Nonterminal {
                        mark: Mark::Hidden,
                        name: name.clone(),
                        output_name: None,
                    },
                ],
            ],
        });
        name
    }

    fn make_option(&mut self, sym: Symbol) -> String {
        let name = self.fresh_name("opt");
        self.rules.push(NormRule {
            mark: Mark::Hidden,
            output_name: name.clone(),
            name: name.clone(),
            alts: vec![vec![], vec![sym]],
        });
        name
    }
}

fn normalize_term(term: &Term) -> Symbol {
    match term {
        Term::Nonterminal { mark, name, alias } => Symbol::Nonterminal {
            mark: *mark,
            name: name.clone(),
            output_name: alias.clone(),
        },
        Term::Literal { mark, value } => {
            let char_len = value.chars().count();
            Symbol::Literal {
                mark: *mark,
                value: value.clone(),
                char_len,
            }
        }
        Term::Inclusion { mark, members } => Symbol::Inclusion {
            mark: *mark,
            members: members.clone(),
        },
        Term::Exclusion { mark, members } => Symbol::Exclusion {
            mark: *mark,
            members: members.clone(),
        },
        Term::Insertion { value } => Symbol::Insertion {
            value: value.clone(),
        },
    }
}
