pub const NOOP: u8 = 0;
pub const END: u8 = 1;
pub const MOV: u8 = 2;
pub const JMP: u8 = 3;
pub const JZ: u8 = 4;
pub const JNZ: u8 = 5;
pub const JN: u8 = 6;
pub const JNN: u8 = 7;
pub const CMP: u8 = 8;
pub const JE: u8 = 9;
pub const JNE: u8 = 10;
pub const JG: u8 = 11;
pub const JGE: u8 = 12;
pub const JL: u8 = 13;
pub const JLE: u8 = 14;
pub const CALL: u8 = 15;
pub const RET: u8 = 16;
pub const INC: u8 = 17;
pub const DEC: u8 = 18;
pub const ADD: u8 = 19;
pub const SUB: u8 = 20;
pub const MUL: u8 = 21;
pub const DIV: u8 = 22;
pub const MOD: u8 = 23;
pub const AND: u8 = 24;
pub const OR: u8 = 25;
pub const NOT: u8 = 26;
pub const NEG: u8 = 27;
pub const XOR: u8 = 28;
pub const SHL: u8 = 29;
pub const SHR: u8 = 30;
pub const SAR: u8 = 31;
pub const PUSH: u8 = 32;
pub const POP: u8 = 33;
pub const PRINT: u8 = 34;
pub const SH: u8 = 35;
pub const PUSH_RET: u8 = 36;
pub const POP_RET: u8 = 37;
pub const CPY: u8 = 38;

pub const BUILTIN_FUNCTIONS: [&str; 6] = [
    "GIT_ADD_ALL",
    "GIT_ADD",
    "GIT_COMMIT_DEFAULT",
    "GIT_COMMIT",
    "GIT_PUSH_UPSTREAM",
    "GIT_PUSH"
];

pub const BUILTIN: char = '@';
pub const LITERAL: char = '#';
pub const ARGUMENT: char = '%';
pub const VARIABLE: char = '$';
pub const REFERENCE: char = '&';
pub const DEREF: char = '*';
pub const INTEGER: char = 'i';
pub const FLOAT: char = 'f';
pub const CHAR: char = 'c';
pub const BOOLEAN_TRUE: char = 1 as char;
pub const BOOLEAN_FALSE: char = 0 as char;

pub const BUILTIN_UNKNOWN: u32 = 0;

pub const BUILTIN_GIT_ADD_ALL: u32 = 128;
pub const BUILTIN_GIT_ADD: u32 = 129;
pub const BUILTIN_GIT_COMMIT_DEFAULT: u32 = 130;
pub const BUILTIN_GIT_COMMIT: u32 = 131;
pub const BUILTIN_GIT_PUSH_UPSTREAM: u32 = 132;
pub const BUILTIN_GIT_PUSH: u32 = 133;