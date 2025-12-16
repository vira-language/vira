use std::env;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::exit;

// Simple Compiler for Vira Language in Rust
// Compiles .vira source to bytecode (.object)
// Bytecode format: simple stack-based VM bytecode
// Opcodes: PUSH_NUM, PUSH_STR, ADD, SUB, MUL, DIV, STORE, LOAD, CALL, WRITE, HALT
// Supports basic features as per syntax

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
                    // Fallback
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
#[derive(Debug)]
enum Expr {
    Number(f64),
    String(String),
    Identifier(String),
    Binary(char, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Debug)]
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
            &self.tokens[self.tokens.len() - 1] // Should be Eof, but safe
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
                let callee = self.previous().value; // Identifier
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

// Bytecode
#[derive(Debug, Clone)]
enum Opcode {
    PushNum(f64),
    PushStr(String),
    Add,
    Sub,
    Mul,
    Div,
    Store(String),
    Load(String),
    Call(usize), // num args
    Write,
    Halt,
}

struct Compiler {
    bytecode: Vec<Opcode>,
    // Simple symbol table, etc. For simplicity, no scopes
    vars: Vec<String>,
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            bytecode: Vec::new(),
            vars: Vec::new(),
        }
    }

    fn compile(&mut self, program: &Program) {
        for stmt in &program.statements {
            self.compile_stmt(stmt);
        }
        self.bytecode.push(Opcode::Halt);
    }

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl(name, init) => {
                if let Some(expr) = init {
                    self.compile_expr(expr);
                } else {
                    self.bytecode.push(Opcode::PushNum(0.0)); // Default
                }
                self.bytecode.push(Opcode::Store(name.clone()));
                self.vars.push(name.clone());
            }
            Stmt::FuncDef(_name, _params, _body) => {
                // Functions not supported in this simple compiler
                println!("Warning: Functions not compiled");
            }
            Stmt::Write(expr) => {
                self.compile_expr(expr);
                self.bytecode.push(Opcode::Write);
            }
            Stmt::Import(_lib) => {
                // Imports ignored for now
            }
            Stmt::ExprStmt(expr) => {
                self.compile_expr(expr);
            }
        }
    }

    fn compile_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(val) => self.bytecode.push(Opcode::PushNum(*val)),
            Expr::String(val) => self.bytecode.push(Opcode::PushStr(val.clone())),
            Expr::Identifier(name) => self.bytecode.push(Opcode::Load(name.clone())),
            Expr::Binary(op, left, right) => {
                self.compile_expr(left);
                self.compile_expr(right);
                match op {
                    '+' => self.bytecode.push(Opcode::Add),
                    '-' => self.bytecode.push(Opcode::Sub),
                    '*' => self.bytecode.push(Opcode::Mul),
                    '/' => self.bytecode.push(Opcode::Div),
                    _ => panic!("Unknown operator"),
                }
            }
            Expr::Call(_callee, _args) => {
                // Calls not supported
                println!("Warning: Calls not compiled");
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: compiler <input.vira> [-o output.object]");
        exit(1);
    }

    let input_path = Path::new(&args[1]);
    let mut output_path = input_path.with_extension("object");

    if args.len() > 3 && args[2] == "-o" {
        output_path = Path::new(&args[3]).to_path_buf();
    }

    let source = std::fs::read_to_string(input_path).expect("Failed to read file");

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
            exit(1);
        }
    }

    let mut parser = Parser::new(tokens);
    let program = parser.parse();

    let mut compiler = Compiler::new();
    compiler.compile(&program);

    let mut file = File::create(output_path).expect("Failed to create output file");

    // Simple serialization: opcode index + data
    for op in compiler.bytecode {
        match op {
            Opcode::PushNum(val) => {
                file.write_all(&[0]).unwrap();
                file.write_all(&val.to_le_bytes()).unwrap();
            }
            Opcode::PushStr(s) => {
                file.write_all(&[1]).unwrap();
                let bytes = s.as_bytes();
                file.write_all(&(bytes.len() as u32).to_le_bytes()).unwrap();
                file.write_all(bytes).unwrap();
            }
            Opcode::Add => file.write_all(&[2]).unwrap(),
            Opcode::Sub => file.write_all(&[3]).unwrap(),
            Opcode::Mul => file.write_all(&[4]).unwrap(),
            Opcode::Div => file.write_all(&[5]).unwrap(),
            Opcode::Store(name) => {
                file.write_all(&[6]).unwrap();
                let bytes = name.as_bytes();
                file.write_all(&(bytes.len() as u32).to_le_bytes()).unwrap();
                file.write_all(bytes).unwrap();
            }
            Opcode::Load(name) => {
                file.write_all(&[7]).unwrap();
                let bytes = name.as_bytes();
                file.write_all(&(bytes.len() as u32).to_le_bytes()).unwrap();
                file.write_all(bytes).unwrap();
            }
            Opcode::Call(num) => {
                file.write_all(&[8]).unwrap();
                file.write_all(&num.to_le_bytes()).unwrap();
            }
            Opcode::Write => file.write_all(&[9]).unwrap(),
            Opcode::Halt => file.write_all(&[10]).unwrap(),
        }
    }

    println!("Compiled to bytecode.");
                  }
