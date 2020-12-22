use crate::{
    runtime::{
        Data, Instruction, InstructionCall, InstructionMap, InstructionName, Labels, Program,
        RustFunction, RustInstruction,
    },
    tokenizer::*,
};
use regex_lexer::Tokens;

lazy_static::lazy_static! {
    static ref EXIT_NAME: InstructionName = InstructionName::new("exit".to_string());
}

//TODO Add heartfelt instruction support

#[derive(Default)]
pub struct Parser {
    available_instructions: InstructionMap,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse<'a, 't: 'a>(&self, text: &'t str) -> Result<(Program, Labels), ParserError<'a>> {
        let mut program = Program::new();
        let mut labels = Labels::default();

        self.parse_text(text, (&mut program, &mut labels))
            .map(|_| (program, labels))
    }

    pub fn parse_text<'a, 't: 'a>(
        &self,
        text: &'t str,
        runtime_values: (&mut Program, &mut Labels),
    ) -> Result<(), ParserError<'a>> {
        let lexer = build_lexer()?;
        let tokens = lexer.tokens(text);

        self.parse_tokens(tokens, runtime_values)
    }

    pub fn parse_tokens<'a>(
        &self,
        mut tokens: Tokens<Token<'a>>,
        runtime_values: (&mut Program, &mut Labels),
    ) -> Result<(), ParserError<'a>> {
        //let mut unfinished_instructions = vec![];
        let mut current_instruction: Option<InstructionCall> = None;

        while let Some(tok) = tokens.next() {
            match tok {
                Token::Text(text) => {
                    if let Some(inst) = &mut current_instruction {
                        inst.args.push(Data::Text(text.to_string()));
                    } else {
                        let (k, v) = self
                            .available_instructions
                            .get_key_value(text)
                            .ok_or(ParserError::UnfoundInstruction(text))?;
                        current_instruction = Some(InstructionCall::new(v.clone(), k.clone()))
                    }
                }
                Token::Data(data) => {
                    if let Some(inst) = &mut current_instruction {
                        inst.args.push(token_to_data(data))
                    } else {
                        return Err(ParserError::InvalidInstruction(Token::Data(data)));
                    }
                }
                Token::NewLine | Token::Symbol(SymbolToken::Semicolon) => {
                    if let Some(inst) = current_instruction.take() {
                        runtime_values.0.push(inst)
                    }
                }
                Token::Label(lbl) => {
                    if let Some(inst) = current_instruction.take() {
                        runtime_values.0.push(inst)
                    }

                    runtime_values
                        .1
                        .insert(lbl.to_string(), runtime_values.0.len());
                }
                Token::Keyword(KeywordToken::Exit) => {
                    if let Some(inst) = &mut current_instruction {
                        inst.args.push(Data::Text("exit".to_string()));
                    } else {
                        current_instruction =
                            Some(InstructionCall::new(Instruction::Exit, EXIT_NAME.clone()));
                    }
                }
                t => return Err(ParserError::NotAllowed(t)),
            }
        }

        if let Some(inst) = current_instruction {
            runtime_values.0.push(inst)
        }

        Ok(())
    }

    pub fn add_instruction(&mut self, key: String, inst: Instruction) {
        let inst_name = InstructionName::new(key);
        self.available_instructions.insert(inst_name, inst);
    }

    pub fn add_rust_function(&mut self, key: String, fun: RustFunction) {
        let inst_name = InstructionName::new(key);
        self.available_instructions
            .insert(inst_name, Instruction::RustFunction(fun));
    }

    pub fn add_rust_instruction(&mut self, key: String, inst: RustInstruction) {
        let inst_name = InstructionName::new(key);
        self.available_instructions
            .insert(inst_name, Instruction::RustInstruction(inst));
    }

    pub fn set_instructions(&mut self, map: InstructionMap) {
        self.available_instructions = map;
    }
}

fn token_to_data(tok: DataToken) -> Data {
    match tok {
        DataToken::Bool(b) => Data::Bool(b),
        DataToken::Float(f) => Data::Float(f),
        DataToken::String(s) => Data::HString(s),
        DataToken::Character(c) => Data::Char(c),
        DataToken::None => Data::None,
        DataToken::Integer(i) => Data::Integer(i),
    }
}

#[derive(Debug)]
pub enum ParserError<'a> {
    NotAllowed(Token<'a>),
    InvalidInstruction(Token<'a>),
    RegexError(regex::Error),
    UnfoundInstruction(&'a str),
}

impl From<regex::Error> for ParserError<'_> {
    fn from(e: regex::Error) -> Self {
        Self::RegexError(e)
    }
}
