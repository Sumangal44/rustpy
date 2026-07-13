#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opcode {
    // Stack manipulation
    PopTop,

    // Variables and Constants
    LoadConst(usize), // index into constants pool
    LoadName(usize),  // index into names pool
    StoreName(usize), // index into names pool

    // Math operations
    BinaryAdd,
    BinarySubtract,
    BinaryMultiply,
    BinaryTrueDivide,
    BinaryFloorDivide,
    BinaryModulo,
    BinaryPower,

    // Comparisons
    CompareEq,
    CompareNotEq,
    CompareLt,
    CompareLtEq,
    CompareGt,
    CompareGtEq,

    // Control flow
    JumpForward(usize),    // offset
    PopJumpIfFalse(usize), // absolute target
    PopJumpIfTrue(usize),  // absolute target
    JumpAbsolute(usize),   // absolute target

    // Functions
    CallFunction(usize), // number of arguments
    ReturnValue,
}
