use std::fs;
use std::sync::Arc;
use hematita::{ast::{lexer, parser}, compiler, lua_lib, lua_tuple, lua_table, lua_value};
use hematita::vm::value::{Table, Value};
use clap::Parser;
use hematita::vm::VirtualMachine;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// File path to execute
    file: String
}

type LuaStack<'a> = Arc<Table<'a>>;
type LuaBuiltinResult<'a> = Result<LuaStack<'a>, String>;
type LuaResult<'a> = Result<LuaStack<'a>, LuaError>;

fn show_error(text: String) {
    eprintln!("\x1b[31;1merror:\x1b[0m {text}")
}

#[derive(Debug)]
enum LuaError {
    Runtime(String),
    Parse(hematita::ast::Error)
}

impl From<hematita::ast::Error> for LuaError {
    fn from(value: hematita::ast::Error) -> Self {
        Self::Parse(value)
    }
}

impl From<String> for LuaError {
    fn from(value: String) -> Self {
        Self::Runtime(value)
    }
}

fn run<'a>(code: &'a str, virtual_machine: VirtualMachine<'a>) -> LuaResult<'a> {
    let lexer = lexer::Lexer {
        source: code.chars().peekable()
    }.peekable();
    let parsed = parser::parse_block(&mut parser::TokenIterator(lexer))?;
    let compiled = compiler::compile_block(&parsed);

    let result = virtual_machine.execute(&compiled.into(), lua_tuple![].arc())?;
    Ok(result)
}

fn debug<'a>(args: LuaStack, _vm: &VirtualMachine) -> LuaBuiltinResult<'a> {
    Ok(Arc::new(lua_table!(Value::String(format!("{:?}", args).into_boxed_str()))))
}

fn get_globals<'a>() -> LuaStack<'a> {
    let global = lua_lib::standard_globals();
    let mut data = global.data.lock().unwrap();

    data.insert(lua_value!("debug"), Value::NativeFunction(&debug));

    drop(data);

    global
}

fn main() {
    let args = Args::parse();

    match fs::read_to_string(args.file) {
        Ok(mut f) => {
            let virtual_machine = VirtualMachine::new(get_globals());

            if f.trim().starts_with("#!") {
                let mut lines = f.trim().lines();
                lines.next();
                f = lines.collect::<Vec<&str>>().join("\n")
            }
            if let Err(e) = run(&f, virtual_machine) {
                match e {
                    LuaError::Runtime(e) => show_error(format!("runtime error: {e}")),
                    LuaError::Parse(e) => show_error(format!("parse error: {e}"))
                }
            }
        }
        Err(e) => show_error(format!("load error: {e}"))
    }
}
