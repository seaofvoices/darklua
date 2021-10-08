#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Position {
    Reference { start: usize, end: usize },
    LineNumber { content: String, line_number: usize },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TriviaKind {
    Comment,
    Whitespace,
}

impl TriviaKind {
    pub fn at(self, start: usize, end: usize) -> Trivia {
        Trivia {
            position: Position::Reference { start, end },
            kind: self,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trivia {
    position: Position,
    kind: TriviaKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    position: Position,
    leading_trivia: Vec<Trivia>,
    trailing_trivia: Vec<Trivia>,
}

impl Token {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            position: Position::Reference { start, end },
            leading_trivia: Vec::new(),
            trailing_trivia: Vec::new(),
        }
    }

    pub fn with_leading_trivia(mut self, trivia: Trivia) -> Self {
        self.leading_trivia.push(trivia);
        self
    }

    pub fn with_trailing_trivia(mut self, trivia: Trivia) -> Self {
        self.trailing_trivia.push(trivia);
        self
    }

    #[inline]
    pub fn push_leading_trivia(&mut self, trivia: Trivia) {
        self.leading_trivia.push(trivia);
    }

    #[inline]
    pub fn push_trailing_trivia(&mut self, trivia: Trivia) {
        self.trailing_trivia.push(trivia);
    }
}

#[cfg(test)]
mod test {
    // use super::*;

    #[test]
    fn do_test() {
        // struct A {
        //     start: usize,
        //     end: usize,
        // }
        // struct B {
        //     content: String,
        //     // line_number: Option<usize>,
        //     line_number: usize,
        // }
        // assert_eq!(4, std::mem::size_of::<Position>());
        // assert_eq!(std::mem::size_of::<A>(), std::mem::size_of::<B>());
    }
}
