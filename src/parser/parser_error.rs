use std::fmt;

use super::{ast_converter::ConvertError, pest_parser};

#[derive(Clone, Debug)]
enum ParserErrorKind {
    Parsing(Vec<full_moon::Error>),
    PestParsing(pest::error::Error<pest_parser::Rule>),
    Converting(ConvertError),
}

#[derive(Clone, Debug)]
pub struct ParserError {
    kind: Box<ParserErrorKind>,
}

impl ParserError {
    pub(crate) fn parsing(err: Vec<full_moon::Error>) -> Self {
        Self {
            kind: ParserErrorKind::Parsing(err).into(),
        }
    }

    pub(crate) fn parsing2(err: pest::error::Error<pest_parser::Rule>) -> Self {
        Self {
            kind: ParserErrorKind::PestParsing(err).into(),
        }
    }

    pub(crate) fn converting(err: ConvertError) -> Self {
        Self {
            kind: ParserErrorKind::Converting(err).into(),
        }
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self.kind {
            ParserErrorKind::Parsing(errors) => {
                for err in errors {
                    writeln!(f, "{}", err)?;
                }
                Ok(())
            }
            ParserErrorKind::PestParsing(err) => write!(f, "{}", err),
            ParserErrorKind::Converting(err) => write!(f, "{}", err),
        }
    }
}
