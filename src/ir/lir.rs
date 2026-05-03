use crate::bytecode::common::bytecode::LuauBuiltinFunction;
use std::collections::HashMap;

pub type Table = Vec<Box<str>>;
pub type ConstantTable = HashMap<Box<str>, Value>;

pub struct Block {
    pub instructions: Vec<Instruction>,
}

/// Represents a data primitive or a reference within the VM state.
pub enum Value {
    /// A virtual machine register index.
    Register(Register),
    Boolean(bool),
    Number(f32),
    String(Box<str>),
    /// A static array of identifiers/strings.
    Table(Box<Table>),
    /// A map of constant identifiers to their resolved values.
    ConstantTable(Box<ConstantTable>),
}

/// A handle to a virtual machine register, used for local variable storage and temporaries.
pub struct Register {
    /// The raw index in the VM stack frame.
    pub index: usize,
    /// The debug name or original source variable name, if available.
    pub name: String,
}

/// Defines the target of a function call.
pub enum CallFunction {
    /// Call a function currently stored in a register (e.g., a local or passed argument).
    Register(Register),

    /// Call a method or property-based function (e.g., `math.abs`).
    /// * `source`: The base object/table containing the function.
    /// * `name`: A `Value` (typically a Constant String) representing the key to look up.
    Name { source: Register, name: Value },

    /// Call a Luau-optimized builtin function directly by its internal ID.
    Fast { id: LuauBuiltinFunction },
}

/// Standard binary and unary mathematical operations.
pub enum Arithmetic {
    Add,  // +
    Sub,  // -
    Mul,  // *
    Div,  // /
    IDiv, // // (Floor division)
    Mod,  // %
    Pow,  // ^
    Neg,  // - (Unary)
}

/// The core instruction set for the Intermediate Representation.
pub enum Instruction {
    /// Moves a literal or register value into a target register.
    Load { target: Register, source: Value },

    /// Retrieves a value from the global environment by name.
    GlobalGet { target: Register, source: Box<str> },

    /// Assigns the value in a register to a global variable name.
    GlobalSet { target: Box<str>, source: Register },

    /// Retrieves a value from the function's upvalue (closure-captured) table.
    UpvalueGet { target: Register, source: usize },

    /// Sets a value in the function's upvalue table.
    UpvalueSet { target: usize, source: Register },

    /// Closes and captures all active upvalues from the stack into the target register's closure.
    UpvalueMigrateAll { target: Register },

    /// Performs an optimized import lookup (e.g., nested globals like `Library.Module.Func`).
    /// `source` must resolve to a valid index in the constant table.
    ImportGet { target: Register, source: Value },

    /// Performs a table lookup: `target = source[index]`
    TableGet {
        target: Register,
        source: Register,
        index: Value,
    },

    /// Performs a table assignment: `source[index] = value_in_target`
    /// Note: In many IRs, 'target' is used as the value to be stored.
    TableSet {
        target: Register,
        source: Register,
        index: Value,
    },

    /// Instantiates a new closure from a function prototype index.
    ClosureNew { target: Register, closure: i16 },

    /// Creates a copy of an existing closure found in the constant table.
    ClosureDuplicate { target: Register, closure: Value },

    /// Executes a function call.
    /// * `function`: The resolved callable entity.
    /// * `sources`: The list of registers containing arguments.
    Call {
        function: CallFunction,
        sources: Vec<Register>,
    },

    /// Performs arithmetic: `target = source op source2`.
    /// If `source2` is None, the operation is treated as Unary (e.g., Neg).
    Arithmetic {
        op: Arithmetic,
        target: Register,
        source: Register,
        source2: Option<Value>,
    },

    /// Logical AND: `target = source and source2`.
    And {
        target: Register,
        source: Register,
        source2: Value,
    },

    /// Logical OR: `target = source or source2`.
    Or {
        target: Register,
        source: Register,
        source2: Value,
    },

    /// Logical NOT: `target = not source`.
    Not { target: Register, source: Register },

    /// Unary Negative: `target = -source`.
    Negative { target: Register, source: Register },

    /// Length operator: `target = #source`.
    Length { target: Register, source: Register },

    /// String concatenation: `target = ..sources[0] .. sources[n]`.
    Concat {
        target: Register,
        sources: Vec<Register>,
    },
}
