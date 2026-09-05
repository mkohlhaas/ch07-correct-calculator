// command.rs - Command pattern implementation

use crate::expression::Expression;
use std::{collections::HashMap, time::SystemTime};

// ============================================ //
// 1. The Receiver: Holds the application state //
// ============================================ //

pub struct Calculator {
    pub variables: HashMap<String, f64>,
    pub calc_history: Vec<Calculation>,
    pub last_result: Option<f64>,
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            calc_history: Vec::new(),
            last_result: None,
        }
    }

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

// Helper struct: Represents a complete calculation
#[derive(Debug, Clone)]
pub struct Calculation {
    pub expression: String,
    pub result: f64,
    pub timestamp: SystemTime,
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

// ======================== //
// A. Evaluates expressions //
// ======================== //

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

// ================= //
// B. Sets variables //
// ================= //

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

// ======================= //
// C. Clears all variables //
// ======================= //

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

pub struct CommandProcessor {
    calculator: Calculator,
    cmd_history: Vec<Box<dyn Command>>,
    undo_stack: Vec<Box<dyn Command>>,
}

impl CommandProcessor {
    pub fn new() -> Self {
        Self {
            calculator: Calculator::new(),
            cmd_history: Vec::new(),
            undo_stack: Vec::new(),
        }
    }

    pub fn execute(&mut self, mut command: Box<dyn Command>) -> Result<Option<f64>, String> {
        let result = command.execute(&mut self.calculator)?;
        self.cmd_history.push(command);
        self.undo_stack.clear(); // Clear redo stack after new command
        Ok(result)
    }

    pub fn undo(&mut self) -> Result<(), String> {
        if let Some(command) = self.cmd_history.pop() {
            command.undo(&mut self.calculator)?;
            self.undo_stack.push(command);
            Ok(())
        } else {
            Err("Nothing to undo".to_string())
        }
    }

    pub fn redo(&mut self) -> Result<(), String> {
        if let Some(mut command) = self.undo_stack.pop() {
            command.execute(&mut self.calculator)?;
            self.cmd_history.push(command);
            Ok(())
        } else {
            Err("Nothing to redo".to_string())
        }
    }

    pub fn history(&self) -> Vec<String> {
        self.cmd_history
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
