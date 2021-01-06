pub mod runtime;
pub mod tokenizer;
pub mod parser;

pub use ahash::RandomState as AHasherBuilder;

pub use parser::Parser;
pub use runtime::*;


pub fn create_runtime(code: &str, mut instructions: InstructionMap) -> Result<Runtime, parser::ParserError> {
    let mut  parser = Parser::new();

    if !instructions.contains_key("goto") {
        instructions.insert(InstructionName::new("goto".to_string()),
        Instruction::RustInstruction(goto));
    }

    parser.set_instructions(instructions);
    
    let mut runtime = Runtime::new();

    let (program, labels) = parser.parse(code)?;

    runtime.set_program(program);
    runtime.set_labels(labels);

    Ok(runtime)
}

pub fn goto(args: Args, runtime: &Runtime) -> Data {
    let arg = &args[0];

    match arg {
        Data::HString(s) => {runtime.goto_label(s);},
        Data::Text(t) => {runtime.goto_label(t);},
        Data::Integer(i) => {runtime.goto(*i as usize);},
        _ => (),
    }

    Data::None
}

#[macro_export]
macro_rules! rust_fn {
    ($fun:path) => {
        $crate::Instruction::RustFunction($fun)
    };
}

#[macro_export]
macro_rules! rust_inst {
    ($fun:path) => {
        $crate::Instruction::RustInstruction($fun)
    };
}

#[macro_export]
macro_rules! add_fn {
    ($parser:ident, $name:expr, $fun:path) => {
        $parser.add_rust_function(String::from($name), heartfelt::FuncContainer::new($fun))
    };
}

#[macro_export]
macro_rules! add_inst {
    ($parser:ident, $name:expr, $fun:path) => {
        $parser.add_rust_instruction(String::from($name), heartfelt::FuncContainer::new($fun))
    };
}
