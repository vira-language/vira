use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use cranelift::prelude::*;
use cranelift_codegen::binemit::{NullStackMapSink, NullTrapSink};
use cranelift_codegen::ir::{AbiParam, ExternalName, InstBuilder, MemFlags};
use cranelift_codegen::isa::{self, CallConv};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{DataContext, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Identifier(String),
    Keyword(String),
    Number(i64),
    StringLiteral(String),
    Punctuator(char),
    EOF,
}

struct Lexer {
    input: String,
    position: usize,
}

impl Lexer {
    fn new(input: String) -> Self {
        Lexer { input, position: 0 }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        if self.position >= self.input.len() {
            return Token::EOF;
        }

        let ch = self.current_char();
        if ch.is_alphabetic() || ch == '_' {
            return self.lex_identifier_or_keyword();
        } else if ch.is_digit(10) {
            return self.lex_number();
        } else if ch == '"' {
            return self.lex_string();
        } else if "+-*/=();{}[]<>,&|!".contains(ch) {
            self.advance();
            return Token::Punctuator(ch);
        } else {
            panic!("Unexpected character: {}", ch);
        }
    }

    fn current_char(&self) -> char {
        self.input.as_bytes()[self.position] as char
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.current_char().is_whitespace() {
            self.advance();
        }
    }

    fn lex_identifier_or_keyword(&mut self) -> Token {
        let mut id = String::new();
        while self.position < self.input.len() && (self.current_char().is_alphanumeric() || self.current_char() == '_') {
            id.push(self.current_char());
            self.advance();
        }
        if ["int", "return", "if", "else", "while", "for"].contains(&id.as_str()) {
            Token::Keyword(id)
        } else {
            Token::Identifier(id)
        }
    }

    fn lex_number(&mut self) -> Token {
        let mut num = 0i64;
        while self.position < self.input.len() && self.current_char().is_digit(10) {
            num = num * 10 + self.current_char().to_digit(10).unwrap() as i64;
            self.advance();
        }
        Token::Number(num)
    }

    fn lex_string(&mut self) -> Token {
        self.advance(); // skip opening "
        let mut s = String::new();
        while self.position < self.input.len() && self.current_char() != '"' {
            s.push(self.current_char());
            self.advance();
        }
        self.advance(); // skip closing "
        Token::StringLiteral(s)
    }
}

#[derive(Debug)]
enum ASTNode {
    Program(Vec<ASTNode>),
    Function(String, Vec<ASTNode>),
    Return(Box<ASTNode>),
    BinaryOp(char, Box<ASTNode>, Box<ASTNode>),
    Number(i64),
    Identifier(String),
    // Add more as needed for full C-like support
}

struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser {
    fn new(input: String) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token();
        Parser { lexer, current_token }
    }

    fn eat(&mut self, expected: Token) {
        if self.current_token == expected {
            self.current_token = self.lexer.next_token();
        } else {
            panic!("Expected {:?}, got {:?}", expected, self.current_token);
        }
    }

    fn parse(&mut self) -> ASTNode {
        let mut functions = Vec::new();
        while self.current_token != Token::EOF {
            functions.push(self.parse_function());
        }
        ASTNode::Program(functions)
    }

    fn parse_function(&mut self) -> ASTNode {
        self.eat(Token::Keyword("int".to_string()));
        if let Token::Identifier(name) = self.current_token.clone() {
            self.eat(Token::Identifier(name.clone()));
            self.eat(Token::Punctuator('('));
            self.eat(Token::Punctuator(')'));
            self.eat(Token::Punctuator('{'));
            let mut statements = Vec::new();
            while self.current_token != Token::Punctuator('}') {
                statements.push(self.parse_statement());
            }
            self.eat(Token::Punctuator('}'));
            ASTNode::Function(name, statements)
        } else {
            panic!("Expected identifier");
        }
    }

    fn parse_statement(&mut self) -> ASTNode {
        if self.current_token == Token::Keyword("return".to_string()) {
            self.eat(Token::Keyword("return".to_string()));
            let expr = self.parse_expr();
            self.eat(Token::Punctuator(';'));
            ASTNode::Return(Box::new(expr))
        } else {
            panic!("Unsupported statement");
        }
    }

    fn parse_expr(&mut self) -> ASTNode {
        let mut node = self.parse_primary();
        while let Token::Punctuator(op) = self.current_token {
            if op == '+' || op == '-' || op == '*' || op == '/' {
                self.eat(Token::Punctuator(op));
                let right = self.parse_primary();
                node = ASTNode::BinaryOp(op, Box::new(node), Box::new(right));
            } else {
                break;
            }
        }
        node
    }

    fn parse_primary(&mut self) -> ASTNode {
        match self.current_token.clone() {
            Token::Number(n) => {
                self.eat(Token::Number(n));
                ASTNode::Number(n)
            }
            Token::Identifier(id) => {
                self.eat(Token::Identifier(id.clone()));
                ASTNode::Identifier(id)
            }
            _ => panic!("Unexpected token in primary: {:?}", self.current_token),
        }
    }
}

struct CodeGenerator {
    module: ObjectModule,
    func_builder_ctx: FunctionBuilderContext,
    data_ctx: DataContext,
    variables: HashMap<String, Variable>,
    var_index: usize,
}

impl CodeGenerator {
    fn new() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let isa_builder = isa::lookup(triple!("x86_64-unknown-linux-gnu")).unwrap(); // For Linux
        // For Windows, we can adjust based on OS
        let isa = isa_builder.finish(settings::Flags::new(flag_builder)).unwrap();

        let builder = ObjectBuilder::new(isa, "vira_module".to_owned(), cranelift_module::default_libcall_names()).unwrap();
        let module = ObjectModule::new(builder);

        CodeGenerator {
            module,
            func_builder_ctx: FunctionBuilderContext::new(),
            data_ctx: DataContext::new(),
            variables: HashMap::new(),
            var_index: 0,
        }
    }

    fn generate(&mut self, ast: &ASTNode) -> Vec<u8> {
        match ast {
            ASTNode::Program(functions) => {
                for func in functions {
                    self.generate_function(func);
                }
            }
            _ => panic!("Expected Program"),
        }

        let product = self.module.finish();
        product.object.write().unwrap()
    }

    fn generate_function(&mut self, func: &ASTNode) {
        if let ASTNode::Function(name, statements) = func {
            let mut sig = self.module.make_signature();
            sig.returns.push(AbiParam::new(types::I32)); // int return
            let func_id = self.module.declare_function(name, Linkage::Export, &sig).unwrap();
            let mut builder = FunctionBuilder::new(&mut sig, &mut self.func_builder_ctx);

            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            for stmt in statements {
                self.generate_statement(stmt, &mut builder);
            }

            // Default return 0 if no return
            let zero = builder.ins().iconst(types::I32, 0);
            builder.ins().return_(&[zero]);

            builder.finalize();

            self.module.define_function(func_id, &mut self.data_ctx, &mut builder.func, &mut NullTrapSink {}, &mut NullStackMapSink {}).unwrap();
            self.data_ctx.clear();
        } else {
            panic!("Expected Function");
        }
    }

    fn generate_statement(&mut self, stmt: &ASTNode, builder: &mut FunctionBuilder) {
        match stmt {
            ASTNode::Return(expr) => {
                let val = self.generate_expr(expr, builder);
                builder.ins().return_(&[val]);
            }
            _ => panic!("Unsupported statement"),
        }
    }

    fn generate_expr(&mut self, expr: &ASTNode, builder: &mut FunctionBuilder) -> Value {
        match expr {
            ASTNode::Number(n) => builder.ins().iconst(types::I32, *n),
            ASTNode::Identifier(id) => {
                if let Some(var) = self.variables.get(id) {
                    builder.use_var(*var)
                } else {
                    panic!("Undefined variable: {}", id);
                }
            }
            ASTNode::BinaryOp(op, left, right) => {
                let lhs = self.generate_expr(left, builder);
                let rhs = self.generate_expr(right, builder);
                match op {
                    '+' => builder.ins().iadd(lhs, rhs),
                    '-' => builder.ins().isub(lhs, rhs),
                    '*' => builder.ins().imul(lhs, rhs),
                    '/' => builder.ins().sdiv(lhs, rhs),
                    _ => panic!("Unsupported op: {}", op),
                }
            }
            _ => panic!("Unsupported expr"),
        }
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: compiler <input.vira> <output.o>");
        return Ok(());
    }

    let input_path = &args[1];
    let output_path = &args[2];

    let input = fs::read_to_string(input_path)?;
    let mut parser = Parser::new(input);
    let ast = parser.parse();

    let mut generator = CodeGenerator::new();
    let obj_bytes = generator.generate(&ast);

    let mut file = File::create(output_path)?;
    file.write_all(&obj_bytes)?;

    // For Windows support, we can detect OS and adjust triple
    let os = env::consts::OS;
    if os == "windows" {
        // Adjust ISA to x86_64-pc-windows-msvc or similar
    }

    // Invoke linker to create executable
    let linker = if os == "linux" {
        "gcc"
    } else if os == "windows" {
        "link.exe" // or adjust
    } else {
        panic!("Unsupported OS");
    };

    let status = Command::new(linker)
        .arg(output_path)
        .arg("-o")
        .arg("a.out") // or exe on Windows
        .status()?;

    if !status.success() {
        panic!("Linking failed");
    }

    Ok(())
}
