//! ixml grammar AST.
use std::fmt;

/// Serialization mark: controls output shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    None,
    /// `^`
    Element,
    /// `@`
    Attribute,
    /// `-`
    Hidden,
}

#[derive(Debug, Clone)]
pub struct Grammar {
    pub rules: Vec<Rule>,
}

impl Grammar {
    #[must_use]
    pub fn root_name(&self) -> &str {
        self.rules.first().map_or("", |r| r.name.as_str())
    }

    #[must_use]
    pub fn find_rule(&self, name: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.name == name)
    }
}

/// `name: alt1 ; alt2 .` or `name>alias: alt1 ; alt2 .`
#[derive(Debug, Clone)]
pub struct Rule {
    pub mark: Mark,
    pub name: String,
    pub alias: Option<String>,
    pub alts: Vec<Alt>,
}

pub type Alt = Vec<Term>;

#[derive(Debug, Clone)]
pub enum Term {
    Nonterminal {
        mark: Mark,
        name: String,
        alias: Option<String>,
    },
    Literal {
        mark: Mark,
        value: String,
    },
    Inclusion {
        mark: Mark,
        members: Vec<CharMember>,
    },
    Exclusion {
        mark: Mark,
        members: Vec<CharMember>,
    },
    /// Appears in output only, not matched in input.
    Insertion {
        value: String,
    },
}

#[derive(Debug, Clone)]
pub enum CharMember {
    /// `"abc"` -- matches a, b, or c.
    Chars(String),
    /// `"a"-"z"`
    Range(char, char),
    /// Unicode category, e.g. `Ll`, `L`.
    Class(String),
}

impl CharMember {
    #[must_use]
    pub fn matches(&self, ch: char) -> bool {
        match self {
            Self::Chars(s) => s.contains(ch),
            Self::Range(lo, hi) => ch >= *lo && ch <= *hi,
            Self::Class(code) => char_in_category(ch, code),
        }
    }
}

fn char_in_category(ch: char, code: &str) -> bool {
    use unicode_general_category::{GeneralCategory, get_general_category};
    let cat = get_general_category(ch);
    match code {
        "L" => matches!(
            cat,
            GeneralCategory::UppercaseLetter
                | GeneralCategory::LowercaseLetter
                | GeneralCategory::TitlecaseLetter
                | GeneralCategory::ModifierLetter
                | GeneralCategory::OtherLetter
        ),
        "M" => matches!(
            cat,
            GeneralCategory::NonspacingMark
                | GeneralCategory::SpacingMark
                | GeneralCategory::EnclosingMark
        ),
        "N" => matches!(
            cat,
            GeneralCategory::DecimalNumber
                | GeneralCategory::LetterNumber
                | GeneralCategory::OtherNumber
        ),
        "P" => matches!(
            cat,
            GeneralCategory::ConnectorPunctuation
                | GeneralCategory::DashPunctuation
                | GeneralCategory::OpenPunctuation
                | GeneralCategory::ClosePunctuation
                | GeneralCategory::InitialPunctuation
                | GeneralCategory::FinalPunctuation
                | GeneralCategory::OtherPunctuation
        ),
        "Z" => matches!(
            cat,
            GeneralCategory::SpaceSeparator
                | GeneralCategory::LineSeparator
                | GeneralCategory::ParagraphSeparator
        ),
        "S" => matches!(
            cat,
            GeneralCategory::MathSymbol
                | GeneralCategory::CurrencySymbol
                | GeneralCategory::ModifierSymbol
                | GeneralCategory::OtherSymbol
        ),
        "C" => matches!(
            cat,
            GeneralCategory::Control
                | GeneralCategory::Format
                | GeneralCategory::Surrogate
                | GeneralCategory::PrivateUse
                | GeneralCategory::Unassigned
        ),
        "LC" => matches!(
            cat,
            GeneralCategory::UppercaseLetter
                | GeneralCategory::LowercaseLetter
                | GeneralCategory::TitlecaseLetter
        ),
        "Lu" => cat == GeneralCategory::UppercaseLetter,
        "Ll" => cat == GeneralCategory::LowercaseLetter,
        "Lt" => cat == GeneralCategory::TitlecaseLetter,
        "Lm" => cat == GeneralCategory::ModifierLetter,
        "Lo" => cat == GeneralCategory::OtherLetter,
        "Mn" => cat == GeneralCategory::NonspacingMark,
        "Mc" => cat == GeneralCategory::SpacingMark,
        "Me" => cat == GeneralCategory::EnclosingMark,
        "Nd" => cat == GeneralCategory::DecimalNumber,
        "Nl" => cat == GeneralCategory::LetterNumber,
        "No" => cat == GeneralCategory::OtherNumber,
        "Pc" => cat == GeneralCategory::ConnectorPunctuation,
        "Pd" => cat == GeneralCategory::DashPunctuation,
        "Ps" => cat == GeneralCategory::OpenPunctuation,
        "Pe" => cat == GeneralCategory::ClosePunctuation,
        "Pi" => cat == GeneralCategory::InitialPunctuation,
        "Pf" => cat == GeneralCategory::FinalPunctuation,
        "Po" => cat == GeneralCategory::OtherPunctuation,
        "Zs" => cat == GeneralCategory::SpaceSeparator,
        "Zl" => cat == GeneralCategory::LineSeparator,
        "Zp" => cat == GeneralCategory::ParagraphSeparator,
        "Sm" => cat == GeneralCategory::MathSymbol,
        "Sc" => cat == GeneralCategory::CurrencySymbol,
        "Sk" => cat == GeneralCategory::ModifierSymbol,
        "So" => cat == GeneralCategory::OtherSymbol,
        "Cc" => cat == GeneralCategory::Control,
        "Cf" => cat == GeneralCategory::Format,
        "Cs" => cat == GeneralCategory::Surrogate,
        "Co" => cat == GeneralCategory::PrivateUse,
        "Cn" => cat == GeneralCategory::Unassigned,
        _ => false,
    }
}

impl fmt::Display for Mark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None | Self::Element => Ok(()),
            Self::Attribute => write!(f, "@"),
            Self::Hidden => write!(f, "-"),
        }
    }
}
