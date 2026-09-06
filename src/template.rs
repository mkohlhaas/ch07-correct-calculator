// template.rs - Template Method pattern implementation

use crate::expression::Expression;
use crate::token::Token;
use std::collections::HashMap;

// ============================================================ //
// 1. The Trait defines the contract and the algorithm skeleton //
// ============================================================ //

// Abstract base class defining template method
pub trait ExpressionEvaluator {
    // -------------------------------------------------------- //
    // A. The Template Method: This controls the fixed workflow //
    // -------------------------------------------------------- //

    // workflow: tokenize -> validate -> parse -> evaluate
    fn evaluate(&self, expression: &str, variables: &HashMap<String, f64>) -> Result<f64, String> {
        // 1. Tokenize the expression
        let tokens = self.tokenize(expression)?;

        // 2. Validate tokens
        self.validate_tokens(&tokens)?;

        // 3. Parse into structured form (implementation varies)
        let parsed = self.parse(tokens)?;

        // 4. Evaluate the structure
        self.evaluate_parsed(parsed, variables)
    }

    // ---------------------------------------------- //
    // B. Shared default behavior for invariant steps //
    // ---------------------------------------------- //

    fn tokenize(&self, expression: &str) -> Result<Vec<Token>, String> {
        // Default tokenization implementation
        // Space-delimited for simplicity
        let tokens: Result<Vec<Token>, String> =
            expression.split_whitespace().map(Token::from_str).collect();

        tokens
    }

    fn validate_tokens(&self, tokens: &[Token]) -> Result<(), String> {
        // Default validation implementation
        if tokens.is_empty() {
            return Err("Empty expression".to_string());
        }

        // Ensure parentheses are balanced
        let mut paren_depth = 0;

        for token in tokens {
            match token {
                Token::OpenParen => paren_depth += 1,
                Token::CloseParen => {
                    paren_depth -= 1;
                    if paren_depth < 0 {
                        return Err("Mismatched parentheses".to_string());
                    }
                }
                _ => {}
            }
        }

        if paren_depth != 0 {
            return Err("Mismatched parentheses".to_string());
        }

        Ok(())
    }

    // -------------------------------------------------------- //
    // C. Abstract methods: Concrete types must implement these //
    // -------------------------------------------------------- //

    fn parse(&self, tokens: Vec<Token>) -> Result<Box<dyn Expression>, String>;

    fn evaluate_parsed(
        &self,
        expression: Box<dyn Expression>,
        variables: &HashMap<String, f64>,
    ) -> Result<f64, String>;
}

// =========================== //
// Recursive Descent Evaluator //
// =========================== //

// Concrete implementation using recursive descent
pub struct RecursiveDescentEvaluator;

impl RecursiveDescentEvaluator {
    pub fn new() -> Self {
        Self
    }

    // Helper function for recursive descent parsing
    fn parse_expression(&self, tokens: &[Token]) -> Result<Box<dyn Expression>, String> {
        if tokens.is_empty() {
            return Err("Empty expression".to_string());
        }

        self.parse_addition(tokens, 0).map(|(expr, _)| expr)
    }

    fn parse_addition(
        &self,
        tokens: &[Token],
        pos: usize,
    ) -> Result<(Box<dyn Expression>, usize), String> {
        // Parse left operand (higher precedence)
        let (mut left, mut next_pos) = self.parse_multiplication(tokens, pos)?;

        // Continue parsing addition/subtraction operators
        while next_pos < tokens.len() {
            match &tokens[next_pos] {
                Token::Operator(op) if op.precedence() == 1 => {
                    // Parse right operand
                    let (right, new_pos) = self.parse_multiplication(tokens, next_pos + 1)?;

                    // Create binary operation node
                    left = Box::new(crate::expression::BinaryOperation::new(
                        left,
                        right,
                        op.clone(),
                    ));

                    next_pos = new_pos;
                }
                _ => break,
            }
        }

        Ok((left, next_pos))
    }

    fn parse_multiplication(
        &self,
        tokens: &[Token],
        pos: usize,
    ) -> Result<(Box<dyn Expression>, usize), String> {
        // Parse left operand (higher precedence)
        let (mut left, mut next_pos) = self.parse_primary(tokens, pos)?;

        // Continue parsing multiplication/division operators
        while next_pos < tokens.len() {
            match &tokens[next_pos] {
                Token::Operator(op) if op.precedence() >= 2 => {
                    // Parse right operand
                    let (right, new_pos) = self.parse_primary(tokens, next_pos + 1)?;

                    // Create binary operation node
                    left = Box::new(crate::expression::BinaryOperation::new(
                        left,
                        right,
                        op.clone(),
                    ));

                    next_pos = new_pos;
                }
                _ => break,
            }
        }

        Ok((left, next_pos))
    }

    fn parse_primary(
        &self,
        tokens: &[Token],
        pos: usize,
    ) -> Result<(Box<dyn Expression>, usize), String> {
        if pos >= tokens.len() {
            return Err("Unexpected end of expression".to_string());
        }

        match &tokens[pos] {
            Token::Number(num) => {
                // Parse number literal
                Ok((
                    Box::new(crate::expression::NumberExpression::new(num.value)),
                    pos + 1,
                ))
            }
            Token::Variable(name) => {
                // Parse variable
                Ok((
                    Box::new(crate::expression::VariableExpression::new(name.clone())),
                    pos + 1,
                ))
            }
            Token::Function(func) => {
                // Parse function call
                if pos + 1 >= tokens.len() || tokens[pos + 1] != Token::OpenParen {
                    return Err("Expected '(' after function name".to_string());
                }

                // Parse argument expression
                let (arg, next_pos) = self
                    .parse_expression(&tokens[pos + 2..])
                    .map(|e| (e, pos + 2))?;

                // Ensure closing parenthesis
                if next_pos >= tokens.len() || tokens[next_pos] != Token::CloseParen {
                    return Err("Expected ')' after function argument".to_string());
                }

                Ok((
                    Box::new(crate::expression::FunctionCall::new(func.clone(), arg)),
                    next_pos + 1,
                ))
            }
            Token::OpenParen => {
                // Parse parenthesized expression
                let (expr, next_pos) = self
                    .parse_expression(&tokens[pos + 1..])
                    .map(|e| (e, pos + 1))?;

                // Ensure closing parenthesis
                if next_pos >= tokens.len() || tokens[next_pos] != Token::CloseParen {
                    return Err("Expected ')'".to_string());
                }

                Ok((expr, next_pos + 1))
            }
            _ => Err(format!("Unexpected token: {:?}", tokens[pos])),
        }
    }
}

impl ExpressionEvaluator for RecursiveDescentEvaluator {
    fn parse(&self, tokens: Vec<Token>) -> Result<Box<dyn Expression>, String> {
        self.parse_expression(&tokens)
    }

    fn evaluate_parsed(
        &self,
        expression: Box<dyn Expression>,
        variables: &HashMap<String, f64>,
    ) -> Result<f64, String> {
        expression.evaluate(variables)
    }
}

// ======================= //
// Shunting-Yard Evaluator //
// ======================= //

// Concrete implementation using shunting yard algorithm
pub struct ShuntingYardEvaluator;

impl ShuntingYardEvaluator {
    pub fn new() -> Self {
        Self
    }
}

impl ExpressionEvaluator for ShuntingYardEvaluator {
    fn parse(&self, tokens: Vec<Token>) -> Result<Box<dyn Expression>, String> {
        // Implementation of shunting yard algorithm
        // Use our parser instead of trying to reimplement
        crate::parser::ExpressionParser::new().parse(
            &tokens
                .iter()
                .map(|t| match t {
                    Token::Number(n) => format!("{}", n.value),
                    Token::Variable(v) => v.clone(),
                    Token::Operator(op) => op.symbol().to_string(),
                    Token::Function(f) => match f {
                        crate::token::Function::Sin => "sin".to_string(),
                        crate::token::Function::Cos => "cos".to_string(),
                        crate::token::Function::Tan => "tan".to_string(),
                        crate::token::Function::Sqrt => "sqrt".to_string(),
                    },
                    Token::OpenParen => "(".to_string(),
                    Token::CloseParen => ")".to_string(),
                })
                .collect::<Vec<String>>()
                .join(" "),
        )
    }

    fn evaluate_parsed(
        &self,
        expression: Box<dyn Expression>,
        variables: &HashMap<String, f64>,
    ) -> Result<f64, String> {
        expression.evaluate(variables)
    }

    // Custom validation specific to shunting yard
    fn validate_tokens(&self, tokens: &[Token]) -> Result<(), String> {
        // Do the basic validation inline to avoid recursive calls
        if tokens.is_empty() {
            return Err("No tokens to evaluate".to_string());
        }

        // Check for balanced parenthesis
        let mut paren_count = 0;

        for token in tokens {
            match token {
                Token::OpenParen => paren_count += 1,
                Token::CloseParen => {
                    paren_count -= 1;
                    if paren_count < 0 {
                        return Err("Unbalanced parenthesis".to_string());
                    }
                }
                _ => {}
            }
        }

        if paren_count != 0 {
            return Err("Unbalanced parenthesis".to_string());
        }

        // Additional validation for shunting yard
        let mut operand_count = 0;
        let mut operator_count = 0;

        for token in tokens {
            match token {
                Token::Number(_) | Token::Variable(_) => operand_count += 1,
                Token::Operator(_) => operator_count += 1,
                _ => {}
            }
        }

        // Basic check for balanced expressions
        if operand_count == 0 {
            return Err("Expression must contain at least one operand".to_string());
        }

        if operand_count != operator_count + 1 && !tokens.is_empty() {
            // This is a simplified check - real validation would be more complex
            // We're ignoring parentheses and functions here
            return Err("Unbalanced expression: check operands and operators".to_string());
        }

        Ok(())
    }
}

// Factory function for creating evaluators
pub fn create_evaluator(use_recursive_descent: bool) -> Box<dyn ExpressionEvaluator> {
    if use_recursive_descent {
        Box::new(RecursiveDescentEvaluator::new())
    } else {
        Box::new(ShuntingYardEvaluator::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn eval(evaluator: &dyn ExpressionEvaluator, expression: &str) -> Result<f64, String> {
        evaluator.evaluate(expression, &HashMap::new())
    }

    fn eval_with_vars(
        evaluator: &dyn ExpressionEvaluator,
        expression: &str,
        variables: &[(&str, f64)],
    ) -> Result<f64, String> {
        let vars: HashMap<String, f64> =
            variables.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        evaluator.evaluate(expression, &vars)
    }

    #[test]
    fn recursive_descent_evaluates_arithmetic() {
        let evaluator = create_evaluator(true);
        assert_eq!(eval(&*evaluator, "2 + 3 * 4").unwrap(), 14.0);
        assert_eq!(eval(&*evaluator, "10 - 3 + 1").unwrap(), 8.0);
        assert_eq!(eval(&*evaluator, "6 * 7").unwrap(), 42.0);
    }

    #[test]
    fn recursive_descent_evaluates_variable_expressions() {
        let evaluator = create_evaluator(true);
        assert_eq!(
            eval_with_vars(&*evaluator, "x + 1", &[("x", 6.0)]).unwrap(),
            7.0
        );
        assert_eq!(
            eval_with_vars(&*evaluator, "x * 3", &[("x", 4.0)]).unwrap(),
            12.0
        );
    }

    #[test]
    fn shunting_yard_evaluates_arithmetic() {
        let evaluator = create_evaluator(false);
        assert_eq!(eval(&*evaluator, "2 + 3 * 4").unwrap(), 14.0);
        assert_eq!(eval(&*evaluator, "( 1 + 2 ) * 3").unwrap(), 9.0);
        assert_eq!(eval(&*evaluator, "2 * ( 3 + 4 )").unwrap(), 14.0);
    }

    #[test]
    fn shunting_yard_evaluates_functions_and_variables() {
        let evaluator = create_evaluator(false);
        assert_eq!(eval(&*evaluator, "sqrt ( 16 )").unwrap(), 4.0);
        assert_eq!(
            eval_with_vars(&*evaluator, "x * 2", &[("x", 5.0)]).unwrap(),
            10.0
        );
    }

    #[test]
    fn empty_expression_is_rejected() {
        let recursive = create_evaluator(true);
        assert!(eval(&*recursive, "").is_err());

        let shunting_yard = create_evaluator(false);
        assert!(eval(&*shunting_yard, "").is_err());
    }

    #[test]
    fn mismatched_parentheses_are_rejected() {
        let recursive = create_evaluator(true);
        assert_eq!(
            eval(&*recursive, "( 1 + 2").unwrap_err(),
            "Mismatched parentheses"
        );

        let shunting_yard = create_evaluator(false);
        assert_eq!(
            eval(&*shunting_yard, "( 1 + 2").unwrap_err(),
            "Unbalanced parenthesis"
        );
    }

    #[test]
    fn shunting_yard_rejects_unbalanced_operands() {
        let evaluator = create_evaluator(false);
        assert_eq!(
            eval(&*evaluator, "2 +").unwrap_err(),
            "Unbalanced expression: check operands and operators"
        );
    }

    #[test]
    fn recursive_descent_rejects_undefined_variables() {
        let evaluator = create_evaluator(true);
        assert_eq!(
            eval(&*evaluator, "x + 1").unwrap_err(),
            "Undefined variable: x"
        );
    }

    #[test]
    fn factory_returns_proper_types() {
        let mut variables = HashMap::new();
        variables.insert("x".to_string(), 3.0);

        let recursive = create_evaluator(true);
        assert_eq!(recursive.evaluate("x * 3", &variables).unwrap(), 9.0);

        let shunting_yard = create_evaluator(false);
        assert_eq!(shunting_yard.evaluate("x * 3", &variables).unwrap(), 9.0);
    }
}
