use clap::{Parser as ClapParser, Subcommand};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

// Simple Interpreter for Vira Language in Rust
// Interprets .vira source directly
// Reuses lexer and parser from previous examples
// Evaluates AST with a simple runtime environment

#[derive(Debug, Clone)]
enum TokenType {
    Eof,
    Identifier,
    Number,
    String,
    Colon,
    Assign,
    Plus,
    Minus,
    Mul,
    Div,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Comma,
    Let,
    Def,
    Write,
    ImportStart,
    Comment,
    Unknown,
}

#[derive(Debug, Clone)]
struct Token {
    typ: TokenType,
    value: String,
    line: usize,
    column: usize,
}

struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    fn new(source: String) -> Self {
        Lexer {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        if self.pos >= self.source.len() {
            return Token {
                typ: TokenType::Eof,
                value: String::new(),
                line: self.line,
                column: self.column,
            };
        }

        let ch = self.source[self.pos];
        if ch.is_digit(10) {
            self.number()
        } else if ch.is_alphabetic() || ch == '_' {
            self.identifier()
        } else if ch == '"' {
            self.string()
        } else if ch == '<' {
            self.comment()
        } else if ch == ':' {
            self.advance();
            if self.pos < self.source.len() && self.source[self.pos].is_alphabetic() {
                let mut lib = String::new();
                while self.pos < self.source.len()
                    && (self.source[self.pos].is_alphanumeric() || self.source[self.pos] == '_')
                {
                    lib.push(self.source[self.pos]);
                    self.advance();
                }
                if self.pos < self.source.len() && self.source[self.pos] == ':' {
                    self.advance();
                    Token {
                        typ: TokenType::ImportStart,
                        value: lib,
                        line: self.line,
                        column: self.column,
                    }
                } else {
                    Token {
                        typ: TokenType::Colon,
                        value: ":".to_string(),
                        line: self.line,
                        column: self.column,
                    }
                }
            } else {
                Token {
                    typ: TokenType::Colon,
                    value: ":".to_string(),
                    line: self.line,
                    column: self.column,
                }
            }
        } else {
            self.advance();
            match ch {
                '=' => Token {
                    typ: TokenType::Assign,
                    value: "=".to_string(),
                    line: self.line,
                    column: self.column,
                },
                '+' => Token {
                    typ: TokenType::Plus,
                    value: "+".to_string(),
                    line: self.line,
                    column: self.column,
                },
                '-' => Token {
                    typ: TokenType::Minus,
                    value: "-".to_string(),
                    line: self.line,
                    column: self.column,
                },
                '*' => Token {
                    typ: TokenType::Mul,
                    value: "*".to_string(),
                    line: self.line,
                    column: self.column,
                },
                '/' => Token {
                    typ: TokenType::Div,
                    value: "/".to_string(),
                    line: self.line,
                    column: self.column,
                },
                '(' => Token {
                    typ: TokenType::LParen,
                    value: "(".to_string(),
                    line: self.line,
                    column: self.column,
                },
                ')' => Token {
                    typ: TokenType::RParen,
                    value: ")".to_string(),
                    line: self.line,
                    column: self.column,
                },
                '{' => Token {
                    typ: TokenType::LBrace,
                    value: "{".to_string(),
                    line: self.line,
                    column: self.column,
                },
                '}' => Token {
                    typ: TokenType::RBrace,
                    value: "}".to_string(),
                    line: self.line,
                    column: self.column,
                },
                ';' => Token {
                    typ: TokenType::Semicolon,
                    value: ";".to_string(),
                    line: self.line,
                    column: self.column,
                },
                ',' => Token {
                    typ: TokenType::Comma,
                    value: ",".to_string(),
                    line: self.line,
                    column: self.column,
                },
                _ => Token {
                    typ: TokenType::Unknown,
                    value: ch.to_string(),
                    line: self.line,
                    column: self.column,
                },
            }
        }
    }

    fn advance(&mut self) {
        if self.source[self.pos] == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        self.pos += 1;
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.source.len() && self.source[self.pos].is_whitespace() {
            self.advance();
        }
    }

    fn number(&mut self) -> Token {
        let mut num = String::new();
        let line = self.line;
        let column = self.column;
        while self.pos < self.source.len() && self.source[self.pos].is_digit(10) {
            num.push(self.source[self.pos]);
            self.advance();
        }
        Token {
            typ: TokenType::Number,
            value: num,
            line,
            column,
        }
    }

    fn identifier(&mut self) -> Token {
        let mut id = String::new();
        let line = self.line;
        let column = self.column;
        while self.pos < self.source.len()
            && (self.source[self.pos].is_alphanumeric() || self.source[self.pos] == '_')
        {
            id.push(self.source[self.pos]);
            self.advance();
        }
        let typ = match id.as_str() {
            "let" => TokenType::Let,
            "def" => TokenType::Def,
            "write" => TokenType::Write,
            _ => TokenType::Identifier,
        };
        Token {
            typ,
            value: id,
            line,
            column,
        }
    }

    fn string(&mut self) -> Token {
        let mut str_val = String::new();
        let line = self.line;
        let column = self.column;
        self.advance(); // Skip "
        while self.pos < self.source.len() && self.source[self.pos] != '"' {
            if self.source[self.pos] == '\\' {
                self.advance();
                if self.pos < self.source.len() {
                    str_val.push(self.source[self.pos]);
                    self.advance();
                }
            } else {
                str_val.push(self.source[self.pos]);
                self.advance();
            }
        }
        if self.pos >= self.source.len() || self.source[self.pos] != '"' {
            panic!("Unterminated string at line {}", self.line);
        }
        self.advance(); // Skip closing "
        Token {
            typ: TokenType::String,
            value: str_val,
            line,
            column,
        }
    }

    fn comment(&mut self) -> Token {
        let mut comment = String::new();
        let line = self.line;
        let column = self.column;
        self.advance(); // Skip <
        while self.pos < self.source.len() && self.source[self.pos] != '\n' {
            comment.push(self.source[self.pos]);
            self.advance();
        }
        Token {
            typ: TokenType::Comment,
            value: comment,
            line,
            column,
        }
    }
}

// AST Nodes
#[derive(Debug, Clone)]
enum Expr {
    Number(f64),
    String(String),
    Identifier(String),
    Binary(char, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Debug, Clone)]
enum Stmt {
    VarDecl(String, Option<Expr>),
    FuncDef(String, Vec<String>, Vec<Stmt>),
    Write(Expr),
    Import(String),
    ExprStmt(Expr),
}

#[derive(Debug)]
struct Program {
    statements: Vec<Stmt>,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn parse(&mut self) -> Program {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            }
        }
        Program { statements }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().typ, TokenType::Eof)
    }

    fn peek(&self) -> &Token {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos]
        } else {
            &self.tokens[self.tokens.len() - 1]
        }
    }

    fn advance(&mut self) -> Token {
        let token = self.peek().clone();
        self.pos += 1;
        token
    }

    fn match_type(&mut self, typ: TokenType) -> bool {
        if matches!(self.peek().typ, typ) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, typ: TokenType, msg: &str) -> Token {
        if self.match_type(typ.clone()) {
            self.previous()
        } else {
            panic!("{} at line {}", msg, self.peek().line);
        }
    }

    fn previous(&self) -> Token {
        self.tokens[self.pos - 1].clone()
    }

    fn declaration(&mut self) -> Option<Stmt> {
        if self.match_type(TokenType::Let) {
            Some(self.var_decl())
        } else if self.match_type(TokenType::Def) {
            Some(self.func_def())
        } else if self.match_type(TokenType::ImportStart) {
            Some(self.import_stmt())
        } else {
            Some(self.statement())
        }
    }

    fn var_decl(&mut self) -> Stmt {
        let name = self.consume(TokenType::Identifier, "Expected variable name").value;
        let mut init = None;
        if self.match_type(TokenType::Assign) {
            init = Some(self.expression());
        }
        self.consume(TokenType::Semicolon, "Expected ';' after variable declaration");
        Stmt::VarDecl(name, init)
    }

    fn func_def(&mut self) -> Stmt {
        let name = self.consume(TokenType::Identifier, "Expected function name").value;
        self.consume(TokenType::LParen, "Expected '(' after function name");
        let mut params = Vec::new();
        if !self.match_type(TokenType::RParen) {
            loop {
                params.push(self.consume(TokenType::Identifier, "Expected parameter name").value);
                if !self.match_type(TokenType::Comma) {
                    break;
                }
            }
            self.consume(TokenType::RParen, "Expected ')' after parameters");
        }
        self.consume(TokenType::LBrace, "Expected '{' before function body");
        let mut body = Vec::new();
        while !self.match_type(TokenType::RBrace) && !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                body.push(stmt);
            }
        }
        Stmt::FuncDef(name, params, body)
    }

    fn import_stmt(&mut self) -> Stmt {
        let lib = self.previous().value;
        self.consume(TokenType::Semicolon, "Expected ';' after import");
        Stmt::Import(lib)
    }

    fn statement(&mut self) -> Stmt {
        if self.match_type(TokenType::Write) {
            self.write_stmt()
        } else {
            self.expr_stmt()
        }
    }

    fn write_stmt(&mut self) -> Stmt {
        let expr = self.expression();
        self.consume(TokenType::Semicolon, "Expected ';' after write");
        Stmt::Write(expr)
    }

    fn expr_stmt(&mut self) -> Stmt {
        let expr = self.expression();
        self.consume(TokenType::Semicolon, "Expected ';' after expression");
        Stmt::ExprStmt(expr)
    }

    fn expression(&mut self) -> Expr {
        self.additive()
    }

    fn additive(&mut self) -> Expr {
        let mut expr = self.multiplicative();
        while self.match_type(TokenType::Plus) || self.match_type(TokenType::Minus) {
            let op = self.previous().value.chars().next().unwrap();
            let right = self.multiplicative();
            expr = Expr::Binary(op, Box::new(expr), Box::new(right));
        }
        expr
    }

    fn multiplicative(&mut self) -> Expr {
        let mut expr = self.unary();
        while self.match_type(TokenType::Mul) || self.match_type(TokenType::Div) {
            let op = self.previous().value.chars().next().unwrap();
            let right = self.unary();
            expr = Expr::Binary(op, Box::new(expr), Box::new(right));
        }
        expr
    }

    fn unary(&mut self) -> Expr {
        if self.match_type(TokenType::Minus) {
            let right = self.unary();
            Expr::Binary('-', Box::new(Expr::Number(0.0)), Box::new(right))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Expr {
        if self.match_type(TokenType::Number) {
            Expr::Number(self.previous().value.parse::<f64>().unwrap())
        } else if self.match_type(TokenType::String) {
            Expr::String(self.previous().value)
        } else if self.match_type(TokenType::Identifier) {
            if self.match_type(TokenType::LParen) {
                let callee = self.previous().value;
                let mut args = Vec::new();
                if !self.match_type(TokenType::RParen) {
                    loop {
                        args.push(self.expression());
                        if !self.match_type(TokenType::Comma) {
                            break;
                        }
                    }
                    self.consume(TokenType::RParen, "Expected ')' after arguments");
                }
                Expr::Call(callee, args)
            } else {
                Expr::Identifier(self.previous().value)
            }
        } else if self.match_type(TokenType::LParen) {
            let expr = self.expression();
            self.consume(TokenType::RParen, "Expected ')' after expression");
            expr
        } else {
            panic!("Unexpected token at line {}", self.peek().line);
        }
    }
}

// Runtime Value
#[derive(Debug, Clone)]
enum Value {
    Num(f64),
    Str(String),
    Func(String, Vec<String>, Vec<Stmt>), // name, params, body
    None,
}

// Interpreter
struct Interpreter {
    env: HashMap<String, Value>,
}

impl Interpreter {
    fn new() -> Self {
        Interpreter {
            env: HashMap::new(),
        }
    }

    fn interpret(&mut self, program: &Program) {
        for stmt in &program.statements {
            self.eval_stmt(stmt);
        }
    }

    fn eval_stmt(&mut self, stmt: &Stmt) -> Value {
        match stmt {
            Stmt::VarDecl(name, init) => {
                let val = if let Some(expr) = init {
                    self.eval_expr(expr)
                } else {
                    Value::None
                };
                self.env.insert(name.clone(), val);
                Value::None
            }
            Stmt::FuncDef(name, params, body) => {
                self.env.insert(name.clone(), Value::Func(name.clone(), params.clone(), body.clone()));
                Value::None
            }
            Stmt::Write(expr) => {
                let val = self.eval_expr(expr);
                match val {
                    Value::Num(n) => println!("{}", n),
                    Value::Str(s) => println!("{}", s),
                    _ => println!("{:?}", val),
                }
                Value::None
            }
            Stmt::Import(_lib) => {
                // For simplicity, ignore imports or load std if "std"
                println!("Importing {} not implemented", _lib);
                Value::None
            }
            Stmt::ExprStmt(expr) => self.eval_expr(expr),
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Number(val) => Value::Num(*val),
            Expr::String(val) => Value::Str(val.clone()),
            Expr::Identifier(name) => {
                self.env.get(name).cloned().unwrap_or_else(|| panic!("Undefined variable: {}", name))
            }
            Expr::Binary(op, left, right) => {
                let l = self.eval_expr(left);
                let r = self.eval_expr(right);
                match (l, r) {
                    (Value::Num(a), Value::Num(b)) => match op {
                        '+' => Value::Num(a + b),
                        '-' => Value::Num(a - b),
                        '*' => Value::Num(a * b),
                        '/' => Value::Num(a / b),
                        _ => panic!("Unknown operator"),
                    },
                    (Value::Str(a), Value::Str(b)) if *op == '+' => Value::Str(a + &b),
                    _ => panic!("Type mismatch"),
                }
            }
            Expr::Call(callee, args) => {
                let func = self.env.get(callee).cloned().unwrap_or_else(|| panic!("Undefined function: {}", callee));
                if let Value::Func(_name, params, body) = func {
                    if params.len() != args.len() {
                        panic!("Argument count mismatch");
                    }
                    let old_env = self.env.clone();
                    for (param, arg) in params.iter().zip(args.iter()) {
                        let arg_val = self.eval_expr(arg);
                        self.env.insert(param.clone(), arg_val);
                    }
                    let mut result = Value::None;
                    for stmt in &body {
                        result = self.eval_stmt(stmt);
                    }
                    self.env = old_env;
                    result
                } else {
                    panic!("Not a function: {}", callee);
                }
            }
        }
    }
}

#[derive(ClapParser)]
#[command(version, about = "Vira Interpreter")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        input: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { input } => {
            let source = fs::read_to_string(&input).expect("Failed to read input file");

            let mut lexer = Lexer::new(source);
            let mut tokens = Vec::new();
            loop {
                let token = lexer.next_token();
                if matches!(token.typ, TokenType::Eof) {
                    break;
                }
                if !matches!(token.typ, TokenType::Comment | TokenType::Unknown) {
                    tokens.push(token);
                } else if matches!(token.typ, TokenType::Unknown) {
                    eprintln!("Unknown token: {} at line {}", token.value, token.line);
                    std::process::exit(1);
                }
            }

            let mut parser = Parser::new(tokens);
            let program = parser.parse();

            let mut interpreter = Interpreter::new();
            interpreter.interpret(&program);

            println!("Interpretation completed.");
        }
    }
                             }
