// mediator.rs - Mediator pattern implementation

// Mediator Pattern: Coordinating Components
//
// The Mediator pattern addresses a common challenge in complex systems: how to enable
// communication between multiple components without creating tight coupling between them. As
// systems grow, direct communication between components leads to a tangled web of dependencies
// that becomes difficult to maintain and extend. The Mediator pattern solves this by introducing a
// central coordinator that manages all interactions between components.
//
// In essence, the Mediator pattern defines an object that encapsulates how a set of objects interact.
// This promotes loose coupling by keeping objects from referring to each other explicitly, allowing
// them to focus on their core responsibilities.
//
// The Mediator pattern breaks these direct dependencies. Each component knows only about the
// mediator, and the mediator knows about all components.
//
// We'll implement this pattern in three parts: first, defining the mediator interface and event types,
// then showing how components interact with the mediator, and finally implementing the concrete
// mediator that orchestrates everything. This is a pattern that works cleanly for one service, but could
// also adapt to a multi-service architecture through a communication interface.

// 1. Define the components
//   - A. EvaluationComponent
//   - B. VariableStorage
//   - C. ConsoleDisplay
// 2. Define the Mediator that completely owns the components

use crate::config::AngleMode;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Events that can be sent through the mediator
pub enum CalculatorEvent {
    ResultComputed(f64),
    VariableChanged(String, f64),
    ModeChanged(String),
    DisplayUpdate(String),
    ErrorOccurred(String),
}

// Mediator interface
pub trait CalculatorMediator: Send + Sync {
    // The notify method is the core of the mediator's functionality. Components call this method to
    // announce events, and the mediator decides how to distribute them.
    fn notify(&mut self, sender: &str, event: CalculatorEvent);

    // Components don't access each other directly; instead, they ask the mediator.
    fn get_result(&self) -> Option<f64>;
    fn get_variable(&self, name: &str) -> Option<f64>;
    fn get_all_variables(&self) -> HashMap<String, f64>;
    fn set_variable(&mut self, name: &str, value: f64);
    fn evaluate(&mut self, expression: &str) -> Result<f64, String>;
    fn change_angle_mode(&mut self, mode: AngleMode);
}

// //////////////////////// //
// 1. Define the components //
// //////////////////////// //

// ====================== //
// A. EvaluationComponent //
// ====================== //

// Component that handles evaluation. It is passive: the mediator hands it the expression and the
// current variables, and it returns the result. It never touches the mediator itself, which keeps the
// mediator from deadlocking when it calls back into its own components.
pub struct EvaluationComponent {
    parser: crate::parser::ExpressionParser,
}

impl EvaluationComponent {
    pub fn new() -> Self {
        Self {
            parser: crate::parser::ExpressionParser::new(),
        }
    }

    pub fn evaluate(
        &self,
        expression: &str,
        variables: &HashMap<String, f64>,
    ) -> Result<f64, String> {
        let expr = self.parser.parse(expression)?;
        expr.evaluate(variables)
    }
}

// ================== //
// B. VariableStorage //
// ================== //

// Component that manages variables.
pub struct VariableStorage {
    variables: HashMap<String, f64>,
}

impl VariableStorage {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn set_variable(&mut self, name: &str, value: f64) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn get_variable(&self, name: &str) -> Option<f64> {
        self.variables.get(name).copied()
    }

    pub fn get_all_variables(&self) -> HashMap<String, f64> {
        self.variables.clone()
    }

    pub fn clear(&mut self) {
        self.variables.clear();
    }
}

// ================= //
// C. ConsoleDisplay //
// ================= //

// Display component interface
pub trait Display: Send + Sync {
    fn show_result(&mut self, result: f64);
    fn show_message(&mut self, message: &str);
    fn show_error(&mut self, error: &str);
    fn clear(&mut self);
}

// Console display component
pub struct ConsoleDisplay;

impl ConsoleDisplay {
    pub fn new() -> Self {
        Self
    }
}

impl Display for ConsoleDisplay {
    fn show_result(&mut self, result: f64) {
        println!("Result: {}", result);
    }

    fn show_message(&mut self, message: &str) {
        println!("{}", message);
    }

    fn show_error(&mut self, error: &str) {
        println!("Error: {}", error);
    }

    fn clear(&mut self) {
        // Clear console (platform-specific)
        // For simplicity, just print some newlines
        println!("\n\n\n\n\n");
    }
}

// ========================================================== //
// 2. Define the Mediator that completely owns the components //
// ========================================================== //

// The concrete mediator knows about all components and orchestrates their interactions. It's the
// one place in the system with knowledge of the overall architecture.

// Concrete mediator implementation
pub struct CalculatorMediatorImpl {
    // Components References
    //
    // Using Option for component references enables flexible initialization. Components can be
    // registered after the mediator is created, and some components might be optional.
    evaluator: Option<Arc<EvaluationComponent>>,
    variables: Option<Arc<Mutex<VariableStorage>>>,
    display: Option<Arc<Mutex<dyn Display>>>,

    last_result: Option<f64>,
    angle_mode: AngleMode,
}

impl CalculatorMediatorImpl {
    pub fn new() -> Self {
        Self {
            evaluator: None,
            variables: None,
            display: None,
            last_result: None,
            angle_mode: AngleMode::Radians,
        }
    }

    // Registration Methods

    pub fn set_evaluator(&mut self, evaluator: Arc<EvaluationComponent>) {
        self.evaluator = Some(evaluator);
    }

    pub fn set_variables(&mut self, variables: Arc<Mutex<VariableStorage>>) {
        self.variables = Some(variables);
    }

    pub fn set_display(&mut self, display: Arc<Mutex<dyn Display>>) {
        self.display = Some(display);
    }
}

impl CalculatorMediator for CalculatorMediatorImpl {
    fn notify(&mut self, sender: &str, event: CalculatorEvent) {
        let _ = sender;
        match event {
            CalculatorEvent::ResultComputed(result) => {
                self.last_result = Some(result);

                if let Some(display) = &self.display {
                    let mut display = display.lock().unwrap();
                    display.show_result(result);
                }
            }
            CalculatorEvent::VariableChanged(name, value) => {
                if let Some(display) = &self.display {
                    let mut display = display.lock().unwrap();
                    display.show_message(&format!("Variable {} set to {}", name, value));
                }
            }
            CalculatorEvent::ModeChanged(mode) => {
                if let Some(display) = &self.display {
                    let mut display = display.lock().unwrap();
                    display.show_message(&format!("Mode changed to {}", mode));
                }
            }
            CalculatorEvent::DisplayUpdate(message) => {
                if let Some(display) = &self.display {
                    let mut display = display.lock().unwrap();
                    display.show_message(&message);
                }
            }
            CalculatorEvent::ErrorOccurred(error) => {
                if let Some(display) = &self.display {
                    let mut display = display.lock().unwrap();
                    display.show_error(&error);
                }
            }
        }
    }

    fn get_result(&self) -> Option<f64> {
        self.last_result
    }

    fn get_variable(&self, name: &str) -> Option<f64> {
        if let Some(variables) = &self.variables {
            let variables = variables.lock().unwrap();
            variables.get_variable(name)
        } else {
            None
        }
    }

    fn get_all_variables(&self) -> HashMap<String, f64> {
        if let Some(variables) = &self.variables {
            let variables = variables.lock().unwrap();
            variables.get_all_variables()
        } else {
            HashMap::new()
        }
    }

    fn set_variable(&mut self, name: &str, value: f64) {
        if let Some(variables) = &self.variables {
            let mut variables = variables.lock().unwrap();
            variables.set_variable(name, value);
        }

        self.notify(
            "variables",
            CalculatorEvent::VariableChanged(name.to_string(), value),
        );
    }

    fn evaluate(&mut self, expression: &str) -> Result<f64, String> {
        let evaluator = self
            .evaluator
            .as_ref()
            .ok_or_else(|| "Evaluator not initialized".to_string())?;

        let variables = self.get_all_variables();
        let result = evaluator.evaluate(expression, &variables)?;

        self.notify(
            "evaluator",
            CalculatorEvent::ResultComputed(result),
        );

        Ok(result)
    }

    fn change_angle_mode(&mut self, mode: AngleMode) {
        self.angle_mode = mode.clone(); // Clone to avoid the moved value error

        let mode_str = match mode {
            AngleMode::Degrees => "Degrees",
            AngleMode::Radians => "Radians",
        };

        self.notify(
            "mediator",
            CalculatorEvent::ModeChanged(mode_str.to_string()),
        );
    }
}

// Helper function to set up mediator system
pub fn create_mediator_system() -> Arc<Mutex<CalculatorMediatorImpl>> {
    // Create mediator as a concrete type
    let mediator = Arc::new(Mutex::new(CalculatorMediatorImpl::new()));

    // Create components and register them with the mediator
    {
        let mut mediator_lock = mediator.lock().unwrap();
        mediator_lock.set_evaluator(Arc::new(EvaluationComponent::new()));
        mediator_lock.set_variables(Arc::new(Mutex::new(VariableStorage::new())));
        mediator_lock.set_display(Arc::new(Mutex::new(ConsoleDisplay::new())));
    }

    mediator
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AngleMode;
    use std::collections::HashMap;

    #[derive(Default)]
    struct TestDisplay {
        messages: Mutex<Vec<String>>,
    }

    impl Display for TestDisplay {
        fn show_result(&mut self, result: f64) {
            self.messages.lock().unwrap().push(format!("result:{}", result));
        }

        fn show_message(&mut self, message: &str) {
            self.messages.lock().unwrap().push(format!("msg:{}", message));
        }

        fn show_error(&mut self, error: &str) {
            self.messages.lock().unwrap().push(format!("err:{}", error));
        }

        fn clear(&mut self) {
            self.messages.lock().unwrap().clear();
        }
    }

    fn setup() -> (Arc<Mutex<CalculatorMediatorImpl>>, Arc<Mutex<TestDisplay>>) {
        let mediator = Arc::new(Mutex::new(CalculatorMediatorImpl::new()));
        let display = Arc::new(Mutex::new(TestDisplay::default()));

        {
            let mut mediator = mediator.lock().unwrap();
            mediator.set_evaluator(Arc::new(EvaluationComponent::new()));
            mediator.set_variables(Arc::new(Mutex::new(VariableStorage::new())));
            mediator.set_display(display.clone());
        }

        (mediator, display)
    }

    #[test]
    fn mediator_evaluates_expressions() {
        let (mediator, display) = setup();

        let result = mediator.lock().unwrap().evaluate("2 + 3").unwrap();
        assert_eq!(result, 5.0);
        assert_eq!(mediator.lock().unwrap().get_result(), Some(5.0));

        let display = display.lock().unwrap();
        let messages = display.messages.lock().unwrap();
        assert!(messages.contains(&"result:5".to_string()));
    }

    #[test]
    fn mediator_evaluates_expressions_with_variables() {
        let (mediator, _display) = setup();

        {
            let mut mediator = mediator.lock().unwrap();
            mediator.set_variable("x", 4.0);
        }

        let result = mediator.lock().unwrap().evaluate("x + 1").unwrap();
        assert_eq!(result, 5.0);
    }

    #[test]
    fn mediator_errors_without_registered_evaluator() {
        let mediator = Arc::new(Mutex::new(CalculatorMediatorImpl::new()));
        assert_eq!(
            mediator.lock().unwrap().evaluate("1 + 1").unwrap_err(),
            "Evaluator not initialized"
        );
    }

    #[test]
    fn mediator_manages_variables() {
        let (mediator, display) = setup();

        {
            let mut mediator = mediator.lock().unwrap();
            mediator.set_variable("x", 5.0);
            assert_eq!(mediator.get_variable("x"), Some(5.0));
            assert_eq!(mediator.get_all_variables().get("x"), Some(&5.0));
        }

        let display = display.lock().unwrap();
        let messages = display.messages.lock().unwrap();
        assert!(messages.contains(&"msg:Variable x set to 5".to_string()));
    }

    #[test]
    fn mediator_returns_none_for_unset_variables() {
        let mediator = Arc::new(Mutex::new(CalculatorMediatorImpl::new()));
        assert_eq!(mediator.lock().unwrap().get_variable("x"), None);
    }

    #[test]
    fn change_angle_mode_is_safe_without_display() {
        let mediator = Arc::new(Mutex::new(CalculatorMediatorImpl::new()));
        let mut mediator = mediator.lock().unwrap();
        mediator.change_angle_mode(AngleMode::Degrees);
    }

    #[test]
    fn create_mediator_system_initializes_all_components() {
        let system = create_mediator_system();
        assert_eq!(system.lock().unwrap().get_result(), None);
        assert_eq!(system.lock().unwrap().get_variable("x"), None);
    }
}