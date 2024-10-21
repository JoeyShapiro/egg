// File: src/main.rs
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "egg.pest"]
struct EggParser;

struct Interpreter {
    variables: std::collections::HashMap<String, Value>,
}

// TODO
// - handle vars better with sub style
// - handle arrays better and allow assignment
// - modify array instead of create new one

impl Interpreter {
    fn new() -> Self {
        Self {
            variables: std::collections::HashMap::new(),
        }
    }

    fn interpret(&mut self, input: &str) {
        let pairs = EggParser::parse(Rule::program, input).unwrap_or_else(|e| panic!("{}", e));
        for pair in pairs {
            for statement in pair.into_inner() {
                match statement.as_rule() {
                    Rule::statement => self.interpret_statement(statement),
                    _ => {}
                }
            }
        }
    }

    fn interpret_statement(&mut self, statement: pest::iterators::Pair<Rule>) {
        for inner_pair in statement.into_inner() {
            match inner_pair.as_rule() {
                Rule::assignment => self.interpret_assignment(inner_pair),
                Rule::print_statement => self.interpret_print_statement(inner_pair),
                Rule::for_statement => self.interpret_for_statement(inner_pair),
                Rule::standalone_identifier => self.print_variable(inner_pair),
                _ => {}
            }
        }
    }

    fn interpret_assignment(&mut self, assignment: pest::iterators::Pair<Rule>) {
        let mut inner = assignment.into_inner();
        let var_name = inner.next().unwrap().as_str().to_string();
        let value = self.evaluate_expression(inner.next().unwrap());
        self.variables.insert(var_name, value);
    }
    
    fn print_variable(&mut self, identifier: pest::iterators::Pair<Rule>) {
        let var_name = identifier.as_str();
        match self.variables.get(var_name) {
            Some(value) => println!("{} = {}", var_name, value.to_string()),
            None => println!("{} is not defined", var_name),
        }
    }
    
    fn interpret_print_statement(&mut self, print_stmt: pest::iterators::Pair<Rule>) {
        let expression = print_stmt.into_inner().next().unwrap();
        let result = self.evaluate_expression(expression);
        println!("{}", result);
    }
    
    fn evaluate_expression(&mut self, expr: pest::iterators::Pair<Rule>) -> Value {
        // println!("Evaluating: {:?}", expr);
        match expr.as_rule() {
            Rule::expression => self.evaluate_expression(expr.into_inner().next().unwrap()),
            Rule::string => Value::String(expr.into_inner().as_str().to_string()),
            Rule::number => Value::Number(expr.as_str().parse().unwrap()),
            Rule::array => {
                let elements: Vec<Value> = expr.into_inner()
                    .map(|e| self.evaluate_expression(e))
                    .collect();
                
                if elements.is_empty() {
                    Value::Array(vec![])
                } else {
                    if elements.iter().all(|_e| true) {
                        Value::Array(elements)
                    } else {
                        Value::String("Error: Array elements must be of the same type".to_string())
                    }
                }
            },
            Rule::identifier => {
                // In a real interpreter, you would look up the variable value here
                // Value::String(format!("Variable: {}", expr.as_str()))
                self.variables.get(expr.as_str())
                    .cloned()
                    .unwrap_or_else(|| Value::String(format!("Undefined variable: {}", expr.as_str())))
            },
            _ => Value::String("Error: Unknown expression type".to_string()),
        }
    }
    
    fn interpret_for_statement(&mut self, for_stmt: pest::iterators::Pair<Rule>) {
        let mut inner = for_stmt.into_inner();
        let iter_name = inner.next().unwrap().as_str().to_string();
        let mutator = inner.next().unwrap().as_str();
        let ender = inner.next().unwrap();
        let end_name = ender.as_str();
        let end = self.evaluate_expression(ender);

        // Check if "by" clause is present
        let (step, block) = match inner.next() {
            Some(pair) if pair.as_rule() == Rule::expression => {
                // "by" clause is present
                (self.evaluate_expression(pair), inner.next().unwrap())
            },
            Some(block_pair) => {
                // No "by" clause, use default step of 1
                (Value::Number(1), block_pair)
            },
            None => panic!("Unexpected end of for loop statement"),
        };

        match mutator {
            "in" => {
                if let (Value::Number(end), Value::Number(step)) = (end, step) {
                    let mut current = 0;
                    while current < end {
                        self.variables.insert(iter_name.clone(), Value::Number(current));
                        self.interpret_block(block.clone());
                        current += step;
                    }
                } else {
                    println!("Error: Invalid for loop parameters");
                }
            },
            "of" => {
                if let Value::Array(arr) = end {
                    let mut arr_new = vec![];
                    for element in arr {
                        self.variables.insert(iter_name.clone(), element);
                        self.interpret_block(block.clone());
                        arr_new.push(self.variables.get(&iter_name).unwrap().clone());
                    }
                    self.variables.insert(end_name.to_string(), Value::Array(arr_new));
                } else {
                    println!("Error: Invalid for loop parameters");
                }
            },
            _ => println!("Error: Invalid for loop mutator"),
            
        }
    }
    
    fn interpret_block(&mut self, block: pest::iterators::Pair<Rule>) {
        for statement in block.into_inner() {
            self.interpret_statement(statement);
        }
    }
}

#[derive(Debug, Clone)]
enum Value {
    String(String),
    Number(i64),
    Array(Vec<Value>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => write!(f, "{}", n),
            Value::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                write!(f, "[ {} ]", elements.join(" "))
            }
        }
    }
}

fn main() {
    // let input = "for i in 10 by 2 { print(i); }";
    let input = r#"
        print "Hello, World!"
        print 42
        print x
        for i in 10 by 2 {
            i
        }

        arr = [ 1 2 34 ]
        print arr

        for i of arr {
            i = 0
        }
        arr
    "#;

    let mut interpreter = Interpreter::new();
    interpreter.interpret(input);

    // // read eval print loop
    // loop {
    //     // get user input
    //     let mut input = String::new();
    //     print!("🥚: ");
    //     std::io::Write::flush(&mut std::io::stdout()).unwrap();
    //     std::io::stdin().read_line(&mut input).unwrap();
        
    //     let pairs = EggParser::parse(Rule::program, &input).unwrap_or_else(|e| panic!("{}", e));

    //     for pair in pairs {
    //         for statement in pair.into_inner() {
    //             match statement.as_rule() {
    //                 Rule::statement => interpret_statement(statement),
    //                 _ => {}
    //             }
    //         }
    //     }

    //     println!();
    // }
}
