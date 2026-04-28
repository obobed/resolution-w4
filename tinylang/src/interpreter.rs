use std::{collections::HashMap, result};
use crate::ast::{Expr, Op, Statement};

#[derive(Clone, Debug)]
pub enum Value {
    Number(f64),
    StringVal(String),
    Bool(bool),
    Nil,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => {
                if *n == (*n as i64) as f64 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{n}")
                }
            }
            Value::StringVal(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
        }
    }
}

struct Function {
    params: Vec<String>,
    body: Vec<Statement>,
}

pub struct Interpreter {
    variables: HashMap<String, Value>,
    functions: HashMap<String, Function>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn run(&mut self, program: Vec<Statement>) -> Result<(), String> {
        for stmt in &program {
            self.exec_statement(stmt)?;
        }
        Ok(())
    }

    fn exec_statement(&mut self, stmt: &Statement) -> Result<Option<Value>, String> {
        match stmt {
            Statement::Puts(expr) => {
                let val = self.eval_expr(expr)?;
                println!("{val}");
                Ok(None)
            }
            Statement::Assign(name, expr) => {
                let val = self.eval_expr(expr)?;
                self.variables.insert(name.clone(), val);
                Ok(None)
            }
            Statement::If(cond, body, else_body) => {
                let val = self.eval_expr(cond)?;
                if self.is_truthy(&val) {
                    for s in body {
                        if let Some(ret) = self.exec_statement(s)? {
                            return Ok(Some(ret));
                        }
                    }
                } else if let Some(else_stmts) = else_body {
                    for s in else_stmts {
                        if let Some(ret) = self.exec_statement(s)? {
                            return Ok(Some(ret));
                        }
                    }
                }
                Ok(None)
            }
            Statement::While(cond, body) => {
                loop {
                    let val = self.eval_expr(cond)?;
                    if !self.is_truthy(&val) {
                        break;
                    }
                    for s in body {
                        if let Some(ret) = self.exec_statement(s)? {
                            return Ok(Some(ret));
                        }
                    }
                }
                Ok(None)
            }
            Statement::Def(name, params, body) => {
                self.functions.insert(name.clone(), Function {
                    params: params.clone(),
                    body: body.clone(),
                });
                Ok(None)
            }
            Statement::Return(expr) => {
                let val = self.eval_expr(expr)?;
                Ok(Some(val))
            }
            Statement::ExprStatement(expr) => {
                self.eval_expr(expr)?;
                Ok(None)
            }
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::StringLiteral(s) => Ok(Value::StringVal(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Variable(name) => {
                self.variables.get(name)
                    .cloned()
                    .ok_or_else(|| format!("undefined variable: {name}"))
            }
            Expr::BinOp(left, op, right) => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                self.eval_binop(l, op, r)
            }
            Expr::Call(name, args) => {
                let evaluated_args: Result<Vec<Value>, String> =
                    args.iter().map(|a| self.eval_expr(a)).collect();
                let evaluated_args = evaluated_args?;
                self.call_function(name, evaluated_args)
            }
        }
    }

    fn eval_binop(&self, left: Value, op: &Op, right: Value) -> Result<Value, String> {
        match (left, op, right) {
            (Value::Number(a), Op::Add, Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::Number(a), Op::Sub, Value::Number(b)) => Ok(Value::Number(a - b)),
            (Value::Number(a), Op::Mul, Value::Number(b)) => Ok(Value::Number(a * b)),
            (Value::Number(a), Op::Div, Value::Number(b)) => {
                if b == 0.0 {
                    Err("division by zero".to_string())
                } else {
                    Ok(Value::Number(a / b))
                }
            }
            (Value::Number(a), Op::Eq, Value::Number(b)) => Ok(Value::Bool(a == b)),
            (Value::Number(a), Op::Lt, Value::Number(b)) => Ok(Value::Bool(a < b)),
            (Value::Number(a), Op::Gt, Value::Number(b)) => Ok(Value::Bool(a > b)),
            (Value::StringVal(a), Op::Add, Value::StringVal(b)) => {
                Ok(Value::StringVal(format!("{a}{b}")))
            }
            (Value::StringVal(a), Op::Eq, Value::StringVal(b)) => Ok(Value::Bool(a == b)),
            _ => Err("type error in binary operation".to_string()),
        }
    }

    fn call_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value, String> {
        if let Ok(result) = self.call_builtin(name, args.clone()) {
            return Ok(result)
        }
        
        let func = self.functions.get(name)
            .ok_or_else(|| format!("undefined function: {name}"))?;

        if args.len() != func.params.len() {
            return Err(format!(
                "{name} expects {} arguments, got {}",
                func.params.len(),
                args.len()
            ));
        }

        // Save the current variables so we can restore them after the call.
        let old_vars = self.variables.clone();

        for (param, arg) in func.params.clone().into_iter().zip(args) {
            self.variables.insert(param, arg);
        }

        let body = func.body.clone();
        let mut result = Value::Nil;
        for stmt in &body {
            if let Some(ret) = self.exec_statement(stmt)? {
                result = ret;
                break;
            }
        }

        self.variables = old_vars;
        Ok(result)
    }

    fn is_truthy(&self, val: &Value) -> bool {
        match val {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::StringVal(s) => !s.is_empty(),
            Value::Nil => false,
        }
    }
}