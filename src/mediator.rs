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

// All components have a `mediator` field.

// ====================== //
// A. EvaluationComponent //
// ====================== //

// Component that handles evaluation
pub struct EvaluationComponent {
    // Arc provides shared ownership so multiple components can hold references. Mutex ensures safe
    // mutable access. Only one component can interact with the mediator at a time. The dyn keyword
    // indicates a trait object, enabling runtime polymorphism if we need different mediator
    // implementations.
    mediator: Arc<Mutex<dyn CalculatorMediator>>,
    parser: crate::parser::ExpressionParser,
}

impl EvaluationComponent {
    pub fn new(mediator: Arc<Mutex<dyn CalculatorMediator>>) -> Self {
        Self {
            mediator,
            parser: crate::parser::ExpressionParser::new(),
        }
    }

    pub fn evaluate(&self, expression: &str) -> Result<f64, String> {
        // Parse expression
        let expr = self.parser.parse(expression)?;

        // Get variables from mediator
        let variables = {
            let mediator = self.mediator.lock().unwrap();
            mediator.get_all_variables()
        };

        // Evaluate
        let result = expr.evaluate(&variables)?;

        // Notify mediator of result
        {
            let mut mediator = self.mediator.lock().unwrap();
            mediator.notify("evaluator", CalculatorEvent::ResultComputed(result));
        }

        Ok(result)
    }
}

// ================== //
// B. VariableStorage //
// ================== //

// Component that manages variables
pub struct VariableStorage {
    mediator: Arc<Mutex<dyn CalculatorMediator>>,
    variables: HashMap<String, f64>,
}

impl VariableStorage {
    pub fn new(mediator: Arc<Mutex<dyn CalculatorMediator>>) -> Self {
        Self {
            mediator,
            variables: HashMap::new(),
        }
    }

    pub fn set_variable(&mut self, name: &str, value: f64) {
        self.variables.insert(name.to_string(), value);

        // Notify mediator
        let mut mediator = self.mediator.lock().unwrap();
        mediator.notify(
            "variables",
            CalculatorEvent::VariableChanged(name.to_string(), value),
        );
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
pub struct ConsoleDisplay {
    mediator: Arc<Mutex<dyn CalculatorMediator>>,
}

impl ConsoleDisplay {
    pub fn new(mediator: Arc<Mutex<dyn CalculatorMediator>>) -> Self {
        Self { mediator }
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
    }

    fn evaluate(&mut self, expression: &str) -> Result<f64, String> {
        if let Some(evaluator) = &self.evaluator {
            evaluator.evaluate(expression)
        } else {
            Err("Evaluator not initialized".to_string())
        }
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

    // Create components
    let evaluator = Arc::new(EvaluationComponent::new(mediator.clone()));
    let variables = Arc::new(Mutex::new(VariableStorage::new(mediator.clone())));
    let display = Arc::new(Mutex::new(ConsoleDisplay::new(mediator.clone())));

    // Register components with mediator
    {
        let mut mediator_lock = mediator.lock().unwrap();
        mediator_lock.set_evaluator(evaluator);
        mediator_lock.set_variables(variables);
        mediator_lock.set_display(display);
    }

    mediator
}
