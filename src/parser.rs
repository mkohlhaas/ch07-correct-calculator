// parser.rs - Parser for expressions

use crate::expression::{
    BinaryOperation, Expression, FunctionCall, NumberExpression, VariableExpression,
};
use crate::token::{Function, Operator, Token};

#[derive(Clone)]
pub struct ExpressionParser;

impl ExpressionParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, expression: &str) -> Result<Box<dyn Expression>, String> {
        // Tokenize
        let tokens = self.tokenize(expression)?;

        // Parse using Shunting-yard algorithm
        self.build_expression_tree(tokens)
    }

    fn tokenize(&self, input: &str) -> Result<Vec<Token>, String> {
        // This is a simplistic tokenizer for demonstration
        // A real tokenizer would be more sophisticated

        // Separate tokens by spaces for simplicity
        let tokens: Result<Vec<Token>, String> =
            input.split_whitespace().map(Token::from_str).collect();

        tokens
    }

    fn build_expression_tree(&self, tokens: Vec<Token>) -> Result<Box<dyn Expression>, String> {
        // Implementation of the Shunting Yard Algorithm
        let mut output_queue: Vec<Box<dyn Expression>> = Vec::new();
        let mut operator_stack: Vec<Token> = Vec::new();

        for token in tokens {
            match token {
                Token::Number(num) => {
                    output_queue.push(Box::new(NumberExpression::new(num.value)));
                }
                Token::Variable(name) => {
                    output_queue.push(Box::new(VariableExpression::new(name)));
                }
                Token::Operator(op) => {
                    // While there's an operator on the stack with greater precedence
                    while let Some(Token::Operator(top_op)) = operator_stack.last() {
                        // Compare precedence before mutably borrowing
                        let higher_precedence = top_op.precedence() >= op.precedence();

                        if higher_precedence {
                            // Now we can pop the operator safely
                            let top_token = operator_stack.pop().unwrap();

                            if output_queue.len() < 2 {
                                return Err("Invalid expression: not enough operands".to_string());
                            }

                            let right = output_queue.pop().unwrap();
                            let left = output_queue.pop().unwrap();

                            // Extract the operator
                            if let Token::Operator(top_operator) = top_token {
                                output_queue.push(Box::new(BinaryOperation::new(
                                    left,
                                    right,
                                    top_operator,
                                )));
                            }
                        } else {
                            break;
                        }
                    }

                    operator_stack.push(Token::Operator(op));
                }
                Token::Function(func) => {
                    operator_stack.push(Token::Function(func));
                }
                Token::OpenParen => {
                    operator_stack.push(token);
                }
                Token::CloseParen => {
                    // Pop until matching open paren
                    let mut found_open_paren = false;

                    while let Some(top) = operator_stack.pop() {
                        match top {
                            Token::OpenParen => {
                                found_open_paren = true;

                                // If there's a function on the stack, apply it
                                if let Some(Token::Function(_)) = operator_stack.last() {
                                    // Get the function token first to avoid borrow issues
                                    let func_token = operator_stack.pop().unwrap();

                                    if output_queue.is_empty() {
                                        return Err(
                                            "Invalid function call: missing argument".to_string()
                                        );
                                    }

                                    let arg = output_queue.pop().unwrap();

                                    // Now safely extract the function
                                    if let Token::Function(func) = func_token {
                                        output_queue.push(Box::new(FunctionCall::new(func, arg)));
                                    }
                                }

                                break;
                            }
                            Token::Operator(op) => {
                                if output_queue.len() < 2 {
                                    return Err(
                                        "Invalid expression: not enough operands".to_string()
                                    );
                                }

                                let right = output_queue.pop().unwrap();
                                let left = output_queue.pop().unwrap();

                                output_queue.push(Box::new(BinaryOperation::new(left, right, op)));
                            }
                            _ => {
                                return Err(format!(
                                    "Unexpected token on operator stack: {:?}",
                                    top
                                ));
                            }
                        }
                    }

                    if !found_open_paren {
                        return Err("Mismatched parentheses".to_string());
                    }
                }
            }
        }

        // Process remaining operators
        while let Some(token) = operator_stack.pop() {
            match token {
                Token::Operator(op) => {
                    if output_queue.len() < 2 {
                        return Err("Invalid expression: not enough operands".to_string());
                    }

                    let right = output_queue.pop().unwrap();
                    let left = output_queue.pop().unwrap();

                    output_queue.push(Box::new(BinaryOperation::new(left, right, op)));
                }
                Token::OpenParen | Token::CloseParen => {
                    return Err("Mismatched parentheses".to_string());
                }
                _ => {
                    return Err(format!("Unexpected token on operator stack: {:?}", token));
                }
            }
        }

        if output_queue.len() != 1 {
            return Err("Invalid expression: too many values".to_string());
        }

        Ok(output_queue.pop().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn eval(expression: &str) -> Result<f64, String> {
        let parser = ExpressionParser::new();
        let tree = parser.parse(expression)?;
        tree.evaluate(&HashMap::new())
    }

    fn eval_with_vars(expression: &str, values: &[(&str, f64)]) -> Result<f64, String> {
        let parser = ExpressionParser::new();
        let tree = parser.parse(expression)?;
        let variables: HashMap<String, f64> = values
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect();
        tree.evaluate(&variables)
    }

    #[test]
    fn parses_simple_arithmetic() {
        assert_eq!(eval("2 + 3").unwrap(), 5.0);
        assert_eq!(eval("8 - 3").unwrap(), 5.0);
        assert_eq!(eval("6 * 7").unwrap(), 42.0);
        assert_eq!(eval("10 / 4").unwrap(), 2.5);
        assert_eq!(eval("2 ^ 10").unwrap(), 1024.0);
    }

    #[test]
    fn parses_operator_precedence() {
        assert_eq!(eval("2 + 3 * 4").unwrap(), 14.0);
        assert_eq!(eval("2 * 3 + 4").unwrap(), 10.0);
        assert_eq!(eval("3 * 4 + 2 * 5").unwrap(), 22.0);
    }

    #[test]
    fn parses_parentheses() {
        assert_eq!(eval("( 1 + 2 ) * 3").unwrap(), 9.0);
        assert_eq!(eval("2 * ( 3 + 4 )").unwrap(), 14.0);
        assert_eq!(eval("( 2 + 3 ) * ( 4 + 5 )").unwrap(), 45.0);
    }

    #[test]
    fn parses_functions() {
        assert_eq!(eval("sqrt ( 16 )").unwrap(), 4.0);
        assert_eq!(eval("sin ( 0 )").unwrap(), 0.0);
        assert_eq!(eval("cos ( 0 )").unwrap(), 1.0);
    }

    #[test]
    fn parses_variables() {
        assert_eq!(eval_with_vars("x + 1", &[("x", 6.0)]).unwrap(), 7.0);
        assert_eq!(
            eval_with_vars("2 * x - 1", &[("x", 5.0)]).unwrap(),
            9.0
        );
    }

    #[test]
    fn rejects_too_few_operands() {
        assert_eq!(
            eval("5 +").unwrap_err(),
            "Invalid expression: not enough operands"
        );
        assert_eq!(
            eval("2 5").unwrap_err(),
            "Invalid expression: too many values"
        );
    }

    #[test]
    fn rejects_unbalanced_input() {
        assert_eq!(eval("( 1 + 2").unwrap_err(), "Mismatched parentheses");
        assert_eq!(eval("( 1 + 2 ) )").unwrap_err(), "Mismatched parentheses");
    }

    #[test]
    fn rejects_unknown_tokens() {
        assert!(eval("5 @ 3").is_err());
    }
}
