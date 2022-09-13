use crate::{
    nodes::{LastStatement, Statement},
    process::{mutation::Mutation, path::NodePathSlice},
};

pub trait StatementVisitor<Arguments, Ret, Context = ()> {
    fn statement(
        &self,
        _node: &Statement,
        _path: &NodePathSlice,
        _context: &mut Context,
    ) -> Option<Mutation> {
        None
    }

    fn last_statement(
        &self,
        _node: &LastStatement,
        _path: &NodePathSlice,
        _context: &mut Context,
    ) -> Option<Mutation> {
        None
    }
}

macro_rules! implement_statement_visitor {
    ($node_type:ty, $callback_identifier:ident) => {
        impl<F, C> StatementVisitor<(&$node_type, &NodePathSlice, &mut C), Option<Mutation>, C>
            for F
        where
            F: Fn(&$node_type, &NodePathSlice, &mut C) -> Option<Mutation>,
        {
            fn $callback_identifier(
                &self,
                node: &$node_type,
                path: &NodePathSlice,
                context: &mut C,
            ) -> Option<Mutation> {
                (self)(node, path, context)
            }
        }

        impl<F, C> StatementVisitor<(&$node_type, &NodePathSlice), Option<Mutation>, C> for F
        where
            F: Fn(&$node_type, &NodePathSlice) -> Option<Mutation>,
        {
            fn $callback_identifier(
                &self,
                node: &$node_type,
                path: &NodePathSlice,
                _context: &mut C,
            ) -> Option<Mutation> {
                (self)(node, path)
            }
        }

        impl<F, C> StatementVisitor<&$node_type, Option<Mutation>, C> for F
        where
            F: Fn(&$node_type) -> Option<Mutation>,
        {
            fn $callback_identifier(
                &self,
                node: &$node_type,
                _path: &NodePathSlice,
                _context: &mut C,
            ) -> Option<Mutation> {
                (self)(node)
            }
        }

        impl<F, C> StatementVisitor<(&$node_type, &NodePathSlice, &mut C), (), C> for F
        where
            F: Fn(&$node_type, &NodePathSlice, &mut C),
        {
            fn $callback_identifier(
                &self,
                node: &$node_type,
                path: &NodePathSlice,
                context: &mut C,
            ) -> Option<Mutation> {
                (self)(node, path, context);
                None
            }
        }

        impl<F, C> StatementVisitor<(&$node_type, &NodePathSlice), (), C> for F
        where
            F: Fn(&$node_type, &NodePathSlice),
        {
            fn $callback_identifier(
                &self,
                node: &$node_type,
                path: &NodePathSlice,
                _context: &mut C,
            ) -> Option<Mutation> {
                (self)(node, path);
                None
            }
        }

        impl<F, C> StatementVisitor<(&$node_type, &mut C), (), C> for F
        where
            F: Fn(&$node_type, &mut C),
        {
            fn $callback_identifier(
                &self,
                node: &$node_type,
                _path: &NodePathSlice,
                context: &mut C,
            ) -> Option<Mutation> {
                (self)(node, context);
                None
            }
        }

        impl<F, C> StatementVisitor<&$node_type, (), C> for F
        where
            F: Fn(&$node_type),
        {
            fn $callback_identifier(
                &self,
                node: &$node_type,
                _path: &NodePathSlice,
                _context: &mut C,
            ) -> Option<Mutation> {
                (self)(node);
                None
            }
        }
    };
}

implement_statement_visitor!(Statement, statement);
implement_statement_visitor!(LastStatement, last_statement);
