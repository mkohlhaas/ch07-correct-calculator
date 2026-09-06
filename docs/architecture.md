# Architecture

This document shows three views of the calculator's architecture as ASCII diagrams:

- [Diagram A: Runtime flow](#diagram-a-runtime-flow) - what `main` runs and which modules each runner touches
- [Diagram B: Module dependencies](#diagram-b-module-dependencies) - which module uses which
- [Diagram C: Pattern roles and structures](#diagram-c-pattern-roles-and-structures) - the concrete Rust types behind each design pattern

## Diagram A: Runtime flow

```
                          main()
                           |
                (menu: 1..4 | exit)
                            |
        +-------------------+------------------+------------------+---------------+
        | choice 1            choice 2           choice 3           choice 4
        v                     v                  v                  v
run_with_command_processor  run_with_mediator  run_with_template  run_with_strategy
        |                     |                  |                  |
        v                     v                  v                  v
      chain.rs            mediator.rs        template.rs        strategy.rs
      command.rs           parser.rs           parser.rs         expression.rs
      parser.rs            config.rs          expression.rs      token.rs
                                              token.rs
```

## Diagram B: Module dependencies

```
token.rs      (leaf, no crate dependencies)
expression.rs --> token.rs
parser.rs     --> expression.rs, token.rs
command.rs    --> expression.rs
chain.rs      --> command.rs, parser.rs
mediator.rs   --> config.rs, parser.rs
template.rs   --> expression.rs, token.rs, parser.rs
strategy.rs   --> expression.rs, token.rs
config.rs     --> token.rs

inline modules (in main.rs, used by tests only):
  bridge.rs   --> expression.rs
  adapter.rs  --> config.rs
```

## Diagram C: Pattern roles and structures

### Command (command.rs)

```
Client            : CommandProcessor { calculator,                                        undo_stack: Vec<Box<dyn Command>>,
                                       redo_stack: Vec<Box<dyn Command>> }
Command interface : trait Command { execute(), undo(), description() }
                     |-- EvaluateCommand { expression, expr_tree: Box<dyn Expression>,
                     |                     previous_result }
                     |-- SetVariableCommand { name, value, previous_value }
                     |-- ClearVariablesCommand { previous_variables }
Receiver          : Calculator { variables: HashMap<String, f64>,
                                 calc_history: Vec<Calculation>,
                                 last_result: Option<f64> }
                     Calculation { expression, result, timestamp }
```

### Chain of Responsibility (chain.rs)

```
trait InputHandler { handle(input, processor), set_next(next) }
  |-- BaseHandler { next: Option<Box<dyn InputHandler>> }
  |-- CommandHandler { base: BaseHandler }
  |-- VariableAssignmentHandler { base, parser: ExpressionParser }
  |-- ExpressionHandler { base, parser: ExpressionParser }

Wiring : create_input_chain(parser)
         command_handler -> variable_assignment_handler -> expression_handler
```

### Mediator (mediator.rs)

```
trait CalculatorMediator { notify(), get_result(), get_variable(), get_all_variables(),
                           set_variable(), evaluate(), change_angle_mode() }
  |-- CalculatorMediatorImpl { evaluator: Option<Arc<EvaluationComponent>>,
                               variables: Option<Arc<Mutex<VariableStorage>>>,
                               display: Option<Arc<Mutex<dyn Display>>>,
                               last_result, angle_mode }

components:
  EvaluationComponent { parser }        (passive: parses/evaluates on request)
  VariableStorage     { variables }     (passive: variable map)
  ConsoleDisplay (+)                    (impl Display, receives events)
  trait Display { show_result(), show_message(), show_error(), clear() }

factory : create_mediator_system() -> Arc<Mutex<CalculatorMediatorImpl>>
```

### Strategy (strategy.rs)

```
Context        : ExpressionEvaluatorContext { evaluation_strategy,
                                              precision_strategy }
Evaluation     : trait EvaluationStrategy
                   |-- RecursiveDescentStrategy { tokenizer }
                   |-- ShuntingYardStrategy { tokenizer }
Tokenization   : trait TokenizationStrategy
                   |-- SimpleTokenizer
Precision      : trait PrecisionStrategy { format(), round() }
                   |-- StandardPrecision { decimal_places }
                   |-- ScientificPrecision { significant_figures }
factories      : create_standard_evaluator(), create_scientific_evaluator()
```

### Template Method (template.rs)

```
trait ExpressionEvaluator
  evaluate()           <- template: tokenize -> validate -> parse -> evaluate_parsed
  tokenize(), validate_tokens()   <- shared default steps
  parse(), evaluate_parsed()      <- abstract, overridden by subclasses
    |-- RecursiveDescentEvaluator
    |-- ShuntingYardEvaluator
factory : create_evaluator(use_recursive_descent)
```

### Composite (expression.rs)

```
trait Expression { evaluate(variables), to_string(), precedence() }
  |-- leaves    : NumberExpression { value }, VariableExpression { name }
  |-- composites: BinaryOperation { left, right: Box<dyn Expression>, operator },
                  FunctionCall    { function, argument: Box<dyn Expression> }
```

### Bridge (inline module in main.rs, tests only)

```
trait Display { show_result(), show_error(), show_expression() }
  |-- ConsoleDisplay
trait EvaluationStrategy { evaluate() }
  |-- StandardEvaluator
abstraction : Evaluator { strategy: Box<dyn EvaluationStrategy> }
```

### Adapter (inline module in main.rs, tests only)

```
trait ScientificOperations { sin(), cos(), tan(), log() }
  |-- StandardScientificOperations { angle_mode }
  |-- ExternalLibraryAdapter { angle_mode }
```