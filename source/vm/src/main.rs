use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, Read, Write};
use std::path::Path;
use std::process::exit;

// Simple Virtual Machine for Vira Bytecode in Rust
// Executes .object files produced by the compiler
// Stack-based VM with basic operations
// Supports: PushNum, PushStr, Add, Sub, Mul, Div, Store, Load, Write, Halt
// Variables stored in a HashMap<String, Value>
// For simplicity, values are f64 or String, but operations assume compatible types

#[derive(Debug, Clone)]
enum Value {
    Num(f64),
    Str(String),
}

struct VM {
    stack: Vec<Value>,
    vars: HashMap<String, Value>,
}

impl VM {
    fn new() -> Self {
        VM {
            stack: Vec::new(),
            vars: HashMap::new(),
        }
    }

    fn run(&mut self, bytecode: &[u8]) {
        let mut pc = 0; // Program counter
        while pc < bytecode.len() {
            let opcode = bytecode[pc];
            pc += 1;
            match opcode {
                0 => { // PushNum
                    if pc + 8 > bytecode.len() {
                        panic!("Incomplete PushNum");
                    }
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&bytecode[pc..pc + 8]);
                    let val = f64::from_le_bytes(bytes);
                    self.stack.push(Value::Num(val));
                    pc += 8;
                }
                1 => { // PushStr
                    if pc + 4 > bytecode.len() {
                        panic!("Incomplete PushStr");
                    }
                    let mut len_bytes = [0u8; 4];
                    len_bytes.copy_from_slice(&bytecode[pc..pc + 4]);
                    let len = u32::from_le_bytes(len_bytes) as usize;
                    pc += 4;
                    if pc + len > bytecode.len() {
                        panic!("Incomplete string");
                    }
                    let str_val = String::from_utf8_lossy(&bytecode[pc..pc + len]).to_string();
                    self.stack.push(Value::Str(str_val));
                    pc += len;
                }
                2 => { // Add
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (Value::Num(x), Value::Num(y)) => self.stack.push(Value::Num(x + y)),
                        (Value::Str(x), Value::Str(y)) => self.stack.push(Value::Str(x + &y)),
                        _ => panic!("Type mismatch in Add"),
                    }
                }
                3 => { // Sub
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (Value::Num(x), Value::Num(y)) = (a, b) {
                        self.stack.push(Value::Num(x - y));
                    } else {
                        panic!("Type mismatch in Sub");
                    }
                }
                4 => { // Mul
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (Value::Num(x), Value::Num(y)) = (a, b) {
                        self.stack.push(Value::Num(x * y));
                    } else {
                        panic!("Type mismatch in Mul");
                    }
                }
                5 => { // Div
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (Value::Num(x), Value::Num(y)) = (a, b) {
                        self.stack.push(Value::Num(x / y));
                    } else {
                        panic!("Type mismatch in Div");
                    }
                }
                6 => { // Store
                    if pc + 4 > bytecode.len() {
                        panic!("Incomplete Store");
                    }
                    let mut len_bytes = [0u8; 4];
                    len_bytes.copy_from_slice(&bytecode[pc..pc + 4]);
                    let len = u32::from_le_bytes(len_bytes) as usize;
                    pc += 4;
                    if pc + len > bytecode.len() {
                        panic!("Incomplete name");
                    }
                    let name = String::from_utf8_lossy(&bytecode[pc..pc + len]).to_string();
                    pc += len;
                    let val = self.stack.pop().unwrap();
                    self.vars.insert(name, val);
                }
                7 => { // Load
                    if pc + 4 > bytecode.len() {
                        panic!("Incomplete Load");
                    }
                    let mut len_bytes = [0u8; 4];
                    len_bytes.copy_from_slice(&bytecode[pc..pc + 4]);
                    let len = u32::from_le_bytes(len_bytes) as usize;
                    pc += 4;
                    if pc + len > bytecode.len() {
                        panic!("Incomplete name");
                    }
                    let name = String::from_utf8_lossy(&bytecode[pc..pc + len]).to_string();
                    pc += len;
                    if let Some(val) = self.vars.get(&name) {
                        self.stack.push(val.clone());
                    } else {
                        panic!("Undefined variable: {}", name);
                    }
                }
                8 => { // Call (not implemented)
                    if pc + 8 > bytecode.len() {
                        panic!("Incomplete Call");
                    }
                    pc += 8; // Skip num args
                    println!("Warning: Call not supported");
                }
                9 => { // Write
                    let val = self.stack.pop().unwrap();
                    match val {
                        Value::Num(n) => println!("{}", n),
                        Value::Str(s) => println!("{}", s),
                    }
                }
                10 => { // Halt
                    break;
                }
                _ => panic!("Unknown opcode: {}", opcode),
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: vm <input.object>");
        exit(1);
    }

    let input_path = Path::new(&args[1]);
    let mut file = File::open(input_path).expect("Failed to open file");
    let mut bytecode = Vec::new();
    file.read_to_end(&mut bytecode).expect("Failed to read file");

    let mut vm = VM::new();
    vm.run(&bytecode);

    println!("Execution completed.");
                  }
