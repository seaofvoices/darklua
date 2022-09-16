use std::collections::VecDeque;

use crate::nodes::Block;

use super::{Mutation, MutationError};

#[derive(Clone, Debug, Default)]
pub struct MutationResolver {
    mutations: VecDeque<Mutation>,
}

impl MutationResolver {
    pub fn add(&mut self, mutation: impl Into<Mutation>) {
        self.mutations.push_back(mutation.into());
    }

    pub fn resolve(mut self, block: &mut Block) -> Result<(), MutationError> {
        while let Some(mutation) = self.mutations.pop_front() {
            let effects = mutation.apply(block)?;
            if !effects.is_empty() {
                for effect in effects {
                    let mut remove_indexes = Vec::new();

                    for (index, next_mutation) in self.mutations.iter_mut().enumerate() {
                        if !next_mutation.mutate(&effect) {
                            remove_indexes.push(index);
                        }
                    }

                    for index in remove_indexes.into_iter().rev() {
                        self.mutations.remove(index);
                    }
                }
            }
        }

        Ok(())
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.mutations.is_empty()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::process::{
        mutation::{StatementInsertion, StatementInsertionContent, StatementReplacement},
        path::NodePathBuf,
    };

    macro_rules! test_resolver {
        ($($name:ident ( $code:literal ) => [ $($mutation:expr),* $(,)? ] => $expect_code:literal),* $(,)?) => {
            $(
                #[test]
                fn $name () {
                    let parser = $crate::Parser::default();
                    let mut block = parser
                        .parse($code)
                        .expect("given test code should parse");

                    let expected_block = parser
                        .parse($expect_code)
                        .expect("given test code should parse");

                    let mut resolver = MutationResolver::default();

                    let mutations: Vec<Mutation> = vec![$( $mutation.into(), )*];

                    for mutation in mutations {
                        resolver.add(mutation)
                    }

                    resolver.resolve(&mut block).expect("mutations should succeed");

                    use $crate::generator::{LuaGenerator, ReadableLuaGenerator};

                    let mut generator = ReadableLuaGenerator::new(80);
                    generator.write_block(&block);

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

    fn statement_path(index: usize) -> NodePathBuf {
        NodePathBuf::default().with_statement(index)
    }

    fn parse_insertion(code: &str) -> StatementInsertionContent {
        let block = crate::Parser::default()
            .parse(code)
            .expect("given insertion code should parse");
        block.into()
    }

    test_resolver!(
        // removals
        remove_all_two("local a local b") => [
            StatementReplacement::remove(statement_path(0)),
            StatementReplacement::remove(statement_path(1)),
        ] => "",
        remove_all_two_in_reverse("local a local b") => [
            StatementReplacement::remove(statement_path(1)),
            StatementReplacement::remove(statement_path(0)),
        ] => "",
        remove_all_three("local a local b local c") => [
            StatementReplacement::remove(statement_path(0).span(2)),
            StatementReplacement::remove(statement_path(2)),
        ] => "",
        remove_all_three_in_reverse("local a local b local c") => [
            StatementReplacement::remove(statement_path(2)),
            StatementReplacement::remove(statement_path(0).span(2)),
        ] => "",
        remove_all_three_with_duplicated_removal("local a local b local c") => [
            StatementReplacement::remove(statement_path(0).span(3)),
            StatementReplacement::remove(statement_path(1)),
        ] => "",
        remove_all_three_with_duplicated_removal_reverse("local a local b local c") => [
            StatementReplacement::remove(statement_path(1)),
            StatementReplacement::remove(statement_path(0).span(3)),
        ] => "",
        remove_all_three_with_overlapping_upper_and_lower_bound("local a local b local c") => [
            StatementReplacement::remove(statement_path(0).span(2)),
            StatementReplacement::remove(statement_path(1).span(2)),
        ] => "",
        remove_two_first_with_overlapping_lower_bound("local a local b local c") => [
            StatementReplacement::remove(statement_path(0)),
            StatementReplacement::remove(statement_path(0).span(2)),
        ] => "local c",
        remove_two_last_with_overlapping_lower_bound("local a local b local c") => [
            StatementReplacement::remove(statement_path(1)),
            StatementReplacement::remove(statement_path(1).span(2)),
        ] => "local a",
        remove_two_last_with_overlapping_upper_bound("local a local b local c") => [
            StatementReplacement::remove(statement_path(2)),
            StatementReplacement::remove(statement_path(1).span(2)),
        ] => "local a",
        // add
        insert_twice_before_the_first_statement("return a + b") => [
            StatementInsertion::insert_before(statement_path(0), parse_insertion("local b")),
            StatementInsertion::insert_before(statement_path(0), parse_insertion("local a")),
        ] => "local a local b return a + b",
        insert_before_and_after_first_statement("local b") => [
            StatementInsertion::insert_before(statement_path(0), parse_insertion("local a")),
            StatementInsertion::insert_after(statement_path(0), parse_insertion("return a + b")),
        ] => "local a local b return a + b",
        insert_after_and_before_first_statement("local b") => [
            StatementInsertion::insert_after(statement_path(0), parse_insertion("return a + b")),
            StatementInsertion::insert_before(statement_path(0), parse_insertion("local a")),
        ] => "local a local b return a + b",
    );
}
