use full_moon::ast::Ast;

use crate::{nodes::Block, utils::Timer, ParserError};

use super::ast_converter::{AstConverter, ConvertError};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FullmoonParser {
    hold_token_data: bool,
}

impl FullmoonParser {
    pub fn parse(&self, code: &str) -> Result<Block, ParserError> {
        let full_moon_parse_timer = Timer::now();
        let parse_result = full_moon::parse(code);
        log::trace!(
            "full-moon parsing done in {}",
            full_moon_parse_timer.duration_label()
        );
        parse_result.map_err(ParserError::parsing).and_then(|ast| {
            log::trace!("start converting full-moon AST");
            let conversion_timer = Timer::now();
            let block = self.convert_ast(ast).map_err(ParserError::converting);
            log::trace!(
                " ⨽ completed AST conversion in {}",
                conversion_timer.duration_label()
            );
            block
        })
    }

    pub fn set_preserve_tokens(&mut self) {
        self.hold_token_data = true;
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
    fn convert_ast(&self, ast: Ast) -> Result<Block, ConvertError> {
        AstConverter::new(self.hold_token_data).convert(&ast)
    }
}
