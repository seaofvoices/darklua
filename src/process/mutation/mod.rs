mod effect;
mod insertion_content;
mod mutation;
mod resolver;
mod statement_insertion;
mod statement_replacement;
mod statement_span;

pub use effect::MutationEffect;
pub use insertion_content::StatementInsertionContent;
pub use mutation::Mutation;
pub use resolver::MutationResolver;
pub use statement_insertion::StatementInsertion;
pub use statement_replacement::StatementReplacement;
pub use statement_span::StatementSpan;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MutationError {}

pub type MutationResult = Result<Vec<MutationEffect>, MutationError>;

#[cfg(test)]
pub mod test {
    macro_rules! test_mutation {
        ( $($name:ident ( $mutation:expr, $code:literal ) => $expect_code:literal => [ $($expect_effect:expr),* $(,)? ] ),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let parser = $crate::Parser::default();
                    let mut block = parser
                        .parse($code)
                        .expect("given test code should parse");

                    let expected_block = parser
                        .parse($expect_code)
                        .expect("given test code should parse");

                    let effect = $mutation
                        .apply(&mut block)
                        .expect("mutation should succeed");

                    let expect_effects = vec![$( $expect_effect, )*];

                    use $crate::generator::{LuaGenerator, ReadableLuaGenerator};

                    let mut generator = ReadableLuaGenerator::new(80);
                    generator.write_block(&block);

                    pretty_assertions::assert_eq!(effect, expect_effects);
                    pretty_assertions::assert_eq!(
                        block,
                        expected_block,
                        "\nExpected: `{}`\nReceived: `{}`",
                        $expect_code,
                        generator.into_string(),
                    );
                }
            )*
        };
    }

    pub(crate) use test_mutation;
}
