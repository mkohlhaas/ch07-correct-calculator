// command.rs - Command pattern implementation

use crate::expression::Expression;
use std::{collections::HashMap, time::SystemTime};

// ============================================ //
// 1. The Receiver: Holds the application state //
// ============================================ //

// Helper struct: Represents a complete calculation
#[derive(Debug, Clone)]
pub struct Calculation {
    pub expression: String,
    pub result: f64,
    pub timestamp: SystemTime,
}

// the receiver
#[derive(Default)]
pub struct Calculator {
    pub variables: HashMap<String, f64>,
    pub calc_history: Vec<Calculation>,
    pub last_result: Option<f64>,
}

impl Calculator {
    pub fn set_variable(&mut self, name: &str, value: f64) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn get_variable(&self, name: &str) -> Option<f64> {
        self.variables.get(name).copied()
    }

    pub fn clear_variable(&mut self, name: &str) {
        self.variables.remove(name);
    }

    pub fn set_last_result(&mut self, result: f64) {
        self.last_result = Some(result);
    }

    pub fn store_calculation(&mut self, expression: String, result: f64) {
        let calculation = Calculation {
            expression,
            result,
            timestamp: SystemTime::now(),
        };
        self.calc_history.push(calculation);
        self.last_result = Some(result);
    }
}

// =============================================== //
// 2. The Command Trait: Decouples execution logic //
// =============================================== //

pub trait Command {
    fn execute(&mut self, calculator: &mut Calculator) -> Result<Option<f64>, String>;
    fn undo(&self, calculator: &mut Calculator) -> Result<(), String>;
    fn description(&self) -> String;
}

// ==================== //
// 3. Concrete commands //
// ==================== //

// ------------------------ //
// A. Evaluates expressions //
// ------------------------ //

pub struct EvaluateCommand {
    expression: String,
    expr_tree: Box<dyn Expression>,
    previous_result: Option<f64>,
}

impl EvaluateCommand {
    pub fn new(expression: String, expr_tree: Box<dyn Expression>) -> Self {
        Self {
            expression,
            expr_tree,
            previous_result: None,
        }
    }
}

impl Command for EvaluateCommand {
    fn execute(&mut self, calculator: &mut Calculator) -> Result<Option<f64>, String> {
        self.previous_result = calculator.last_result;

        let result = self.expr_tree.evaluate(&calculator.variables)?;
        calculator.store_calculation(self.expression.clone(), result);

        Ok(Some(result))
    }

    fn undo(&self, calculator: &mut Calculator) -> Result<(), String> {
        // Remove the last entry from history
        if !calculator.calc_history.is_empty() {
            calculator.calc_history.pop();
        }

        // Restore previous result
        calculator.last_result = self.previous_result;

        Ok(())
    }

    fn description(&self) -> String {
        format!("Evaluate: {}", self.expression)
    }
}

// ----------------- //
// B. Sets variables //
// ----------------- //

pub struct SetVariableCommand {
    name: String,
    value: f64,
    previous_value: Option<f64>,
}

impl SetVariableCommand {
    pub fn new(name: String, value: f64) -> Self {
        Self {
            name,
            value,
            previous_value: None,
        }
    }
}

impl Command for SetVariableCommand {
    fn execute(&mut self, calculator: &mut Calculator) -> Result<Option<f64>, String> {
        self.previous_value = calculator.get_variable(&self.name);
        calculator.set_variable(&self.name, self.value);
        Ok(None)
    }

    fn undo(&self, calculator: &mut Calculator) -> Result<(), String> {
        match self.previous_value {
            Some(value) => {
                calculator.set_variable(&self.name, value);
                Ok(())
            }
            None => {
                calculator.clear_variable(&self.name);
                Ok(())
            }
        }
    }

    fn description(&self) -> String {
        format!("Set: {} = {}", self.name, self.value)
    }
}

// ----------------------- //
// C. Clears all variables //
// ----------------------- //

pub struct ClearVariablesCommand {
    previous_variables: Option<HashMap<String, f64>>,
}

impl ClearVariablesCommand {
    pub fn new() -> Self {
        Self {
            previous_variables: None,
        }
    }
}

impl Command for ClearVariablesCommand {
    fn execute(&mut self, calculator: &mut Calculator) -> Result<Option<f64>, String> {
        self.previous_variables = Some(calculator.variables.clone());
        calculator.variables.clear();
        Ok(None)
    }

    fn undo(&self, calculator: &mut Calculator) -> Result<(), String> {
        if let Some(vars) = &self.previous_variables {
            calculator.variables = vars.clone();
            Ok(())
        } else {
            Err("No previous variables state saved".to_string())
        }
    }

    fn description(&self) -> String {
        "Clear all variables".to_string()
    }
}

// ================================================================= //
// 4. The Command Processor: Manages history and schedules execution //
// ================================================================= //

#[derive(Default)]
pub struct CommandProcessor {
    calculator: Calculator,
    undo_stack: Vec<Box<dyn Command>>, // executed, undoable commands
    redo_stack: Vec<Box<dyn Command>>, // undone commands pending re-application
}

impl CommandProcessor {
    pub fn execute(&mut self, mut command: Box<dyn Command>) -> Result<Option<f64>, String> {
        let result = command.execute(&mut self.calculator)?;
        self.undo_stack.push(command);
        self.redo_stack.clear(); // Clear redo stack after new command
        Ok(result)
    }

    pub fn undo(&mut self) -> Result<(), String> {
        if let Some(command) = self.undo_stack.pop() {
            command.undo(&mut self.calculator)?;
            self.redo_stack.push(command);
            Ok(())
        } else {
            Err("Nothing to undo".to_string())
        }
    }

    pub fn redo(&mut self) -> Result<(), String> {
        if let Some(mut command) = self.redo_stack.pop() {
            command.execute(&mut self.calculator)?;
            self.undo_stack.push(command);
            Ok(())
        } else {
            Err("Nothing to redo".to_string())
        }
    }

    pub fn history(&self) -> Vec<String> {
        self.undo_stack
            .iter()
            .map(|cmd| cmd.description())
            .collect()
    }

    pub fn get_calculator(&self) -> &Calculator {
        &self.calculator
    }

    pub fn get_calculator_mut(&mut self) -> &mut Calculator {
        &mut self.calculator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ExpressionParser;

    fn parse(expression: &str) -> Box<dyn Expression> {
        ExpressionParser::new().parse(expression).unwrap()
    }

    #[test]
    fn calculator_default_is_empty() {
        let calculator = Calculator::default();
        assert!(calculator.variables.is_empty());
        assert!(calculator.calc_history.is_empty());
        assert_eq!(calculator.last_result, None);
    }

    #[test]
    fn calculator_manages_variables() {
        let mut calculator = Calculator::default();
        assert_eq!(calculator.get_variable("x"), None);

        calculator.set_variable("x", 5.0);
        assert_eq!(calculator.get_variable("x"), Some(5.0));

        calculator.clear_variable("x");
        assert_eq!(calculator.get_variable("x"), None);
    }

    #[test]
    fn calculator_stores_calculations() {
        let mut calculator = Calculator::default();
        calculator.store_calculation("2 + 3".to_string(), 5.0);

        assert_eq!(calculator.last_result, Some(5.0));
        assert_eq!(calculator.calc_history.len(), 1);
        assert_eq!(calculator.calc_history[0].expression, "2 + 3");
        assert_eq!(calculator.calc_history[0].result, 5.0);
    }

    #[test]
    fn evaluate_command_executes_and_undoes() {
        let mut processor = CommandProcessor::default();
        let command = Box::new(EvaluateCommand::new("2 + 3".to_string(), parse("2 + 3")));

        assert_eq!(processor.execute(command).unwrap(), Some(5.0));
        assert_eq!(processor.get_calculator().last_result, Some(5.0));
        assert_eq!(processor.history().len(), 1);

        processor.undo().unwrap();
        assert_eq!(processor.get_calculator().last_result, None);
        assert!(processor.history().is_empty());
    }

    #[test]
    fn evaluate_command_restores_previous_result_on_undo() {
        let mut processor = CommandProcessor::default();

        let first = Box::new(EvaluateCommand::new("3 + 4".to_string(), parse("3 + 4")));
        processor.execute(first).unwrap();
        assert_eq!(processor.get_calculator().last_result, Some(7.0));

        let second = Box::new(EvaluateCommand::new("10 - 1".to_string(), parse("10 - 1")));
        processor.execute(second).unwrap();
        assert_eq!(processor.get_calculator().last_result, Some(9.0));

        processor.undo().unwrap();
        assert_eq!(processor.get_calculator().last_result, Some(7.0));
        assert_eq!(processor.get_calculator().calc_history.len(), 1);
    }

    #[test]
    fn undo_and_redo_round_trip() {
        let mut processor = CommandProcessor::default();
        let command = Box::new(EvaluateCommand::new("7 * 6".to_string(), parse("7 * 6")));
        processor.execute(command).unwrap();

        processor.undo().unwrap();
        assert_eq!(processor.get_calculator().last_result, None);

        processor.redo().unwrap();
        assert_eq!(processor.get_calculator().last_result, Some(42.0));
        assert_eq!(processor.history().len(), 1);
    }

    #[test]
    fn undo_redo_error_when_empty() {
        let mut processor = CommandProcessor::default();
        assert_eq!(processor.undo().unwrap_err(), "Nothing to undo");
        assert_eq!(processor.redo().unwrap_err(), "Nothing to redo");
    }

    #[test]
    fn new_command_clears_redo_stack() {
        let mut processor = CommandProcessor::default();
        let command = Box::new(EvaluateCommand::new("1 + 1".to_string(), parse("1 + 1")));
        processor.execute(command).unwrap();
        processor.undo().unwrap();
        assert!(processor.redo().is_ok());

        let other = Box::new(EvaluateCommand::new("2 + 2".to_string(), parse("2 + 2")));
        processor.execute(other).unwrap();
        assert_eq!(processor.redo().unwrap_err(), "Nothing to redo");
    }

    #[test]
    fn set_variable_command_undo_restores_previous_value() {
        let mut processor = CommandProcessor::default();

        let set = Box::new(SetVariableCommand::new("x".to_string(), 5.0));
        processor.execute(set).unwrap();
        assert_eq!(processor.get_calculator().get_variable("x"), Some(5.0));

        let overwrite = Box::new(SetVariableCommand::new("x".to_string(), 9.0));
        processor.execute(overwrite).unwrap();
        assert_eq!(processor.get_calculator().get_variable("x"), Some(9.0));

        processor.undo().unwrap();
        assert_eq!(processor.get_calculator().get_variable("x"), Some(5.0));
    }

    #[test]
    fn set_variable_command_undo_removes_new_variable() {
        let mut processor = CommandProcessor::default();
        let set = Box::new(SetVariableCommand::new("x".to_string(), 5.0));
        processor.execute(set).unwrap();

        processor.undo().unwrap();
        assert_eq!(processor.get_calculator().get_variable("x"), None);
    }

    #[test]
    fn clear_variables_command_undo_restores_state() {
        let mut processor = CommandProcessor::default();
        processor.get_calculator_mut().set_variable("x", 5.0);
        processor.get_calculator_mut().set_variable("y", 6.0);

        let clear = Box::new(ClearVariablesCommand::new());
        processor.execute(clear).unwrap();
        assert!(processor.get_calculator().variables.is_empty());

        processor.undo().unwrap();
        assert_eq!(processor.get_calculator().get_variable("x"), Some(5.0));
        assert_eq!(processor.get_calculator().get_variable("y"), Some(6.0));
    }

    #[test]
    fn history_records_descriptions() {
        let mut processor = CommandProcessor::default();
        processor
            .execute(Box::new(EvaluateCommand::new(
                "2 + 3".to_string(),
                parse("2 + 3"),
            )))
            .unwrap();
        processor
            .execute(Box::new(SetVariableCommand::new("x".to_string(), 5.0)))
            .unwrap();

        let history = processor.history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0], "Evaluate: 2 + 3");
        assert_eq!(history[1], "Set: x = 5");
    }

    #[test]
    fn command_execution_with_undefined_variable_fails() {
        let mut processor = CommandProcessor::default();
        let command = Box::new(EvaluateCommand::new("x + 1".to_string(), parse("x + 1")));
        assert_eq!(
            processor.execute(command).unwrap_err(),
            "Undefined variable: x"
        );
    }
}
