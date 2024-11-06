use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "egg.pest"]
struct EggParser;

struct Interpreter {
    variables: std::collections::HashMap<String, Value>,
    base: u32, // you dont need this many, but rust wants this
}

// TODO
// - handle vars better with sub style
// - handle arrays better and allow assignment
// - modify array instead of create new one
// - bitwise with block
// - semicolon optional or print if at end
// - piping
// - for start
// - load from as
// - 0 < x < 10
// - double for loop
// - empty for return, ? for print
// - allow blocks everywhere (kinda can), and ans is returned
// - ; is same line, ? is debug print, empty sets ans which is return

impl Interpreter {
    fn new() -> Self {
        Self {
            variables: std::collections::HashMap::new(),
            base: 10,
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
                Rule::call => { self.interpret_call(inner_pair); },
                Rule::for_statement => self.interpret_for_statement(inner_pair),
                Rule::if_statement => {
                    let mut inner = inner_pair.into_inner();
                    let condition = self.evaluate_expression(inner.next().unwrap());
                    let block_if = inner.next().unwrap();
                    let block_else = inner.next();
                    if let Value::Number(n) = condition {
                        if n != 0 {
                            self.interpret_block(block_if);
                        } else {
                            if let Some(block) = block_else {
                                self.interpret_block(block);
                            }
                        }
                    }
                },
                Rule::which_statement => {
                    let mut inner = inner_pair.into_inner();
                    let mut should_break = false;
                    while !should_break {
                        let mut expression = inner.next().unwrap().into_inner();
                        // check if default case
                        let case = expression.next().unwrap();
                        let condition = if case.as_rule() == Rule::expression {
                            self.evaluate_expression(case)
                        } else {
                            Value::Number(1)
                        };

                        let block_if = expression.next().unwrap();
                        if let Value::Number(n) = condition {
                            if n != 0 {
                                should_break = true;
                                self.interpret_block(block_if);
                            }
                        }
                    }
                },
                // Rule::standalone_identifier => self.print_variable(inner_pair),å
                Rule::expression => {
                    let ans = self.evaluate_expression(inner_pair);
                    self.variables.insert("ans".to_string(), ans.clone());
                    println!("ans = {}", ans); // TODO move this for special case
                }
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
    
    fn interpret_call(&mut self, stmt: pest::iterators::Pair<Rule>) -> Value {
        let mut inner = stmt.into_inner();
        let func_name = inner.next().unwrap().as_str();
        let expression = inner.next().unwrap();
        let result = self.evaluate_expression(expression);
        // println!("{func_name} {}", result);
        match func_name {
            "print" => println!("{}", result),
            _ => println!("Error: Unknown function"),
        }
        Value::String("".to_string())
    }
    
    fn evaluate_expression(&mut self, expr: pest::iterators::Pair<Rule>) -> Value {
        // println!("Evaluating: {:?}", expr);
        match expr.as_rule() {
            Rule::expression | Rule::term => self.evaluate_expression(expr.into_inner().next().unwrap()),
            Rule::string => Value::String(expr.into_inner().as_str().to_string()),
            Rule::number => {
                // split on b, should only be one
                let val = expr.as_str().split("b").collect::<Vec<&str>>();
                let (num, base) = match val.len() {
                    1 => (val[0], self.base),
                    2 => (val[0], val[1].parse().unwrap()),
                    _ => panic!("Invalid number"),
                };

                // convert to base 10
                let num = i64::from_str_radix(num, base).unwrap();

                Value::Number(num)
            },
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
            Rule::binary_operation => {
                let mut parts = expr.into_inner();
                let mut result = self.evaluate_expression(parts.next().unwrap());
                
                while let (Some(op), Some(right)) = (parts.next(), parts.next()) {
                    let right_val = self.evaluate_expression(right);
                    result = match (result, op.as_str(), right_val) {
                        (Value::Number(a), "+", Value::Number(b)) => Value::Number(a + b),
                        (Value::Number(a), "-", Value::Number(b)) => Value::Number(a - b),
                        (Value::Number(a), "*", Value::Number(b)) => Value::Number(a * b),
                        (Value::Number(a), "/", Value::Number(b)) => Value::Number(a / b),
                        (Value::Number(a), "**", Value::Number(b)) => Value::Number(a.pow(b as u32)),
                        (Value::Number(a), "==", Value::Number(b)) => Value::Number((a == b) as i64),
                        (Value::Number(a), ">=", Value::Number(b)) => Value::Number((a >= b) as i64),
                        (Value::Number(a), "<=", Value::Number(b)) => Value::Number((a <= b) as i64),
                        _ => {
                            println!("Error: Invalid operation {op:?}");
                            Value::String("Error: Invalid operation".to_string())
                        },
                    };
                }
                result
            },
            Rule::function => {
                let mut parts = expr.into_inner();
                let params: Vec<String> = parts.next().unwrap().as_str().split_whitespace().map(|s| s.to_string()).collect();
                let block = parts.next().unwrap().to_string();
                Value::Function("".to_string(), params, block)
            },
            Rule::call => {
                self.interpret_call(expr)
            },
            _ => {
                println!("Evaluating: {:?}", expr);
                Value::String("Error: Unknown expression type".to_string())
            },
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
    Function(String, Vec<String>, String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => write!(f, "{}", n),
            Value::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                write!(f, "[ {} ]", elements.join(" "))
            },
            Value::Function(name, params, pair) => {
                write!(f, "Function({})", params.join("\n"))
            },
        }
    }
}

fn main() {
    // let input = "for i in 10 by 2 { print(i); }";
    let mut interpreter = Interpreter::new();
    
    // get file from args
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let filename = &args[1];
        let input = std::fs::read_to_string(filename).unwrap();
        interpreter.interpret(&input);
    } else {
        // read eval print loop
        loop {
            // get user input
            let mut input = String::new();
            print!("🥚: ");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            std::io::stdin().read_line(&mut input).unwrap();
        
            interpreter.interpret(&input);

            println!();
        }
    }
}
