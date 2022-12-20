mod permutator;

pub(crate) use permutator::Permutator;

pub(crate) type CharPermutator = Permutator<std::str::Chars<'static>>;

pub(crate) fn identifier_permutator() -> CharPermutator {
    Permutator::new("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_0123456789".chars())
}

pub(crate) const KEYWORDS: [&str; 21] = [
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "if", "in", "local",
    "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
];

pub(crate) mod keywords {
    macro_rules! matches_any {
        () => {
            "and"
                | "break"
                | "do"
                | "else"
                | "elseif"
                | "end"
                | "false"
                | "for"
                | "function"
                | "if"
                | "in"
                | "local"
                | "nil"
                | "not"
                | "or"
                | "repeat"
                | "return"
                | "then"
                | "true"
                | "until"
                | "while"
        };
    }

    pub(crate) use matches_any;
}

pub(crate) fn is_valid_identifier(identifier: &str) -> bool {
    !identifier.is_empty()
        && identifier.is_ascii()
        && identifier
            .char_indices()
            .all(|(i, c)| c.is_alphabetic() || c == '_' || (c.is_ascii_digit() && i > 0))
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn is_valid_identifier_is_true() {
        assert!(is_valid_identifier("hello"));
        assert!(is_valid_identifier("foo"));
        assert!(is_valid_identifier("bar"));
    }

    #[test]
    fn is_valid_identifier_is_false() {
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("$hello"));
        assert!(!is_valid_identifier(" "));
        assert!(!is_valid_identifier("5"));
        assert!(!is_valid_identifier("1bar"));
    }
}
