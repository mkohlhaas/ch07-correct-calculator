// expression.rs - Expression tree implementation (from Chapter 6)

use crate::token::{Function, Operator};
use std::collections::HashMap;

// Expression trait defining common behavior
pub trait Expression {
    fn evaluate(&self, variables: &HashMap<String, f64>) -> Result<f64, String>;
    fn to_string(&self) -> String;

    // For debugging and visualization
    fn precedence(&self) -> u8 {
        0 // Leaf nodes have lowest precedence by default
    }
}

// Leaf node for number values
#[derive(Debug, Clone)]
pub struct NumberExpression {
    pub value: f64,
}

impl NumberExpression {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl Expression for NumberExpression {
    fn evaluate(&self, _variables: &HashMap<String, f64>) -> Result<f64, String> {
        Ok(self.value)
    }

    fn to_string(&self) -> String {
        format!("{}", self.value)
    }
}

// Leaf node for variables
#[derive(Debug, Clone)]
pub struct VariableExpression {
    pub name: String,
}

impl VariableExpression {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Expression for VariableExpression {
    fn evaluate(&self, variables: &HashMap<String, f64>) -> Result<f64, String> {
        variables
            .get(&self.name)
            .copied()
            .ok_or_else(|| format!("Undefined variable: {}", self.name))
    }

    fn to_string(&self) -> String {
        self.name.clone()
    }
}

// Composite node for binary operations
// Cannot derive Debug and Clone for Box<dyn Expression>
pub struct BinaryOperation {
    pub left: Box<dyn Expression>,
    pub right: Box<dyn Expression>,
    pub operator: Operator,
}

// Manual implementation of Debug
impl std::fmt::Debug for BinaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BinaryOperation {{ left: <expr>, right: <expr>, operator: {:?} }}",
            self.operator
        )
    }
}

impl BinaryOperation {
    pub fn new(left: Box<dyn Expression>, right: Box<dyn Expression>, operator: Operator) -> Self {
        Self {
            left,
            right,
            operator,
        }
    }

    fn operator_symbol(&self) -> &'static str {
        match self.operator {
            Operator::Add => "+",
            Operator::Subtract => "-",
            Operator::Multiply => "*",
            Operator::Divide => "/",
            Operator::Power => "^",
        }
    }
}

impl Expression for BinaryOperation {
    fn evaluate(&self, variables: &HashMap<String, f64>) -> Result<f64, String> {
        let left_val = self.left.evaluate(variables)?;
        let right_val = self.right.evaluate(variables)?;

        match self.operator {
            Operator::Add => Ok(left_val + right_val),
            Operator::Subtract => Ok(left_val - right_val),
            Operator::Multiply => Ok(left_val * right_val),
            Operator::Divide => {
                if right_val == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(left_val / right_val)
                }
            }
            Operator::Power => Ok(left_val.powf(right_val)),
        }
    }

    fn to_string(&self) -> String {
        let left_str = if self.left.precedence() < self.precedence() {
            format!("({})", self.left.to_string())
        } else {
            self.left.to_string()
        };

        let right_str = if self.right.precedence() < self.precedence() {
            format!("({})", self.right.to_string())
        } else {
            self.right.to_string()
        };

        format!("{} {} {}", left_str, self.operator_symbol(), right_str)
    }

    fn precedence(&self) -> u8 {
        match self.operator {
            Operator::Add | Operator::Subtract => 1,
            Operator::Multiply | Operator::Divide => 2,
            Operator::Power => 3,
        }
    }
}

// Function call expression
// Cannot derive Debug and Clone for Box<dyn Expression>
pub struct FunctionCall {
    pub function: Function,
    pub argument: Box<dyn Expression>,
}

// Manual implementation of Debug
impl std::fmt::Debug for FunctionCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FunctionCall {{ function: {:?}, argument: <expr> }}",
            self.function
        )
    }
}

impl FunctionCall {
    pub fn new(function: Function, argument: Box<dyn Expression>) -> Self {
        Self { function, argument }
    }
}

impl Expression for FunctionCall {
    fn evaluate(&self, variables: &HashMap<String, f64>) -> Result<f64, String> {
        let arg_val = self.argument.evaluate(variables)?;

        match self.function {
            Function::Sin => Ok(arg_val.sin()),
            Function::Cos => Ok(arg_val.cos()),
            Function::Tan => {
                if (arg_val - std::f64::consts::PI / 2.0).abs() % std::f64::consts::PI < 1e-10 {
                    Err("Tangent undefined at this value".to_string())
                } else {
                    Ok(arg_val.tan())
                }
            }
            Function::Sqrt => {
                if arg_val < 0.0 {
                    Err("Cannot take square root of negative number".to_string())
                } else {
                    Ok(arg_val.sqrt())
                }
            }
        }
    }

    fn to_string(&self) -> String {
        let func_name = match self.function {
            Function::Sin => "sin",
            Function::Cos => "cos",
            Function::Tan => "tan",
            Function::Sqrt => "sqrt",
        };

        format!("{}({})", func_name, self.argument.to_string())
    }

    fn precedence(&self) -> u8 {
        4 // Function calls have highest precedence
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Function, Operator};

    fn vars() -> HashMap<String, f64> {
        HashMap::new()
    }

    fn number(value: f64) -> Box<dyn Expression> {
        Box::new(NumberExpression::new(value))
    }

    #[test]
    fn number_expression_evaluates_to_its_value() {
        let expr = NumberExpression::new(42.5);
        assert_eq!(expr.evaluate(&vars()).unwrap(), 42.5);
        assert_eq!(expr.to_string(), "42.5");
        assert_eq!(expr.precedence(), 0);
    }

    #[test]
    fn variable_expression_reads_from_variables() {
        let expr = VariableExpression::new("x");
        assert_eq!(expr.to_string(), "x");

        let mut variables = vars();
        variables.insert("x".to_string(), 7.0);
        assert_eq!(expr.evaluate(&variables).unwrap(), 7.0);
    }

    #[test]
    fn variable_expression_errors_when_undefined() {
        let expr = VariableExpression::new("missing");
        let err = expr.evaluate(&vars()).unwrap_err();
        assert_eq!(err, "Undefined variable: missing");
    }

    #[test]
    fn binary_operation_arithmetic() {
        let add = BinaryOperation::new(number(2.0), number(3.0), Operator::Add);
        assert_eq!(add.evaluate(&vars()).unwrap(), 5.0);

        let sub = BinaryOperation::new(number(8.0), number(3.0), Operator::Subtract);
        assert_eq!(sub.evaluate(&vars()).unwrap(), 5.0);

        let mul = BinaryOperation::new(number(6.0), number(7.0), Operator::Multiply);
        assert_eq!(mul.evaluate(&vars()).unwrap(), 42.0);

        let pow = BinaryOperation::new(number(2.0), number(10.0), Operator::Power);
        assert_eq!(pow.evaluate(&vars()).unwrap(), 1024.0);
    }

    #[test]
    fn binary_operation_division() {
        let div = BinaryOperation::new(number(10.0), number(4.0), Operator::Divide);
        assert_eq!(div.evaluate(&vars()).unwrap(), 2.5);

        let div_by_zero = BinaryOperation::new(number(10.0), number(0.0), Operator::Divide);
        assert_eq!(div_by_zero.evaluate(&vars()).unwrap_err(), "Division by zero");
    }

    #[test]
    fn binary_operation_precedence() {
        assert_eq!(
            BinaryOperation::new(number(1.0), number(2.0), Operator::Add).precedence(),
            1
        );
        assert_eq!(
            BinaryOperation::new(number(1.0), number(2.0), Operator::Multiply).precedence(),
            2
        );
        assert_eq!(
            BinaryOperation::new(number(1.0), number(2.0), Operator::Power).precedence(),
            3
        );
    }

    #[test]
    fn binary_operation_to_string_adds_parens_when_needed() {
        let simple = BinaryOperation::new(number(1.0), number(2.0), Operator::Add);
        assert_eq!(simple.to_string(), "(1) + (2)");

        let nested = BinaryOperation::new(number(1.0), number(2.0), Operator::Add);
        let outer = BinaryOperation::new(number(3.0), Box::new(nested), Operator::Multiply);
        assert_eq!(outer.to_string(), "(3) * ((1) + (2))");
    }

    #[test]
    fn function_call_evaluates_functions() {
        let sin = FunctionCall::new(Function::Sin, number(0.0));
        assert_eq!(sin.evaluate(&vars()).unwrap(), 0.0);

        let cos = FunctionCall::new(Function::Cos, number(0.0));
        assert_eq!(cos.evaluate(&vars()).unwrap(), 1.0);

        let tan = FunctionCall::new(Function::Tan, number(0.0));
        assert_eq!(tan.evaluate(&vars()).unwrap(), 0.0);

        let sqrt = FunctionCall::new(Function::Sqrt, number(16.0));
        assert_eq!(sqrt.evaluate(&vars()).unwrap(), 4.0);
    }

    #[test]
    fn function_call_errors() {
        let sqrt = FunctionCall::new(Function::Sqrt, number(-4.0));
        assert_eq!(
            sqrt.evaluate(&vars()).unwrap_err(),
            "Cannot take square root of negative number"
        );

        let tan = FunctionCall::new(Function::Tan, number(std::f64::consts::PI / 2.0));
        assert_eq!(tan.evaluate(&vars()).unwrap_err(), "Tangent undefined at this value");
    }

    #[test]
    fn function_call_to_string_and_precedence() {
        let sqrt = FunctionCall::new(Function::Sqrt, number(16.0));
        assert_eq!(sqrt.to_string(), "sqrt(16)");
        assert_eq!(sqrt.precedence(), 4);

        let sin = FunctionCall::new(Function::Sin, Box::new(VariableExpression::new("x")));
        assert_eq!(sin.to_string(), "sin(x)");
    }
}
