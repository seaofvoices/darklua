use std::str::CharIndices;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringExpression {
    value: String,
}

impl StringExpression {
    pub fn new(string: String) -> Option<Self> {
        let mut chars = string.chars();
        let value = match chars.next() {
            Some('"') | Some('\'') => string.get(1..string.len() - 1).map(str::to_owned),
            Some('[') => chars
                .enumerate()
                .find_map(|(indice, character)| if character == '[' { Some(indice) } else { None })
                .and_then(|indice| {
                    let length = 2 + indice;
                    string.get(length..string.len() - length).map(str::to_owned)
                }),
            _ => None,
        };

        value.map(|value| Self { value })
    }

    pub fn empty() -> Self {
        Self {
            value: "".to_owned(),
        }
    }

    pub fn from_value<T: Into<String>>(value: T) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

    pub fn is_multiline(&self) -> bool {
        self.value.find("\n").is_some()
    }

    pub fn has_single_quote(&self) -> bool {
        self.find_not_escaped('\'').is_some()
    }

    pub fn has_double_quote(&self) -> bool {
        self.find_not_escaped('"').is_some()
    }

    fn find_not_escaped(&self, pattern: char) -> Option<usize> {
        self.find_not_escaped_from(pattern, &mut self.value.char_indices())
    }

    fn find_not_escaped_from(&self, pattern: char, chars: &mut CharIndices) -> Option<usize> {
        let mut escaped = false;
        chars.find_map(|(index, character)| {
            if escaped {
                escaped = false;
                None
            } else {
                match character {
                    '\\' => {
                        escaped = true;
                        None
                    }
                    value => {
                        if value == pattern {
                            Some(index)
                        } else {
                            None
                        }
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_removes_double_quotes() {
        let string = StringExpression::new(r#""hello""#.to_owned()).unwrap();

        assert_eq!(string.get_value(), "hello");
    }

    #[test]
    fn new_removes_single_quotes() {
        let string = StringExpression::new("'hello'".to_owned()).unwrap();

        assert_eq!(string.get_value(), "hello");
    }

    #[test]
    fn new_removes_double_brackets() {
        let string = StringExpression::new("[[hello]]".to_owned()).unwrap();

        assert_eq!(string.get_value(), "hello");
    }

    #[test]
    fn new_removes_double_brackets_with_one_equals() {
        let string = StringExpression::new("[=[hello]=]".to_owned()).unwrap();

        assert_eq!(string.get_value(), "hello");
    }

    #[test]
    fn new_removes_double_brackets_with_multiple_equals() {
        let string = StringExpression::new("[==[hello]==]".to_owned()).unwrap();

        assert_eq!(string.get_value(), "hello");
    }

    #[test]
    fn has_single_quote_is_false_if_no_single_quotes() {
        let string = StringExpression::from_value("hello");

        assert_eq!(string.has_single_quote(), false);
    }

    #[test]
    fn has_single_quote_is_true_if_unescaped_single_quotes() {
        let string = StringExpression::from_value("don't");

        assert_eq!(string.has_single_quote(), true);
    }

    #[test]
    fn has_single_quote_is_true_if_unescaped_single_quotes_with_escaped_backslash() {
        let string = StringExpression::from_value(r#"don\\'t"#);

        assert_eq!(string.has_single_quote(), true);
    }

    #[test]
    fn has_single_quote_is_false_if_escaped_single_quotes() {
        let string = StringExpression::from_value(r#"don\'t"#);

        assert_eq!(string.has_single_quote(), false);
    }

    #[test]
    fn has_double_quote_is_false_if_no_double_quotes() {
        let string = StringExpression::from_value("hello");

        assert_eq!(string.has_double_quote(), false);
    }

    #[test]
    fn has_double_quote_is_true_if_unescaped_double_quotes() {
        let string = StringExpression::from_value(r#"Say: "Hi!""#);

        assert_eq!(string.has_double_quote(), true);
    }

    #[test]
    fn has_double_quote_is_false_if_escaped_double_quotes() {
        let string = StringExpression::from_value(r#"hel\"o"#);

        assert_eq!(string.has_double_quote(), false);
    }
}
