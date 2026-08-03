use crate::ast::CharMember;

pub fn matches_inclusion(ch: char, members: &[CharMember]) -> bool {
    members.iter().any(|m| m.matches(ch))
}

pub fn matches_exclusion(ch: char, members: &[CharMember]) -> bool {
    !members.iter().any(|m| m.matches(ch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inclusion_range() {
        let members = vec![CharMember::Range('a', 'z')];
        assert!(matches_inclusion('m', &members));
        assert!(!matches_inclusion('A', &members));
    }

    #[test]
    fn exclusion_chars() {
        let members = vec![CharMember::Chars("{}".into())];
        assert!(matches_exclusion('a', &members));
        assert!(!matches_exclusion('{', &members));
    }

    #[test]
    fn class_match() {
        let members = vec![CharMember::Class("Lu".into())];
        assert!(matches_inclusion('A', &members));
        assert!(!matches_inclusion('a', &members));
    }
}
