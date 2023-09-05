pub use expr::Expr;

mod expr;
mod parser;

pub mod new_rule {
    use super::expr::{Expr, InfixOp, Literal, PrefixOp};
    use crate::types::Feature;

    pub fn expr(expr: &str) -> Expr {
        Expr::new(expr)
    }

    pub fn chain<'a, I>(op: InfixOp, features: I) -> Option<Expr>
    where
        I: IntoIterator<Item = &'a Feature>,
    {
        let mut features = features.into_iter();
        let cur = features.next();
        if let Some(cur) = cur {
            let mut ans = Expr::Literal(Literal::Feature(cur.clone()));
            for cur in features {
                ans = ans.infix(op, Expr::Literal(Literal::Feature(cur.clone())));
            }
            Some(ans)
        } else {
            None
        }
    }

    pub fn all<'a, I>(features: I) -> Option<Expr>
    where
        I: IntoIterator<Item = &'a Feature>,
    {
        chain(InfixOp::And, features)
    }

    pub fn not_all<'a, I>(features: I) -> Option<Expr>
    where
        I: IntoIterator<Item = &'a Feature>,
    {
        all(features).map(|expr| expr.prefix(PrefixOp::Not))
    }

    pub fn any<'a, I>(features: I) -> Option<Expr>
    where
        I: IntoIterator<Item = &'a Feature>,
    {
        chain(InfixOp::Or, features)
    }

    pub fn not_any<'a, I>(features: I) -> Option<Expr>
    where
        I: IntoIterator<Item = &'a Feature>,
    {
        any(features).map(|expr| expr.prefix(PrefixOp::Not))
    }

    pub fn implication<'a, I>(if_all: I, then_all: I) -> Option<Expr>
    where
        I: IntoIterator<Item = &'a Feature>,
    {
        let then_all = all(then_all);
        if let Some(then_all) = then_all {
            let if_all = all(if_all);
            if let Some(if_all) = if_all {
                Some(if_all.infix(InfixOp::Implies, then_all))
            } else {
                Some(then_all)
            }
        } else {
            None
        }
    }
}
#[cfg(test)]
mod tests {
    use super::new_rule;
    use crate::types::{Feature, FeatureList};
    use std::collections::HashSet;

    #[test]
    fn new_implication_rule() {
        let if_all = vec![
            Feature("A".to_owned()),
            Feature("B".to_owned()),
            Feature("C".to_owned()),
        ];
        let then_all = vec![Feature("D".to_owned()), Feature("E".to_owned())];
        let mut all = if_all.clone();
        all.extend(then_all.clone());
        let implication_rule = new_rule::implication(
            FeatureList(if_all.clone()).iter(),
            FeatureList(then_all.clone()).iter(),
        )
        .unwrap();
        for i in 0..(1 << all.len()) {
            let mut select = HashSet::new();
            for (j, a) in all.iter().enumerate() {
                if (i >> j) & 1 == 1 {
                    select.insert(a);
                }
            }
            let mut if_all_covered = true;
            for a in if_all.iter() {
                if !select.contains(&a) {
                    if_all_covered = false;
                    break;
                }
            }
            let mut then_all_covered = true;
            for a in then_all.iter() {
                if !select.contains(&a) {
                    then_all_covered = false;
                    break;
                }
            }
            let decision = implication_rule.eval(&select).unwrap();
            if if_all_covered {
                assert!(decision == then_all_covered);
            } else {
                assert!(decision);
            }
        }
    }
}
