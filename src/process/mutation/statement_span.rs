use std::fmt;

use crate::process::path::{NodePath, NodePathBuf, NodePathSlice};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatementSpan {
    path: NodePathBuf,
    length: usize,
}

impl StatementSpan {
    #[inline]
    pub fn path(&self) -> &NodePathBuf {
        &self.path
    }

    /// get a path to the last statement of the span
    pub fn last_path(&self) -> NodePathBuf {
        let (parent, index) = self.split_statement();
        parent.join_statement(index + self.length.saturating_sub(1))
    }

    /// get a path to the next statement after the span
    pub fn next_path(&self) -> NodePathBuf {
        let (parent, index) = self.split_statement();
        parent.join_statement(index + self.length)
    }

    fn split_statement(&self) -> (&NodePathSlice, usize) {
        self.path
            .parent()
            .zip(self.path.last_statement())
            .expect("statement span should be made of a statement path")
    }

    #[inline]
    pub fn set_path(&mut self, new_path: impl Into<NodePathBuf>) {
        self.path = new_path.into();
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.length
    }

    #[inline]
    pub fn resize(&mut self, new_length: usize) {
        self.length = new_length;
    }

    pub fn contains(&self, path: &NodePathSlice) -> bool {
        if !self.path.is_statement_path() {
            return false;
        }

        let statement_ancestor = if let Some(ancestor) = path.find_first_statement_ancestor() {
            ancestor
        } else {
            return false;
        };

        if self.path.parent() != statement_ancestor.parent() {
            return false;
        }

        let span_index = self
            .path
            .last_statement()
            .expect("statement span should end with a statement component");

        let path_index = statement_ancestor
            .last_statement()
            .expect("path should end with a statement component");

        path_index >= span_index && path_index < span_index.saturating_add(self.length)
    }

    pub fn contains_span(&self, span: &Self) -> bool {
        if !self.path.is_statement_path() {
            return false;
        }

        let statement_ancestor = if let Some(ancestor) = span.path().find_first_statement_ancestor()
        {
            ancestor
        } else {
            return false;
        };

        if self.path.parent() != statement_ancestor.parent() {
            return false;
        }

        let span_index = self
            .path
            .last_statement()
            .expect("statement span should end with a statement component");

        let path_index = statement_ancestor
            .last_statement()
            .expect("path should end with a statement component");

        let span_index_end = span_index.saturating_add(self.length);

        path_index >= span_index
            && path_index < span_index_end
            && path_index.saturating_add(span.len()) <= span_index_end
    }
}

impl NodePathBuf {
    pub fn span(self, length: usize) -> StatementSpan {
        debug_assert!(length > 0, "StatementSpan length must always be above zero");
        StatementSpan { path: self, length }
    }
}

impl From<NodePathBuf> for StatementSpan {
    fn from(path: NodePathBuf) -> Self {
        Self { path, length: 1 }
    }
}

impl fmt::Display for StatementSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("[{}={}]", self.path, self.length))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn new() -> NodePathBuf {
        NodePathBuf::default()
    }

    mod contains {
        use super::*;

        macro_rules! test_contains {
            ($($name:ident ( $span:expr, $path:expr )),* $(,)?) => {
                $(
                    #[test]
                    fn $name() {
                        let span = StatementSpan::from($span);
                        assert!(span.contains(&$path));
                    }
                )*
            };
        }

        macro_rules! test_not_contains {
            ($($name:ident ( $span:expr, $path:expr )),* $(,)?) => {
                $(
                    #[test]
                    fn $name() {
                        let span = StatementSpan::from($span);
                        assert!(!span.contains(&$path));
                    }
                )*
            };
        }

        test_contains!(
            first_statement(new().with_statement(0).span(1), new().with_statement(0)),
            second_statement(new().with_statement(1).span(1), new().with_statement(1)),
            two_first_statement_contains_first(
                new().with_statement(0).span(2),
                new().with_statement(0)
            ),
            two_first_statement_contains_second(
                new().with_statement(0).span(2),
                new().with_statement(1)
            ),
            first_statement_contains_nested_expression(
                new().with_statement(0).span(1),
                new().with_statement(0).with_expression(1)
            ),
        );

        test_not_contains!(
            first_statement_does_not_contain_second(
                new().with_statement(0).span(1),
                new().with_statement(1)
            ),
            second_statement_does_not_contain_first_statement(
                new().with_statement(1).span(1),
                new().with_statement(0)
            ),
            nested_first_statement_does_not_contain_second(
                new().with_statement(2).with_statement(0).span(1),
                new().with_statement(2).with_statement(1)
            ),
        );
    }

    mod contains_span {
        use super::*;

        macro_rules! test_contains {
            ($($name:ident ( $span:expr, $other:expr )),* $(,)?) => {
                $(
                    #[test]
                    fn $name() {
                        let span = StatementSpan::from($span);
                        let other_span = StatementSpan::from($other);
                        assert!(
                            span.contains_span(&other_span),
                            "`{}` should not contain `{}`",
                            span,
                            other_span,
                        );
                    }
                )*
            };
        }

        macro_rules! test_not_contains {
            ($($name:ident ( $span:expr, $other:expr )),* $(,)?) => {
                $(
                    #[test]
                    fn $name() {
                        let span = StatementSpan::from($span);
                        let other_span = StatementSpan::from($other);
                        assert!(
                            !span.contains_span(&other_span),
                            "`{}` should not contain `{}`",
                            span,
                            other_span,
                        );
                    }
                )*
            };
        }

        test_contains!(
            first_statement(new().with_statement(0), new().with_statement(0)),
            second_statement(new().with_statement(1), new().with_statement(1)),
            two_first_statement_contains_first(
                new().with_statement(0).span(2),
                new().with_statement(0)
            ),
            two_first_statement_contains_second(
                new().with_statement(0).span(2),
                new().with_statement(1)
            ),
            two_first_statement_contains_two_same_statements(
                new().with_statement(0).span(2),
                new().with_statement(0).span(2)
            ),
            first_statement_contains_nested_expression(
                new().with_statement(0).span(1),
                new().with_statement(0).with_expression(1)
            ),
        );

        test_not_contains!(
            first_statement_does_not_contain_second(
                new().with_statement(0).span(1),
                new().with_statement(1)
            ),
            second_statement_does_not_contain_first_statement(
                new().with_statement(1).span(1),
                new().with_statement(0)
            ),
            first_statement_does_not_contain_two_first_statements(
                new().with_statement(0).span(1),
                new().with_statement(0).span(2)
            ),
            nested_first_statement_does_not_contain_second(
                new().with_statement(2).with_statement(0).span(1),
                new().with_statement(2).with_statement(1)
            ),
        );
    }
}
