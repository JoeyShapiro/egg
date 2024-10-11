// File: src/main.rs
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "egg.pest"]
struct EggParser;

struct Interpreter {
    variables: std::collections::HashMap<String, Value>,
}

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
                Rule::print_statement => self.interpret_print_statement(inner_pair),
                Rule::for_statement => self.interpret_for_statement(inner_pair),
                Rule::standalone_identifier => self.print_variable(inner_pair),
                _ => {}
            }
        }
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
            Rule::identifier => {
                // In a real interpreter, you would look up the variable value here
                Value::String(format!("Variable: {}", expr.as_str()))
            },
            _ => Value::String("Error: Unknown expression type".to_string()),
        }
    }
    
    fn interpret_for_statement(&mut self, for_loop: pest::iterators::Pair<Rule>) {
        let mut inner = for_loop.into_inner();
        let var_name = inner.next().unwrap().as_str();
        let limit = inner.next().unwrap().as_str().parse::<i32>().unwrap();
        let step = inner.next().unwrap().as_str().parse::<i32>().unwrap();
        let body = inner.next().unwrap();
    
        for i in (0..limit).step_by(step as usize) {
            println!("{} = {}", var_name, i);
            self.interpret_block(body.clone(), i);
        }
    }
    
    fn interpret_block(&mut self, block: pest::iterators::Pair<Rule>, i: i32) {
        for statement in block.into_inner() {
            match statement.as_rule() {
                Rule::print_statement => {
                    let content = statement.into_inner().next().unwrap().as_str();
                    if content == "i" {
                        println!("{}", i);
                    } else {
                        println!("{}", content);
                    }
                }
                // Add other statement types here
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Value {
    String(String),
    Number(i64),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => write!(f, "{}", n),
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
            print i
        }
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
