use super::expr::{Expr, InfixOp, PrefixOp};
use crate::rules::expr::Literal;
use crate::types::Feature;
use pest::{iterators::Pairs, pratt_parser::PrattParser, Parser};
use rule_parser::Rule as ParserRule;
use rule_parser::RuleParser;

mod rule_parser {
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "rules/rule_grammar.pest"]
    pub struct RuleParser;
}

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<ParserRule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use ParserRule::*;

        // Precedence is defined lowest to highest
        PrattParser::new()
        .op(Op::infix(equal, Left))
        .op(Op::infix(lower, Left) | Op::infix(greater, Left) | Op::infix(loweq, Left) | Op::infix(greateq, Left))
        .op(Op::infix(add, Left) | Op::infix(sub, Left))
        .op(Op::infix(implied, Left))
        .op(Op::infix(equiv, Left))
        .op(Op::infix(or, Left))
        .op(Op::infix(xor, Left))
        .op(Op::infix(and, Left))
        .op(Op::prefix(not) | Op::prefix(neg))
    };
}

fn parse_recur_expr(pairs: Pairs<ParserRule>) -> Expr {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            ParserRule::featname | ParserRule::simple_featname => {
                Expr::Literal(Literal::Feature(Feature(primary.as_str().to_owned())))
            }
            ParserRule::integer => {
                Expr::Literal(Literal::Integer(primary.as_str().parse().unwrap()))
            }
            ParserRule::recur_expr => parse_recur_expr(primary.into_inner()),
            rule => unreachable!("Expr::parse expected primary, found {:?}", rule),
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                ParserRule::and => InfixOp::And,
                ParserRule::xor => InfixOp::Xor,
                ParserRule::or => InfixOp::Or,
                ParserRule::equiv => InfixOp::Equiv,
                ParserRule::implied => InfixOp::Implies,
                ParserRule::add => InfixOp::Add,
                ParserRule::sub => InfixOp::Sub,
                ParserRule::lower => InfixOp::Lower,
                ParserRule::greater => InfixOp::Greater,
                ParserRule::loweq => InfixOp::LowEq,
                ParserRule::greateq => InfixOp::GreatEq,
                ParserRule::equal => InfixOp::Equal,
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };
            lhs.infix(op, rhs)
        })
        .map_prefix(|op, rhs| {
            let op = match op.as_rule() {
                ParserRule::not => PrefixOp::Not,
                ParserRule::neg => PrefixOp::Neg,
                rule => unreachable!("Expr::parse expected prefix operation, found {:?}", rule),
            };
            rhs.prefix(op)
        })
        .parse(pairs)
}

pub fn parse_expr(input: &str) -> Expr {
    parse_recur_expr(
        RuleParser::parse(ParserRule::expr, input)
            .unwrap()
            .next()
            .unwrap()
            .into_inner(),
    )
}

#[cfg(test)]
mod tests {
    use super::parse_expr;
    use crate::types::Feature;
    use std::collections::HashSet;

    #[test]
    fn parser() {
        let expr = parse_expr("h_a | c4 & (AB => std)");
        assert_eq!("h_a | (c4 & (AB => std))", format!("{}", expr));
        let h_a = Feature("h_a".to_owned());
        let c4 = Feature("c4".to_owned());
        let ab = Feature("AB".to_owned());
        let std = Feature("std".to_owned());
        assert!(!expr.eval(&HashSet::from([])).unwrap());
        assert!(!expr.eval(&HashSet::from([&ab])).unwrap());
        assert!(!expr.eval(&HashSet::from([&std])).unwrap());
        assert!(!expr.eval(&HashSet::from([&c4, &ab])).unwrap());
        assert!(!expr.eval(&HashSet::from([&ab, &std])).unwrap());
        assert!(expr.eval(&HashSet::from([&h_a])).unwrap());
        assert!(expr.eval(&HashSet::from([&c4])).unwrap());
        assert!(expr.eval(&HashSet::from([&h_a, &c4])).unwrap());
        assert!(expr.eval(&HashSet::from([&h_a, &ab])).unwrap());
        assert!(expr.eval(&HashSet::from([&h_a, &std])).unwrap());
        assert!(expr.eval(&HashSet::from([&c4, &std])).unwrap());
        assert!(expr.eval(&HashSet::from([&h_a, &c4, &ab])).unwrap());
        assert!(expr.eval(&HashSet::from([&h_a, &c4, &std])).unwrap());
        assert!(expr.eval(&HashSet::from([&h_a, &ab, &std])).unwrap());
        assert!(expr.eval(&HashSet::from([&c4, &ab, &std])).unwrap());
        assert!(expr.eval(&HashSet::from([&h_a, &c4, &ab, &std])).unwrap());
    }

    #[test]
    fn parse_infixes() {
        assert_eq!("A <=> B", format!("{}", parse_expr("A <=> B")));
        assert_eq!("A <= B", format!("{}", parse_expr("'A' <= B")));
        assert_eq!("1 & B", format!("{}", parse_expr("1 & B")));
    }
}
