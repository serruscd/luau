use crate::bytecode::common::bytecode::{LuauBuiltinFunction, LuauOpcode};
use crate::bytecode::types::constant::Constant;
use crate::bytecode::types::instruction::Instruction as LuauInstruction;
use std::collections::HashMap;
use std::convert::TryFrom;

pub type Table = Vec<Box<str>>;
pub type ConstantTable = HashMap<Box<str>, Value>;

pub struct Block {
    pub instructions: Vec<Instruction>,
}

#[derive(Clone, Debug)]
pub enum Value {
    Nil,
    Register(Register),
    Boolean(bool),
    Number(f64),
    Integer(i64),
    String(Box<str>),
    Table(Box<Table>),
    ConstantTable(Box<ConstantTable>),
    ImportPath(Vec<Box<str>>),
    Vector(f32, f32, f32, f32),
    Closure(usize),
}

#[derive(Clone, Debug)]
pub struct Register {
    pub index: usize,
    pub name: String,
}

#[derive(Debug)]
pub enum CallFunction {
    Register(Register),
    Name { source: Register, name: Value },
    Fast { id: LuauBuiltinFunction },
}

#[derive(Debug)]
pub enum Arithmetic {
    Add,
    Sub,
    Mul,
    Div,
    IDiv,
    Mod,
    Pow,
    Neg,
}

#[derive(Debug)]
pub enum Instruction {
    NoOp,
    Break,

    // Memory & Variables
    Load {
        target: Register,
        source: Value,
    },
    GlobalGet {
        target: Register,
        source: Box<str>,
    },
    GlobalSet {
        target: Box<str>,
        source: Register,
    },
    UpvalueGet {
        target: Register,
        source: usize,
    },
    UpvalueSet {
        target: usize,
        source: Register,
    },
    UpvalueMigrateAll {
        target: Register,
    },
    ImportGet {
        target: Register,
        source: Value,
    },

    // Tables
    TableGet {
        target: Register,
        source: Register,
        index: Value,
    },
    TableSet {
        target: Register,
        source: Register,
        index: Value,
    },
    NewTable {
        target: Register,
        size: u8,
        array_size: u32,
    },
    DupTable {
        target: Register,
        template: Value,
    },
    SetList {
        target: Register,
        source_start: Register,
        count: u8,
        table_index: u32,
    },

    // Closures & Functions
    ClosureNew {
        target: Register,
        closure: usize,
    },
    ClosureDuplicate {
        target: Register,
        closure: Value,
    },
    Call {
        function: CallFunction,
        sources: Vec<Register>,
        returns: u8,
    },
    NameCall {
        target: Register,
        source: Register,
        name: Value,
    },
    Return {
        start: Register,
        count: u8,
    },
    FastCall {
        id: LuauBuiltinFunction,
        args: Vec<Value>,
    },
    GetVarArgs {
        target: Register,
        count: u8,
    },
    PrepVarArgs {
        fixed_args: u8,
    },
    Capture {
        type_: u8,
        source: u8,
    },

    // Math & Logic
    Arithmetic {
        op: Arithmetic,
        target: Register,
        source: Value,
        source2: Option<Value>,
    },
    And {
        target: Register,
        source: Register,
        source2: Value,
    },
    Or {
        target: Register,
        source: Register,
        source2: Value,
    },
    Not {
        target: Register,
        source: Register,
    },
    Negative {
        target: Register,
        source: Register,
    },
    Length {
        target: Register,
        source: Register,
    },
    Concat {
        target: Register,
        sources: Vec<Register>,
    },

    // Control Flow
    Jump {
        offset: i32,
    },
    JumpIf {
        condition: Register,
        offset: i32,
    },
    JumpIfNot {
        condition: Register,
        offset: i32,
    },
    JumpIfEq {
        reg1: Register,
        reg2: Value,
        offset: i32,
    },
    JumpIfLe {
        reg1: Register,
        reg2: Value,
        offset: i32,
    },
    JumpIfLt {
        reg1: Register,
        reg2: Value,
        offset: i32,
    },
    JumpIfNotEq {
        reg1: Register,
        reg2: Value,
        offset: i32,
    },
    JumpIfNotLe {
        reg1: Register,
        reg2: Value,
        offset: i32,
    },
    JumpIfNotLt {
        reg1: Register,
        reg2: Value,
        offset: i32,
    },

    // Loops
    ForNPrep {
        target: Register,
        offset: i32,
    },
    ForNLoop {
        target: Register,
        offset: i32,
    },
    ForGPrep {
        target: Register,
        offset: i32,
    },
    ForGLoop {
        target: Register,
        offset: i32,
        aux: u32,
    },
    ForGPrepINext {
        target: Register,
        offset: i32,
    },
    ForGPrepNext {
        target: Register,
        offset: i32,
    },
}

impl Block {
    pub fn from_luau(code: &[LuauInstruction], constants: &[Constant]) -> Self {
        let mut block = Block {
            instructions: Vec::new(),
        };

        let reg = |index: u8| Register {
            index: index as usize,
            name: format!("R{index}"),
        };

        // Recursive helper to transform raw constants to LIR values natively.
        fn constant_to_value(c: &Constant) -> Value {
            match c {
                Constant::Nil => Value::Nil,
                Constant::Boolean(b) => Value::Boolean(*b),
                Constant::Number(n) => Value::Number(*n),
                Constant::Integer(i) => Value::Integer(*i),
                Constant::String(s) => Value::String(s.clone()),
                Constant::Vector { x, y, z, w } => Value::Vector(*x, *y, *z, *w),
                Constant::Table(t) => Value::Table(Box::new(t.clone())),
                Constant::Import(m, n, a) => {
                    let mut path = vec![m.clone()];
                    if let Some(n) = n {
                        path.push(n.clone());
                    }
                    if let Some(a) = a {
                        path.push(a.clone());
                    }
                    Value::ImportPath(path)
                }
                Constant::ConstantTable(t) => {
                    let mut map = HashMap::new();
                    for (k, v) in t {
                        map.insert(k.clone(), constant_to_value(v));
                    }
                    Value::ConstantTable(Box::new(map))
                }
                Constant::Closure(c) => Value::Closure(*c),
            }
        }

        let get_const = |index: u32| -> Value {
            constants
                .get(index as usize)
                .map(constant_to_value)
                .unwrap_or(Value::Nil)
        };

        let get_builtin = |id: u8| -> LuauBuiltinFunction {
            LuauBuiltinFunction::try_from(id).unwrap_or(LuauBuiltinFunction::LBF_NONE)
        };

        for inst in code {
            let target = reg(inst.a);

            macro_rules! arith {
                ($op:ident, r, $r1:expr, $r2:expr) => {
                    Instruction::Arithmetic {
                        op: Arithmetic::$op,
                        target: target.clone(),
                        source: Value::Register(reg($r1)),
                        source2: Some(Value::Register(reg($r2))),
                    }
                };
                ($op:ident, k, $r1:expr, $k2:expr) => {
                    Instruction::Arithmetic {
                        op: Arithmetic::$op,
                        target: target.clone(),
                        source: Value::Register(reg($r1)),
                        source2: Some(get_const($k2 as u32)),
                    }
                };
                ($op:ident, rk, $k1:expr, $r2:expr) => {
                    Instruction::Arithmetic {
                        op: Arithmetic::$op,
                        target: target.clone(),
                        source: get_const($k1 as u32),
                        source2: Some(Value::Register(reg($r2))),
                    }
                };
            }

            let lir_inst = match inst.opcode {
                LuauOpcode::LOP_NOP => Instruction::NoOp,
                LuauOpcode::LOP_BREAK => Instruction::Break,
                LuauOpcode::LOP_LOADNIL => Instruction::Load {
                    target,
                    source: Value::Nil,
                },
                LuauOpcode::LOP_LOADB => {
                    block.instructions.push(Instruction::Load {
                        target: target.clone(),
                        source: Value::Boolean(inst.b != 0),
                    });
                    if inst.c > 0 {
                        block.instructions.push(Instruction::Jump {
                            offset: inst.c as i32,
                        });
                    }
                    continue; // Skip the default push to prevent duplicates
                }
                LuauOpcode::LOP_LOADN => Instruction::Load {
                    target,
                    source: Value::Number(inst.d as f64),
                },
                LuauOpcode::LOP_LOADK => Instruction::Load {
                    target,
                    source: get_const(inst.d as u32),
                },
                LuauOpcode::LOP_LOADKX => Instruction::Load {
                    target,
                    source: get_const(inst.aux.unwrap_or(0)),
                },
                LuauOpcode::LOP_MOVE => Instruction::Load {
                    target,
                    source: Value::Register(reg(inst.b)),
                },

                LuauOpcode::LOP_GETGLOBAL => {
                    if let Value::String(name) = get_const(inst.aux.unwrap_or(0)) {
                        Instruction::GlobalGet {
                            target,
                            source: name,
                        }
                    } else {
                        continue;
                    }
                }
                LuauOpcode::LOP_SETGLOBAL => {
                    if let Value::String(name) = get_const(inst.aux.unwrap_or(0)) {
                        Instruction::GlobalSet {
                            target: name,
                            source: target,
                        }
                    } else {
                        continue;
                    }
                }

                LuauOpcode::LOP_GETUPVAL => Instruction::UpvalueGet {
                    target,
                    source: inst.b as usize,
                },
                LuauOpcode::LOP_SETUPVAL => Instruction::UpvalueSet {
                    target: inst.b as usize,
                    source: target,
                },
                LuauOpcode::LOP_CLOSEUPVALS => Instruction::UpvalueMigrateAll { target },
                LuauOpcode::LOP_GETIMPORT => Instruction::ImportGet {
                    target,
                    source: get_const(inst.d as u32),
                },

                LuauOpcode::LOP_GETTABLE => Instruction::TableGet {
                    target,
                    source: reg(inst.b),
                    index: Value::Register(reg(inst.c)),
                },
                LuauOpcode::LOP_SETTABLE => Instruction::TableSet {
                    target: reg(inst.a),
                    source: reg(inst.b),
                    index: Value::Register(reg(inst.c)),
                },
                LuauOpcode::LOP_GETTABLEKS => Instruction::TableGet {
                    target,
                    source: reg(inst.b),
                    index: get_const(inst.aux.unwrap_or(0)),
                },
                LuauOpcode::LOP_SETTABLEKS => Instruction::TableSet {
                    target: reg(inst.a),
                    source: reg(inst.b),
                    index: get_const(inst.aux.unwrap_or(0)),
                },
                LuauOpcode::LOP_GETTABLEN => Instruction::TableGet {
                    target,
                    source: reg(inst.b),
                    index: Value::Number((inst.c as f64) + 1.0),
                },
                LuauOpcode::LOP_SETTABLEN => Instruction::TableSet {
                    target: reg(inst.a),
                    source: reg(inst.b),
                    index: Value::Number((inst.c as f64) + 1.0),
                },

                LuauOpcode::LOP_NEWCLOSURE => Instruction::ClosureNew {
                    target,
                    closure: inst.d as usize,
                },
                LuauOpcode::LOP_DUPCLOSURE => Instruction::ClosureDuplicate {
                    target,
                    closure: get_const(inst.d as u32),
                },
                LuauOpcode::LOP_NAMECALL => Instruction::NameCall {
                    target,
                    source: reg(inst.b),
                    name: get_const(inst.aux.unwrap_or(0)),
                },

                LuauOpcode::LOP_CALL => {
                    let mut sources = Vec::new();
                    if inst.b > 0 {
                        for i in 1..inst.b {
                            sources.push(reg(inst.a + i));
                        }
                    }
                    Instruction::Call {
                        function: CallFunction::Register(reg(inst.a)),
                        sources,
                        returns: inst.c,
                    }
                }
                LuauOpcode::LOP_RETURN => Instruction::Return {
                    start: target,
                    count: inst.b,
                },

                // Jumps
                LuauOpcode::LOP_JUMP | LuauOpcode::LOP_JUMPBACK => Instruction::Jump {
                    offset: inst.d as i32,
                },
                LuauOpcode::LOP_JUMPX => Instruction::Jump { offset: inst.e },
                LuauOpcode::LOP_JUMPIF => Instruction::JumpIf {
                    condition: target,
                    offset: inst.d as i32,
                },
                LuauOpcode::LOP_JUMPIFNOT => Instruction::JumpIfNot {
                    condition: target,
                    offset: inst.d as i32,
                },
                LuauOpcode::LOP_JUMPIFEQ => Instruction::JumpIfEq {
                    reg1: target,
                    reg2: Value::Register(reg(inst.aux.unwrap_or(0) as u8)),
                    offset: inst.d as i32,
                },
                LuauOpcode::LOP_JUMPIFLE => Instruction::JumpIfLe {
                    reg1: target,
                    reg2: Value::Register(reg(inst.aux.unwrap_or(0) as u8)),
                    offset: inst.d as i32,
                },
                LuauOpcode::LOP_JUMPIFLT => Instruction::JumpIfLt {
                    reg1: target,
                    reg2: Value::Register(reg(inst.aux.unwrap_or(0) as u8)),
                    offset: inst.d as i32,
                },
                LuauOpcode::LOP_JUMPIFNOTEQ => Instruction::JumpIfNotEq {
                    reg1: target,
                    reg2: Value::Register(reg(inst.aux.unwrap_or(0) as u8)),
                    offset: inst.d as i32,
                },
                LuauOpcode::LOP_JUMPIFNOTLE => Instruction::JumpIfNotLe {
                    reg1: target,
                    reg2: Value::Register(reg(inst.aux.unwrap_or(0) as u8)),
                    offset: inst.d as i32,
                },
                LuauOpcode::LOP_JUMPIFNOTLT => Instruction::JumpIfNotLt {
                    reg1: target,
                    reg2: Value::Register(reg(inst.aux.unwrap_or(0) as u8)),
                    offset: inst.d as i32,
                },

                LuauOpcode::LOP_JUMPXEQKNIL
                | LuauOpcode::LOP_JUMPXEQKB
                | LuauOpcode::LOP_JUMPXEQKN
                | LuauOpcode::LOP_JUMPXEQKS => {
                    let aux = inst.aux.unwrap_or(0);
                    let not = (aux >> 31) != 0;
                    let val = match inst.opcode {
                        LuauOpcode::LOP_JUMPXEQKNIL => Value::Nil,
                        LuauOpcode::LOP_JUMPXEQKB => Value::Boolean((aux & 1) != 0),
                        _ => get_const(aux & 0xFFFFFF),
                    };
                    if not {
                        Instruction::JumpIfNotEq {
                            reg1: target,
                            reg2: val,
                            offset: inst.d as i32,
                        }
                    } else {
                        Instruction::JumpIfEq {
                            reg1: target,
                            reg2: val,
                            offset: inst.d as i32,
                        }
                    }
                }

                // Arithmetic
                LuauOpcode::LOP_ADD => arith!(Add, r, inst.b, inst.c),
                LuauOpcode::LOP_SUB => arith!(Sub, r, inst.b, inst.c),
                LuauOpcode::LOP_MUL => arith!(Mul, r, inst.b, inst.c),
                LuauOpcode::LOP_DIV => arith!(Div, r, inst.b, inst.c),
                LuauOpcode::LOP_MOD => arith!(Mod, r, inst.b, inst.c),
                LuauOpcode::LOP_POW => arith!(Pow, r, inst.b, inst.c),
                LuauOpcode::LOP_IDIV => arith!(IDiv, r, inst.b, inst.c),

                LuauOpcode::LOP_ADDK => arith!(Add, k, inst.b, inst.c),
                LuauOpcode::LOP_SUBK => arith!(Sub, k, inst.b, inst.c),
                LuauOpcode::LOP_MULK => arith!(Mul, k, inst.b, inst.c),
                LuauOpcode::LOP_DIVK => arith!(Div, k, inst.b, inst.c),
                LuauOpcode::LOP_MODK => arith!(Mod, k, inst.b, inst.c),
                LuauOpcode::LOP_POWK => arith!(Pow, k, inst.b, inst.c),
                LuauOpcode::LOP_IDIVK => arith!(IDiv, k, inst.b, inst.c),

                LuauOpcode::LOP_SUBRK => arith!(Sub, rk, inst.b, inst.c),
                LuauOpcode::LOP_DIVRK => arith!(Div, rk, inst.b, inst.c),

                LuauOpcode::LOP_AND => Instruction::And {
                    target,
                    source: reg(inst.b),
                    source2: Value::Register(reg(inst.c)),
                },
                LuauOpcode::LOP_OR => Instruction::Or {
                    target,
                    source: reg(inst.b),
                    source2: Value::Register(reg(inst.c)),
                },
                LuauOpcode::LOP_ANDK => Instruction::And {
                    target,
                    source: reg(inst.b),
                    source2: get_const(inst.c as u32),
                },
                LuauOpcode::LOP_ORK => Instruction::Or {
                    target,
                    source: reg(inst.b),
                    source2: get_const(inst.c as u32),
                },

                LuauOpcode::LOP_CONCAT => {
                    let sources = (inst.b..=inst.c).map(reg).collect();
                    Instruction::Concat { target, sources }
                }

                LuauOpcode::LOP_NOT => Instruction::Not {
                    target,
                    source: reg(inst.b),
                },
                LuauOpcode::LOP_MINUS => Instruction::Negative {
                    target,
                    source: reg(inst.b),
                },
                LuauOpcode::LOP_LENGTH => Instruction::Length {
                    target,
                    source: reg(inst.b),
                },

                LuauOpcode::LOP_NEWTABLE => Instruction::NewTable {
                    target,
                    size: inst.b,
                    array_size: inst.aux.unwrap_or(0),
                },
                LuauOpcode::LOP_DUPTABLE => Instruction::DupTable {
                    target,
                    template: get_const(inst.d as u32),
                },
                LuauOpcode::LOP_SETLIST => Instruction::SetList {
                    target,
                    source_start: reg(inst.b),
                    count: inst.c,
                    table_index: inst.aux.unwrap_or(0),
                },

                // Loop constructs
                LuauOpcode::LOP_FORNPREP => Instruction::ForNPrep {
                    target,
                    offset: inst.d as i32,
                },
                LuauOpcode::LOP_FORNLOOP => Instruction::ForNLoop {
                    target,
                    offset: inst.d as i32,
                },
                LuauOpcode::LOP_FORGPREP => Instruction::ForGPrep {
                    target,
                    offset: inst.d as i32,
                },
                LuauOpcode::LOP_FORGLOOP => Instruction::ForGLoop {
                    target,
                    offset: inst.d as i32,
                    aux: inst.aux.unwrap_or(0),
                },
                LuauOpcode::LOP_FORGPREP_INEXT => Instruction::ForGPrepINext {
                    target,
                    offset: inst.d as i32,
                },
                LuauOpcode::LOP_FORGPREP_NEXT => Instruction::ForGPrepNext {
                    target,
                    offset: inst.d as i32,
                },

                // Varargs & Builtins
                LuauOpcode::LOP_GETVARARGS => Instruction::GetVarArgs {
                    target,
                    count: inst.b,
                },
                LuauOpcode::LOP_PREPVARARGS => Instruction::PrepVarArgs { fixed_args: inst.a },
                LuauOpcode::LOP_CAPTURE => Instruction::Capture {
                    type_: inst.a,
                    source: inst.b,
                },

                LuauOpcode::LOP_FASTCALL1 => Instruction::FastCall {
                    id: get_builtin(inst.a),
                    args: vec![Value::Register(reg(inst.b))],
                },
                LuauOpcode::LOP_FASTCALL2 => Instruction::FastCall {
                    id: get_builtin(inst.a),
                    args: vec![
                        Value::Register(reg(inst.b)),
                        Value::Register(reg(inst.aux.unwrap_or(0) as u8)),
                    ],
                },
                LuauOpcode::LOP_FASTCALL2K => Instruction::FastCall {
                    id: get_builtin(inst.a),
                    args: vec![
                        Value::Register(reg(inst.b)),
                        get_const(inst.aux.unwrap_or(0)),
                    ],
                },
                LuauOpcode::LOP_FASTCALL3 => {
                    let aux = inst.aux.unwrap_or(0);
                    Instruction::FastCall {
                        id: get_builtin(inst.a),
                        args: vec![
                            Value::Register(reg(inst.b)),
                            Value::Register(reg((aux & 0xFF) as u8)),
                            Value::Register(reg(((aux >> 8) & 0xFF) as u8)),
                        ],
                    }
                }
                LuauOpcode::LOP_FASTCALL => Instruction::FastCall {
                    id: get_builtin(inst.a),
                    args: vec![],
                },

                LuauOpcode::LOP_COVERAGE
                | LuauOpcode::LOP_NATIVECALL
                | LuauOpcode::LOP_GETUDATAKS
                | LuauOpcode::LOP_SETUDATAKS
                | LuauOpcode::LOP_NAMECALLUDATA
                | LuauOpcode::LOP__COUNT => continue,
            };

            block.instructions.push(lir_inst);
        }

        block
    }
}
