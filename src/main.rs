// main.rs - Main entry point for the calculator
// Combines structural patterns from Chapter 6 and behavioral patterns from Chapter 7

#![allow(unused)]

// Chapter 6 modules (structural patterns)
mod config;
mod expression;
mod token;

// Chapter 7 modules (behavioral patterns)
mod chain;
mod command;
mod mediator;
mod parser;
mod strategy;
mod template;

// Bridge module from Chapter 6
// This is a simplified version to show integration between chapters
mod bridge {
    use crate::expression::Expression;
    use std::collections::HashMap;

    // Display interface from Chapter 6 Bridge pattern
    pub trait Display {
        fn show_result(&self, result: f64);
        fn show_error(&self, error: &str);
        fn show_expression(&self, expression: &dyn Expression);
    }

    // Concrete display implementation
    pub struct ConsoleDisplay;

    impl Display for ConsoleDisplay {
        fn show_result(&self, result: f64) {
            println!("Result: {}", result);
        }

        fn show_error(&self, error: &str) {
            println!("Error: {}", error);
        }

        fn show_expression(&self, expression: &dyn Expression) {
            println!("Expression: {}", expression.to_string());
        }
    }

    // Evaluator bridge from Chapter 6
    pub struct Evaluator {
        strategy: Box<dyn EvaluationStrategy>,
    }

    pub trait EvaluationStrategy {
        fn evaluate(
            &self,
            expression: &dyn Expression,
            variables: &HashMap<String, f64>,
        ) -> Result<f64, String>;
    }

    pub struct StandardEvaluator;

    impl EvaluationStrategy for StandardEvaluator {
        fn evaluate(
            &self,
            expression: &dyn Expression,
            variables: &HashMap<String, f64>,
        ) -> Result<f64, String> {
            expression.evaluate(variables)
        }
    }

    impl Evaluator {
        pub fn new(strategy: Box<dyn EvaluationStrategy>) -> Self {
            Self { strategy }
        }

        pub fn evaluate(
            &self,
            expression: &dyn Expression,
            variables: &HashMap<String, f64>,
        ) -> Result<f64, String> {
            self.strategy.evaluate(expression, variables)
        }

        // TODO: maybe you want to return the old EvaluationStrategy using e.g. .take()
        pub fn change_strategy(&mut self, strategy: Box<dyn EvaluationStrategy>) {
            self.strategy = strategy;
        }
    }
}

// Adapter module from Chapter 6
// This is a simplified version to show integration between chapters
mod adapter {
    use crate::config::AngleMode;

    // Science operations interface from Chapter 6 Adapter pattern
    pub trait ScientificOperations {
        fn sin(&self, angle: f64) -> f64;
        fn cos(&self, angle: f64) -> f64;
        fn tan(&self, angle: f64) -> f64;
        fn log(&self, value: f64, base: f64) -> Result<f64, String>;
    }

    // Standard implementation using Rust's math functions
    pub struct StandardScientificOperations {
        pub angle_mode: AngleMode,
    }

    impl ScientificOperations for StandardScientificOperations {
        fn sin(&self, angle: f64) -> f64 {
            match self.angle_mode {
                AngleMode::Radians => angle.sin(),
                AngleMode::Degrees => (angle * std::f64::consts::PI / 180.0).sin(),
            }
        }

        fn cos(&self, angle: f64) -> f64 {
            match self.angle_mode {
                AngleMode::Radians => angle.cos(),
                AngleMode::Degrees => (angle * std::f64::consts::PI / 180.0).cos(),
            }
        }

        fn tan(&self, angle: f64) -> f64 {
            match self.angle_mode {
                AngleMode::Radians => angle.tan(),
                AngleMode::Degrees => (angle * std::f64::consts::PI / 180.0).tan(),
            }
        }

        fn log(&self, value: f64, base: f64) -> Result<f64, String> {
            if value <= 0.0 {
                return Err("Cannot take logarithm of non-positive number".to_string());
            }
            if base <= 0.0 || base == 1.0 {
                return Err("Invalid logarithm base".to_string());
            }

            Ok(value.log(base))
        }
    }

    // Adapter for an external library (simplified)
    pub struct ExternalLibraryAdapter {
        pub angle_mode: AngleMode,
    }

    impl ExternalLibraryAdapter {
        pub fn new(angle_mode: AngleMode) -> Self {
            Self { angle_mode }
        }
    }

    impl ScientificOperations for ExternalLibraryAdapter {
        fn sin(&self, angle: f64) -> f64 {
            // In a real adapter, this would call the external library
            match self.angle_mode {
                AngleMode::Radians => angle.sin(),
                AngleMode::Degrees => angle.to_radians().sin(),
            }
        }

        fn cos(&self, angle: f64) -> f64 {
            match self.angle_mode {
                AngleMode::Radians => angle.cos(),
                AngleMode::Degrees => angle.to_radians().cos(),
            }
        }

        fn tan(&self, angle: f64) -> f64 {
            match self.angle_mode {
                AngleMode::Radians => angle.tan(),
                AngleMode::Degrees => angle.to_radians().tan(),
            }
        }

        fn log(&self, value: f64, base: f64) -> Result<f64, String> {
            if value <= 0.0 || base <= 0.0 || base == 1.0 {
                return Err("Invalid logarithm arguments".to_string());
            }

            Ok(value.log(base))
        }
    }
}

use chain::{InputHandler, create_input_chain};
use command::CommandProcessor;
use mediator::CalculatorMediator;
use parser::ExpressionParser;
use std::io::{self, Write};

fn main() {
    println!("Correct Calculator - Chapter 7");
    println!("Incorporating structural patterns from Chapter 6");
    println!("Type expressions to evaluate, variables to set (x = 5),");
    println!("or commands (/help, /undo, /redo, /history, /clear, /exit)");

    // Set up the calculator components (combining Ch6 & Ch7 patterns)
    let mut processor = CommandProcessor::default();
    let parser = ExpressionParser::new();
    let input_chain = create_input_chain(parser);

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        // read input
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input, please try again");
            continue;
        }

        // check if user wants to exit
        let input = input.trim();
        if input == "/exit" {
            break;
        }

        // do the calculator stuff
        match input_chain.handle(input, &mut processor) {
            Ok(Some(result)) => println!("= {}", result),
            Ok(None) => {} // Command executed with no result to display
            Err(error) => println!("Error: {}", error),
        }
    }

    println!("Goodbye!");
}

// Example using the Mediator pattern
fn run_with_mediator() {
    println!("Correct Calculator with Mediator");

    // Create mediator system
    let mediator = mediator::create_mediator_system();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input, please try again");
            continue;
        }

        let input = input.trim();
        if input == "/exit" {
            break;
        }

        // Process through mediator
        let result = {
            let mut mediator = mediator.lock().unwrap();

            if let Some(cmd) = input.strip_prefix("/") {
                // Special commands
                match cmd {
                    "help" => {
                        println!("Commands: /help, /exit");
                        Ok(None)
                    }
                    _ => Err("Unknown command".to_string()),
                }
            } else if let Some((name, value_str)) = input.split_once('=') {
                // Variable assignment
                let name = name.trim();
                let value_str = value_str.trim();

                match mediator.evaluate(value_str) {
                    Ok(value) => {
                        mediator.set_variable(name, value);
                        Ok(Some(value))
                    }
                    Err(e) => Err(e),
                }
            } else {
                // Expression evaluation
                mediator.evaluate(input).map(Some)
            }
        };

        match result {
            Ok(Some(value)) => println!("= {}", value),
            Ok(None) => {}
            Err(error) => println!("Error: {}", error),
        }
    }

    println!("Goodbye!");
}

// Example using the Template Method pattern
fn run_with_template() {
    println!("Correct Calculator with Template Method");

    // Create evaluator using template method
    let evaluator = template::create_evaluator(true); // true for recursive descent

    let mut variables = std::collections::HashMap::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input, please try again");
            continue;
        }

        let input = input.trim();
        if input == "exit" {
            break;
        }

        // Process input
        match evaluator.evaluate(input, &variables) {
            Ok(result) => println!("= {}", result),
            Err(error) => println!("Error: {}", error),
        }
    }

    println!("Goodbye!");
}

// Example using Strategy pattern
fn run_with_strategy() {
    println!("Correct Calculator with Strategy Pattern");

    // Create evaluator with strategies
    let mut evaluator = strategy::create_standard_evaluator();
    let mut variables = std::collections::HashMap::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input, please try again");
            continue;
        }

        let input = input.trim();
        if input == "exit" {
            break;
        }

        if input == "switch_scientific" {
            evaluator = strategy::create_scientific_evaluator();
            println!("Switched to scientific mode");
            continue;
        }

        // Process input
        match evaluator.evaluate(input, &variables) {
            Ok(result) => {
                println!("= {}", evaluator.format_result(result));
                variables.insert("ans".to_string(), result);
            }
            Err(error) => println!("Error: {}", error),
        }
    }

    println!("Goodbye!");
}

#[cfg(test)]
mod tests {
    use super::adapter::ScientificOperations;
    use super::adapter::StandardScientificOperations;
    use super::bridge::{Display, Evaluator, StandardEvaluator};
    use super::config::AngleMode;
    use super::expression::{BinaryOperation, NumberExpression};
    use super::token::Operator;
    use std::collections::HashMap;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn standard_operations_radians() {
        let ops = StandardScientificOperations {
            angle_mode: AngleMode::Radians,
        };
        assert!(approx_eq(ops.sin(std::f64::consts::PI / 2.0), 1.0));
        assert!(approx_eq(ops.cos(0.0), 1.0));
        assert!(approx_eq(ops.tan(0.0), 0.0));
    }

    #[test]
    fn standard_operations_degrees() {
        let ops = StandardScientificOperations {
            angle_mode: AngleMode::Degrees,
        };
        assert!(approx_eq(ops.sin(30.0), 0.5));
        assert!(approx_eq(ops.cos(60.0), 0.5));
        assert!(approx_eq(ops.tan(45.0), 1.0));
    }

    #[test]
    fn standard_operations_log() {
        let ops = StandardScientificOperations {
            angle_mode: AngleMode::Radians,
        };
        assert_eq!(ops.log(8.0, 2.0).unwrap(), 3.0);
        assert!(ops.log(-1.0, 10.0).is_err());
        assert!(ops.log(10.0, 1.0).is_err());
        assert!(ops.log(10.0, 0.0).is_err());
    }

    #[test]
    fn external_library_adapter_matches_standard_results() {
        let standard = super::adapter::ExternalLibraryAdapter::new(AngleMode::Degrees);
        assert!(approx_eq(standard.sin(30.0), 0.5));

        let radians = super::adapter::ExternalLibraryAdapter::new(AngleMode::Radians);
        assert!(approx_eq(radians.sin(std::f64::consts::PI / 2.0), 1.0));
        assert_eq!(radians.log(8.0, 2.0).unwrap(), 3.0);
        assert!(radians.log(-1.0, 2.0).is_err());
    }

    #[test]
    fn bridge_console_display_renders() {
        let display = super::bridge::ConsoleDisplay;
        display.show_result(42.0);
        display.show_error("boom");
        display.show_expression(&NumberExpression::new(7.0));
    }

    #[test]
    fn bridge_evaluator_evaluates_and_swaps_strategy() {
        let multiply = Box::new(BinaryOperation::new(
            Box::new(NumberExpression::new(3.0)),
            Box::new(NumberExpression::new(4.0)),
            Operator::Multiply,
        ));
        let add = Box::new(BinaryOperation::new(
            Box::new(NumberExpression::new(2.0)),
            multiply,
            Operator::Add,
        ));
        let variables = HashMap::new();
        let mut evaluator = Evaluator::new(Box::new(StandardEvaluator));
        assert_eq!(evaluator.evaluate(&*add, &variables).unwrap(), 14.0);

        evaluator.change_strategy(Box::new(StandardEvaluator));
        assert_eq!(evaluator.evaluate(&*add, &variables).unwrap(), 14.0);
    }
}
