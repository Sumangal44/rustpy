#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opcode {
    // Stack manipulation
    PopTop,

    // Variables and Constants
    LoadConst(usize), // index into constants pool
    LoadName(usize),  // index into names pool
    StoreName(usize), // index into names pool
    DeleteName(usize), // index into names pool
    DeleteAttr(String), // pops object, deletes attribute
    DeleteSubscript, // pops key, pops collection, deletes item

    // Math operations
    BinaryAdd,
    BinarySubtract,
    BinaryMultiply,
    BinaryTrueDivide,
    BinaryFloorDivide,
    BinaryModulo,
    BinaryPower,
    BinaryMatMul,
    BinaryBitAnd,
    BinaryBitOr,
    BinaryBitXor,
    BinaryLShift,
    BinaryRShift,
    UnaryNegative,
    UnaryPositive,
    UnaryNot,
    UnaryInvert,

    // Stack manipulation
    DupTop,

    // Comparisons
    CompareEq,
    CompareNotEq,
    CompareLt,
    CompareLtEq,
    CompareGt,
    CompareGtEq,
    CompareIn,
    CompareNotIn,
    CompareIs,
    CompareIsNot,

    // Control flow
    JumpForward(usize),    // offset
    PopJumpIfFalse(usize), // absolute target
    PopJumpIfTrue(usize),  // absolute target
    JumpAbsolute(usize),   // absolute target

    // Functions
    MakeFunction,        // Pops a code object and creates a function object
    CallFunction(usize), // number of positional arguments
    CallFunctionKw(usize), // number of total arguments. Top of stack is a tuple of kwarg names.
    CallFunctionEx(usize), // 0 if only *args on stack, 1 if both *args and **kwargs on stack.
    ReturnValue,
    YieldValue,

    // Data Structures
    BuildList(usize), // count
    BuildMap(usize),  // count (number of key-value pairs)
    BuildTuple(usize), // count
    BuildSet(usize),  // count (number of elements; 0 for comprehension)
    ListExtend,       // pops an iterable, pops a list, extends list, pushes list
    DictMerge,        // pops a dict, pops a dict, merges, pushes dict
    ListAppend,       // pops item, pops list, appends item, pushes list back
    MapAdd,           // pops value, pops key, pops dict, inserts key:value, pushes dict back
    SetAdd,           // pops item, pops set, adds item (no-op if duplicate), pushes set back
    BuildSlice,       // pops step (or None), stop (or None), start (or None), pushes PySlice
    BinarySubscript,  // pops index, pops collection, pushes item
    StoreSubscript,   // pops value, pops index, pops collection, stores item
    UnpackSequence(usize), // pops sequence, pushes individual items in reverse order

    // Control Flow
    GetIter,        // pops collection, pushes iterator
    ForIter(usize), // pops iterator, gets next. If some, pushes iterator then item. If none, pops iterator, jumps forward by offset.

    // Classes and Attributes
    BuildClass(usize), // number of base classes
    LoadAttr(String),  // pops object, pushes attribute
    StoreAttr(String), // pops object, pops value, sets attribute

    // Exceptions
    SetupExcept(usize), // pushes a block onto block stack, with target
    PopExcept,          // pops a block from block stack
    SetupFinally(usize), // pushes a finally block; on exception, saves exception and jumps to target
    PopFinally,          // pops a finally block from block stack
    EndFinally,          // end of finally block; re-raises pending exception if any
    Raise,              // pops an object, raises it as an exception
    TryEnd,             // marker for end of try structure

    // Context Managers
    SetupWith(usize), // pushes block, calls __enter__, pushes result
    WithCleanup,      // pops block, calls __exit__

    // Import
    ImportName(usize), // index into names; pops fromlist, level; pushes module
    ImportFrom(usize), // index into names; pops module, pushes (attr, module)
    ImportStar,        // pops module, stores all names in scope
}
