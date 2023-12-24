use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionOperand {
    Symbol(String),
    Value(i32),
    Locctr,
}

impl TryFrom<&str> for ExpressionOperand {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value == "*" {
            Ok(ExpressionOperand::Locctr)
        } else if value.chars().all(|c| c.is_ascii_digit()) {
            // All characters are digits
            let value = value
                .parse::<i32>()
                .map_err(|_| "Failed to parse value".to_string())?;
            Ok(ExpressionOperand::Value(value))
        } else {
            // Not all characters are digits
            Ok(ExpressionOperand::Symbol(value.to_string()))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl Display for ExpressionOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionOperator::Add => write!(f, "+"),
            ExpressionOperator::Subtract => write!(f, "-"),
            ExpressionOperator::Multiply => write!(f, "*"),
            ExpressionOperator::Divide => write!(f, "/"),
        }
    }
}

/// An expression is a combination of operands and operators, or a single operand.
/// Notice: locctr `*` cannot be used in expressions, should be resolved before parsing.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Unsolved(UnsolvedExpression),
    Resolved(i32),
    Literal(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnsolvedExpression {
    pub left: ExpressionOperand,
    pub op: Option<ExpressionOperator>,
    pub right: Option<ExpressionOperand>,
}

impl Expression {
    pub fn eval(&self) -> Option<i32> {
        match self {
            Expression::Unsolved(expr) => {
                match expr.op {
                    None => match expr.left {
                        ExpressionOperand::Value(value) => {
                            Some(value)
                        }
                        _ => None,
                    },
                    _ => {
                        let left = match expr.left {
                            ExpressionOperand::Value(value) => Some(value),
                            _ => None,
                        };
                        let right = match expr.right {
                            Some(ExpressionOperand::Value(value)) => Some(value),
                            _ => None,
                        };

                        

                        match (left, right) {
                            (Some(left), Some(right)) => match expr.op {
                                Some(ExpressionOperator::Add) => Some(left + right),
                                Some(ExpressionOperator::Subtract) => Some(left - right),
                                Some(ExpressionOperator::Multiply) => Some(left * right),
                                Some(ExpressionOperator::Divide) => Some(left / right),
                                None => None,
                            },
                            _ => None,
                        }
                    }
                }
            }
            Expression::Resolved(value) => Some(*value),
            Expression::Literal(_) => {
                panic!("Literal expression should be resolved before evaluation")
            }
        }
    }

    pub fn eval_and_update(&mut self) -> Option<i32> {
        let res = self.eval();
        if let Some(value) = res {
            *self = Expression::Resolved(value);
        }

        res
    }

    pub fn deps(&self) -> Vec<&str> {
        match self {
            Expression::Resolved(_) => vec![],
            Expression::Unsolved(expr) => {
                let mut deps = Vec::new();

                match expr.left {
                    ExpressionOperand::Symbol(ref symbol) => deps.push(symbol.as_str()),
                    _ => {}
                }

                match expr.right {
                    Some(ExpressionOperand::Symbol(ref symbol)) => deps.push(symbol.as_str()),
                    _ => {}
                }

                deps
            }
            Expression::Literal(_) => vec![],
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Resolved(value) => write!(f, "{}", value),
            Expression::Unsolved(expr) => {
                let mut string = String::new();

                match expr.left {
                    ExpressionOperand::Symbol(ref symbol) => string.push_str(symbol),
                    ExpressionOperand::Value(value) => string.push_str(value.to_string().as_str()),
                    ExpressionOperand::Locctr => string.push('*'),
                }

                match expr.op {
                    Some(ExpressionOperator::Add) => string.push('+'),
                    Some(ExpressionOperator::Subtract) => string.push('-'),
                    Some(ExpressionOperator::Multiply) => string.push('*'),
                    Some(ExpressionOperator::Divide) => string.push('/'),
                    None => {}
                }

                match expr.right {
                    Some(ExpressionOperand::Symbol(ref symbol)) => string.push_str(symbol),
                    Some(ExpressionOperand::Value(value)) => {
                        string.push_str(value.to_string().as_str())
                    }
                    Some(ExpressionOperand::Locctr) => string.push('*'),
                    None => {}
                }

                write!(f, "{}", string)
            }
            Expression::Literal(ref literal) => write!(f, "{}", literal),
        }
    }
}

pub fn parse(input: &str) -> Result<Expression, String> {
    if input.is_empty() {
        return Err("Input is empty".to_string());
    }

    if input.starts_with('=') {
        return Ok(Expression::Literal(input[1..].to_string()));
    }

    if input.chars().all(|c| c.is_ascii_digit()) {
        let value = input
            .parse::<i32>()
            .map_err(|_| "Failed to parse value".to_string())?;
        return Ok(Expression::Resolved(value));
    }

    let mut operator = None;
    let mut split_index = None;
    let mut operator_count = 0;

    for (index, char) in input.chars().enumerate() {
        if index == 0 && char == '*' {
            // Skip the locctr
            continue;
        }

        match char {
            '+' | '-' | '*' | '/' => {
                operator_count += 1;
                if operator_count > 1 {
                    // If more than one operator is found, return an error
                    return Err("Multiple operators detected".to_string());
                }

                operator = match char {
                    '+' => Some(ExpressionOperator::Add),
                    '-' => Some(ExpressionOperator::Subtract),
                    '*' => Some(ExpressionOperator::Multiply),
                    '/' => Some(ExpressionOperator::Divide),
                    _ => None, // This case will never occur due to the match pattern
                };
                split_index = Some(index);
            }
            _ => {}
        }
    }

    if operator_count == 0 {
        let operand = ExpressionOperand::try_from(input)?;
        Ok(Expression::Unsolved(UnsolvedExpression {
            left: operand,
            op: None,
            right: None,
        }))
    } else {
        let (left, right) = input.split_at(split_index.unwrap());
        let left_operand = ExpressionOperand::try_from(left)?;
        let right_operand = ExpressionOperand::try_from(right[1..].to_string().as_str())?;

        let expr = Expression::Unsolved(UnsolvedExpression {
            left: left_operand,
            op: operator,
            right: Some(right_operand),
        });

        Ok(expr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_operator() {
        let expr = parse("1+2+3");
        assert_eq!(expr, Err("Multiple operators detected".to_string()));
    }

    #[test]
    fn test_value_value() {
        let expr = parse("1*2");
        assert_eq!(
            expr,
            Ok(Expression::Unsolved(UnsolvedExpression {
                left: ExpressionOperand::Value(1),
                op: Some(ExpressionOperator::Multiply),
                right: Some(ExpressionOperand::Value(2)),
            }))
        );
    }

    #[test]
    fn test_symbol_value() {
        let expr = parse("ABC+123");
        assert_eq!(
            expr,
            Ok(Expression::Unsolved(UnsolvedExpression {
                left: ExpressionOperand::Symbol("ABC".to_string()),
                op: Some(ExpressionOperator::Add),
                right: Some(ExpressionOperand::Value(123)),
            }))
        );
    }

    #[test]
    fn test_value_symbol() {
        let expr = parse("123+ABC");
        assert_eq!(
            expr,
            Ok(Expression::Unsolved(UnsolvedExpression {
                left: ExpressionOperand::Value(123),
                op: Some(ExpressionOperator::Add),
                right: Some(ExpressionOperand::Symbol("ABC".to_string())),
            }))
        );
    }

    #[test]
    fn test_symbol_symbol() {
        let expr = parse("ABC-DEF");
        assert_eq!(
            expr,
            Ok(Expression::Unsolved(UnsolvedExpression {
                left: ExpressionOperand::Symbol("ABC".to_string()),
                op: Some(ExpressionOperator::Subtract),
                right: Some(ExpressionOperand::Symbol("DEF".to_string())),
            }))
        );
    }

    #[test]
    fn test_value() {
        let expr = parse("12345");
        assert_eq!(expr, Ok(Expression::Resolved(12345)));
    }

    #[test]
    fn test_symbol() {
        let expr = parse("ABC123");
        assert_eq!(
            expr,
            Ok(Expression::Unsolved(UnsolvedExpression {
                left: ExpressionOperand::Symbol("ABC123".to_string()),
                op: None,
                right: None,
            }))
        );
    }

    #[test]
    fn test_literal() {
        let expr = parse("=C'IT\\'S A STRING'");
        assert_eq!(
            expr,
            Ok(Expression::Literal("C'IT\\'S A STRING'".to_string()))
        );
    }
}
