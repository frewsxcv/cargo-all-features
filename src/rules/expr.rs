use super::parser::parse_expr;
use crate::types::Feature;
use std::{collections::HashSet, fmt::Debug, fmt::Display};

pub enum Expr {
    Literal(Literal),
    Infix {
        lhs: Box<Expr>,
        op: InfixOp,
        rhs: Box<Expr>,
    },
    Prefix {
        op: PrefixOp,
        rhs: Box<Expr>,
    },
}

impl Expr {
    pub fn new(expr: &str) -> Self {
        parse_expr(expr)
    }

    pub fn infix(self, op: InfixOp, rhs: Self) -> Self {
        Self::Infix {
            lhs: Box::new(self),
            op,
            rhs: Box::new(rhs),
        }
    }

    pub fn prefix(self, op: PrefixOp) -> Self {
        Self::Prefix {
            op,
            rhs: Box::new(self),
        }
    }

    fn evaluate(&self, feature_set: &HashSet<&Feature>) -> Result<ExprResult, String> {
        match self {
            Expr::Literal(Literal::Feature(name)) => {
                Ok(ExprResult::Bool(feature_set.contains(name)))
            }
            Expr::Literal(Literal::Integer(int)) => Ok(ExprResult::Int(*int)),
            Expr::Infix { lhs, op, rhs } => {
                op.evaluate(lhs.evaluate(feature_set)?, rhs.evaluate(feature_set)?)
            }
            Expr::Prefix { op, rhs } => op.evaluate(rhs.evaluate(feature_set)?),
        }
    }

    pub fn eval(&self, feature_set: &HashSet<&Feature>) -> Result<bool, String> {
        let ans = self.evaluate(feature_set);
        match ans {
            Ok(ExprResult::Bool(b)) => Ok(b),
            Ok(ExprResult::Int(i)) => Err(format!(
                "provided rule returned integer {} instead of a boolean: {}",
                i,
                self.value_annotation_str(feature_set)
            )),
            Err(msg) => Err(format!(
                "{}: {}",
                msg,
                self.value_annotation_str(feature_set)
            )),
        }
    }

    fn value_annotation_str(&self, feature_set: &HashSet<&Feature>) -> String {
        let annotation = self.evaluate(feature_set);
        let annotation = match annotation {
            Ok(val) => format!("{}", val),
            Err(_) => "ERR".to_owned(),
        };
        match self {
            Expr::Literal(_) => format!("{}: {}", self, annotation),
            Expr::Infix { lhs, op, rhs } => format!(
                "({}) {} ({}): {}",
                lhs.value_annotation_str(feature_set),
                op,
                rhs.value_annotation_str(feature_set),
                annotation
            ),
            Expr::Prefix { op, rhs } => {
                format!(
                    "{}({}): {}",
                    op,
                    rhs.value_annotation_str(feature_set),
                    annotation
                )
            }
        }
    }

    fn inside_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Infix { .. } => {
                write!(f, "(")?;
                self.fmt(f)?;
                write!(f, ")")
            }
            _ => self.fmt(f),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Literal(literal) => literal.fmt(f),
            Self::Infix { lhs, op, rhs } => {
                lhs.inside_fmt(f)?;
                write!(f, " {} ", op)?;
                rhs.inside_fmt(f)
            }
            Self::Prefix { op, rhs } => {
                Display::fmt(&op, f)?;
                rhs.inside_fmt(f)
            }
        }
    }
}

#[derive(PartialEq)]
enum ExprResult {
    Bool(bool),
    Int(i32),
}

impl Display for ExprResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprResult::Bool(b) => write!(f, "{}", b),
            ExprResult::Int(i) => write!(f, "{}", i),
        }
    }
}

impl From<ExprResult> for i32 {
    fn from(value: ExprResult) -> Self {
        match value {
            ExprResult::Bool(b) => b as i32,
            ExprResult::Int(i) => i,
        }
    }
}

impl TryFrom<ExprResult> for bool {
    type Error = String;

    fn try_from(value: ExprResult) -> Result<Self, Self::Error> {
        match value {
            ExprResult::Bool(b) => Ok(b),
            ExprResult::Int(_) => Err("integer cannot be converted to bool".to_owned()),
        }
    }
}

pub enum Literal {
    Feature(Feature),
    Integer(i32),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Feature(feat) => Display::fmt(&feat.0, f),
            Literal::Integer(int) => Display::fmt(&int, f),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum InfixOp {
    Equal,
    Lower,
    Greater,
    LowEq,
    GreatEq,
    Add,
    Sub,
    Implies,
    Equiv,
    Or,
    Xor,
    And,
}

impl InfixOp {
    fn evaluate(&self, lhsr: ExprResult, rhsr: ExprResult) -> Result<ExprResult, String> {
        Ok(match self {
            InfixOp::Equal => ExprResult::Bool(lhsr == rhsr),
            InfixOp::Lower => ExprResult::Bool(i32::from(lhsr) < i32::from(rhsr)),
            InfixOp::Greater => ExprResult::Bool(i32::from(lhsr) > i32::from(rhsr)),
            InfixOp::LowEq => ExprResult::Bool(i32::from(lhsr) <= i32::from(rhsr)),
            InfixOp::GreatEq => ExprResult::Bool(i32::from(lhsr) >= i32::from(rhsr)),
            InfixOp::Add => ExprResult::Int(i32::from(lhsr) + i32::from(rhsr)),
            InfixOp::Sub => ExprResult::Int(i32::from(lhsr) - i32::from(rhsr)),
            InfixOp::Implies => ExprResult::Bool(!bool::try_from(lhsr)? || bool::try_from(rhsr)?),
            InfixOp::Equiv => ExprResult::Bool(bool::try_from(lhsr)? == bool::try_from(rhsr)?),
            InfixOp::Or => ExprResult::Bool(bool::try_from(lhsr)? || bool::try_from(rhsr)?),
            InfixOp::Xor => ExprResult::Bool(bool::try_from(lhsr)? ^ bool::try_from(rhsr)?),
            InfixOp::And => ExprResult::Bool(bool::try_from(lhsr)? && bool::try_from(rhsr)?),
        })
    }
}

impl Display for InfixOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InfixOp::Equal => write!(f, "=="),
            InfixOp::Lower => write!(f, "<"),
            InfixOp::Greater => write!(f, ">"),
            InfixOp::LowEq => write!(f, "<="),
            InfixOp::GreatEq => write!(f, ">="),
            InfixOp::Add => write!(f, "+"),
            InfixOp::Sub => write!(f, "-"),
            InfixOp::Implies => write!(f, "=>"),
            InfixOp::Equiv => write!(f, "<=>"),
            InfixOp::Or => write!(f, "|"),
            InfixOp::Xor => write!(f, "^"),
            InfixOp::And => write!(f, "&"),
        }
    }
}

#[derive(Debug)]
pub enum PrefixOp {
    Not,
    Neg,
}

impl PrefixOp {
    fn evaluate(&self, rhsr: ExprResult) -> Result<ExprResult, String> {
        Ok(match self {
            PrefixOp::Not => ExprResult::Bool(!bool::try_from(rhsr)?),
            PrefixOp::Neg => ExprResult::Int(-i32::from(rhsr)),
        })
    }
}

impl Display for PrefixOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrefixOp::Not => write!(f, "!"),
            PrefixOp::Neg => write!(f, "-"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::expr::InfixOp::*;
    use crate::{
        rules::{expr::Literal, Expr},
        types::Feature,
    };
    use std::collections::HashSet;

    #[test]
    fn lazy_eval_avoids_err() {
        let a = Expr::Literal(Literal::Feature(Feature("A".to_owned())));
        let b = Expr::Literal(Literal::Feature(Feature("B".to_owned())));
        let c = Expr::Literal(Literal::Feature(Feature("C".to_owned())));
        let expr = a.infix(Implies, b.infix(Add, c));
        let ans = expr.eval(&HashSet::from_iter([&Feature("B".to_owned())]));
        assert!(ans.is_ok_and(|a| a));
    }

    #[test]
    fn nonlazy_eval_produces_err() {
        let a = Expr::Literal(Literal::Feature(Feature("A".to_owned())));
        let b = Expr::Literal(Literal::Feature(Feature("B".to_owned())));
        let c = Expr::Literal(Literal::Feature(Feature("C".to_owned())));
        let expr = a.infix(Implies, b.infix(Add, c));
        let ans = expr.eval(&HashSet::from_iter([&Feature("A".to_owned())]));
        assert!(ans.is_err());
        assert_eq!(
            "integer cannot be converted to bool: (A: true) => ((B: false) + (C: false): 0): ERR",
            ans.err().unwrap()
        );
    }
}
