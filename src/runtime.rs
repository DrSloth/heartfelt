use ahash::RandomState as AHasherBuilder;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::{
    cell::{Cell, UnsafeCell},
    collections::HashMap,
    io::Error as IoError,
    sync::Arc,
};

macro_rules! dump_program {
    ($w:ident, $prog:ident) => {{
        let mut lbl_iter: Vec<(&String, &usize)> = $prog.labels.iter().collect();
        lbl_iter.sort_by(|(_,v1), (_,v2)| v1.cmp(v2));
        let mut lbl_iter = lbl_iter.iter();
        let mut cur_lbl = lbl_iter.next();
        let mut next_idx = 0;

        for (i, instruction) in $prog.program.iter().enumerate() {
            if i == next_idx {
                if let Some(entry) = cur_lbl {
                    if i == *entry.1 {
                        writeln!($w, "{}:", entry.0)?;
                        cur_lbl = lbl_iter.next();
                        next_idx = cur_lbl.map(|lbl| *lbl.1).unwrap_or(0);
                    } else {
                        next_idx = *entry.1
                    }
                }
            }

            writeln!($w, "{}", instruction)?;
        }

        while let Some(lbl) = cur_lbl {
            writeln!($w, "{}:", lbl.0)?;
            cur_lbl = lbl_iter.next();
        }

        Ok(())
    }};
}

pub type Args<'a> = &'a [Data];
pub type ArgContainer = Vec<Data>;
pub type RustFunction = FuncContainer<dyn Fn(Args) -> Data>;
pub type RustInstruction = FuncContainer<dyn Fn(Args, &Runtime) -> Data>;
pub type HeartfeltInstruction = Vec<Instruction>;
pub type FuncContainer<T> = Arc<T>;
pub type Program = Vec<InstructionCall>;
pub type Labels = HashMap<String, usize, AHasherBuilder>;
pub type InstructionMap = HashMap<InstructionName, Instruction, AHasherBuilder>;

#[derive(Default)]
pub struct Runtime {
    program: Program,
    data_map: UnsafeCell<HashMap<String, Data, AHasherBuilder>>,
    labels: Labels,
    pc: Cell<usize>,
}

/// API Functions
impl Runtime {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(&mut self) -> Data {
        unsafe { self.run_unsafe() }
    }

    pub unsafe fn run_unsafe(&self) -> Data {
        while self.pc.get() < self.program.len() {
            match self.program[self.pc.get()].get_id() {
                0 => {
                    self.rust_function();
                }
                1 => {
                    self.rust_instruction();
                }
                2 => self.heartfelt_instruction(),
                3 => return self.exit(),
                _ => unreachable!(),
            }

            self.pc.set(self.pc.get() + 1)
        }

        Data::None
    }

    pub fn set_program(&mut self, program: Program) {
        self.program = program;
    }

    pub fn add_instructions(&mut self, mut instructions: Program) {
        self.program.append(&mut instructions);
    }

    pub fn prepend_instructions(&mut self, mut instructions: Program) {
        for lbl in self.labels.values_mut() {
            *lbl += instructions.len();
        }

        std::mem::swap(&mut self.program, &mut instructions);

        self.program.append(&mut instructions);
    }

    pub fn clear_program(&mut self) {
        self.pc.set(0);
        self.program.clear();
    }

    pub fn clear(&mut self) {
        self.clear_program();
    }

    pub fn reset_program(&mut self) {
        self.pc.set(0);
    }

    pub fn reset(&mut self) {
        self.reset_program();
    }

    pub fn take_program(&mut self) -> Program {
        std::mem::take(&mut self.program)
    }

    pub fn swap_program(&mut self, new_program: &mut Program) {
        std::mem::swap(&mut self.program, new_program)
    }

    /// Returns the previous value associated with key
    pub fn def_var(&self, key: String, value: Data) -> Option<Data> {
        unsafe { (*self.data_map.get()).insert(key, value) }
    }

    /// Returns true if value existed false otherwise
    pub fn set_var(&self, key: &str, value: Data) -> bool {
        unsafe {
            if let Some(d) = (*self.data_map.get()).get_mut(key) {
                *d = value
            } else {
                return false;
            }
        }
        true
    }

    pub fn get_var_ref(&self, key: &str) -> Option<&Data> {
        unsafe { (*self.data_map.get()).get(key) }
    }

    pub unsafe fn get_var_mut_unsafe(&self, key: &str) -> Option<&mut Data> {
        (*self.data_map.get()).get_mut(key)
    }

    pub fn get_var_mut(&mut self, key: &str) -> Option<&mut Data> {
        unsafe { (*self.data_map.get()).get_mut(key) }
    }

    pub fn get_var(&self, key: &str) -> Option<Data> {
        unsafe { (*self.data_map.get()).get(key).map(|d| d.clone()) }
    }

    pub fn is_var_defined(&self, key: &str) -> bool {
        unsafe { (*self.data_map.get()).contains_key(key) }
    }

    pub fn goto(&self, instruction_idx: usize) {
        self.pc.set(instruction_idx.overflowing_sub(1).0)
    }

    pub fn goto_label(&self, label: &str) -> bool {
        if let Some(l) = self.labels.get(label) {
            self.goto(*l);

            return true;
        }

        false
    }

    pub fn add_label(&mut self, label: String, instruction_idx: usize) {
        self.labels.insert(label, instruction_idx);
    }

    pub fn set_labels(&mut self, labels: Labels) {
        self.labels = labels;
    }

    pub fn dump(&self) -> Result<String, std::fmt::Error> {
        let mut s = String::new();
        self.fmt_to(&mut s)?;
        Ok(s)
    }

    pub fn dump_to(&self, w: &mut impl std::io::Write) -> Result<(), IoError> {
        dump_program!(w, self)
    }

    pub fn fmt_to(&self, w: &mut impl std::fmt::Write) -> FmtResult {
        dump_program!(w, self)
    }
}

/// Internal Functions
impl Runtime {
    fn rust_function(&self) -> Data {
        let call = &self.program[self.pc.get()];
        if let Instruction::RustFunction(fun) = &call.inst {
            fun(&call.args)
        } else {
            unreachable!()
        }
    }

    fn rust_instruction(&self) -> Data {
        let call = &self.program[self.pc.get()];
        if let Instruction::RustInstruction(inst) = &call.inst {
            inst(&call.args, self)
        } else {
            unreachable!()
        }
    }

    fn heartfelt_instruction(&self) {
        todo!("heartfelt functions are not supported in heartfelt 0.1.x")
    }

    fn exit(&self) -> Data {
        let call = &self.program[self.pc.get()];
        if let Instruction::Exit = call.inst {
            Data::Array(call.args.clone())
        } else {
            unreachable!()
        }
    }
}

#[derive(Clone)]
pub enum Instruction {
    RustFunction(RustFunction),
    RustInstruction(RustInstruction),
    HeartfeltInstruction(HeartfeltInstruction),
    Exit,
}

#[derive(Clone)]
pub struct InstructionCall {
    pub args: ArgContainer,
    pub inst: Instruction,
    pub name: InstructionName,
}

//TODO Documentation
impl Display for InstructionCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} ", self.name)?;
        dump_data_arr(&self.args, f)
    }
}

impl InstructionCall {
    pub fn new_with_args(inst: Instruction, args: ArgContainer, name: InstructionName) -> Self {
        Self { inst, args, name }
    }

    pub fn new(inst: Instruction, name: InstructionName) -> Self {
        Self {
            inst,
            args: Default::default(),
            name,
        }
    }

    fn get_id(&self) -> u8 {
        match self.inst {
            Instruction::RustFunction(_) => 0,
            Instruction::RustInstruction(_) => 1,
            Instruction::HeartfeltInstruction(_) => 2,
            Instruction::Exit => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Data {
    Integer(i64),
    Float(f64),
    HString(String),
    Text(String),
    Bool(bool),
    Char(char),
    Array(Vec<Self>),
    None,
}

impl Display for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use Data::*;
        match self {
            Integer(i) => write!(f, "{}", i),
            Float(fl) => write!(f, "{}", fl),
            HString(s) => write!(f, "{}", s),
            Text(t) => write!(f, "{}", t),
            Bool(b) => write!(f, "{}", b),
            Char(c) => write!(f, "{}", c),
            Array(v) => fmt_data_arr(v, f),
            None => write!(f, "NONE"),
        }
    }
}

impl Data {
    fn dump(&self, f: &mut Formatter<'_>) -> FmtResult {
        use Data::*;
        match self {
            Integer(i) => write!(f, "{} ", i),
            Float(fl) => write!(f, "{} ", fl),
            HString(s) => write!(f, "\"{}\" ", s),
            Text(t) => write!(f, "{} ", t),
            Bool(b) => write!(f, "{} ", b),
            Char(c) => write!(f, "{} ", c),
            Array(v) => fmt_data_arr(v, f),
            None => write!(f, "NONE "),
        }
    }
}

fn fmt_data_arr(v: &[Data], f: &mut Formatter<'_>) -> FmtResult {
    v.iter()
        .map(|d| write!(f, "{} ", d))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

fn dump_data_arr(v: &[Data], f: &mut Formatter<'_>) -> FmtResult {
    v.iter()
        .map(|d| d.dump(f))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq)]
pub struct InstructionName(Arc<String>);

impl InstructionName {
    pub fn new(name: String) -> Self {
        Self(Arc::new(name))
    }
}

impl std::borrow::Borrow<str> for InstructionName {
    fn borrow<'a>(&'a self) -> &'a str {
        &self.0[..]
    }
}

impl From<Arc<String>> for InstructionName {
    fn from(arc: Arc<String>) -> Self {
        Self(arc)
    }
}

impl From<String> for InstructionName {
    fn from(s: String) -> Self {
        Self(Arc::new(s))
    }
}

impl From<&'_ str> for InstructionName {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

impl Display for InstructionName {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", *self.0)
    }
}

pub fn fmt_program(program: &[InstructionCall], w: &mut impl std::fmt::Write) -> FmtResult {
    for instruction in program.iter() {
        write!(w, "{}", instruction)?;
    }

    Ok(())
}

pub fn write_program(
    program: &[InstructionCall],
    w: &mut impl std::io::Write,
) -> Result<(), IoError> {
    for instruction in program.iter() {
        write!(w, "{}", instruction)?;
    }

    Ok(())
}
