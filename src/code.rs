use crate::typ::{FloatType, IntType, Type};
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Default)]
pub struct Program {
    pub functions: Vec<Function>,
    pub externs: Vec<String>,
}
impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for name in &self.externs {
            writeln!(f, "extern {name}")?;
        }
        writeln!(f, "global main")?;
        writeln!(f, "section .text")?;
        for function in &self.functions {
            write!(f, "{function}")?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub registers: usize,
    pub return_type: Type,
    pub body: Vec<Instruction>,
    pub strings: Vec<String>,
}
impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", self.name)?;
        for instr in &self.body {
            writeln!(f, "{instr}")?;
        }
        for (idx, string) in self.strings.iter().enumerate() {
            write!(f, "{}_c{idx} db `{string}`, 0", self.name)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum RegisterName {
    A,
    C,
    D,
    B,
    SP,
    BP,
    SI,
    DI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}
impl TryFrom<u8> for RegisterName {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::A),
            1 => Ok(Self::B),
            2 => Ok(Self::C),
            3 => Ok(Self::D),
            4 => Ok(Self::SI),
            5 => Ok(Self::DI),
            6 => Ok(Self::R8),
            7 => Ok(Self::R9),
            8 => Ok(Self::R10),
            9 => Ok(Self::R11),
            10 => Ok(Self::R12),
            11 => Ok(Self::R13),
            12 => Ok(Self::R14),
            13 => Ok(Self::R15),
            _ => Err(()),
        }
    }
}
impl Display for RegisterName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterName::A => write!(f, "a"),
            RegisterName::C => write!(f, "c"),
            RegisterName::D => write!(f, "d"),
            RegisterName::B => write!(f, "b"),
            RegisterName::SP => write!(f, "sp"),
            RegisterName::BP => write!(f, "bp"),
            RegisterName::SI => write!(f, "si"),
            RegisterName::DI => write!(f, "di"),
            RegisterName::R8 => write!(f, "r8"),
            RegisterName::R9 => write!(f, "r9"),
            RegisterName::R10 => write!(f, "r10"),
            RegisterName::R11 => write!(f, "r11"),
            RegisterName::R12 => write!(f, "r12"),
            RegisterName::R13 => write!(f, "r13"),
            RegisterName::R14 => write!(f, "r14"),
            RegisterName::R15 => write!(f, "r15"),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum RegisterSize {
    S64,
    S32,
    S16,
    S8,
}
impl RegisterSize {
    pub fn typ(typ: &Type) -> Option<Self> {
        match typ {
            Type::UInt(inttyp) => match inttyp {
                IntType::Size => Some(Self::S32),
                IntType::S8 => Some(Self::S8),
                IntType::S16 => Some(Self::S16),
                IntType::S32 => Some(Self::S32),
                IntType::S64 => Some(Self::S64),
            },
            Type::Int(inttyp) => match inttyp {
                IntType::Size => Some(Self::S32),
                IntType::S8 => Some(Self::S8),
                IntType::S16 => Some(Self::S16),
                IntType::S32 => Some(Self::S32),
                IntType::S64 => Some(Self::S64),
            },
            Type::Float(floattyp) => match floattyp {
                FloatType::S32 => Some(Self::S32),
                FloatType::S64 => Some(Self::S64),
            },
            Type::Array { typ, size: _ } => Self::typ(typ.as_ref()),
            _ => None,
        }
    }
    pub fn bytes(&self) -> usize {
        match self {
            RegisterSize::S64 => 8,
            RegisterSize::S32 => 4,
            RegisterSize::S16 => 2,
            RegisterSize::S8 => 1,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Register {
    pub name: RegisterName,
    pub size: RegisterSize,
}
impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.name {
            RegisterName::A | RegisterName::C | RegisterName::D | RegisterName::B => {
                match self.size {
                    RegisterSize::S64 => write!(f, "r{}x", self.name),
                    RegisterSize::S32 => write!(f, "e{}x", self.name),
                    RegisterSize::S16 => write!(f, "{}x", self.name),
                    RegisterSize::S8 => write!(f, "{}l", self.name),
                }
            }
            RegisterName::SP | RegisterName::BP | RegisterName::SI | RegisterName::DI => {
                match self.size {
                    RegisterSize::S64 => write!(f, "r{}", self.name),
                    RegisterSize::S32 => write!(f, "e{}", self.name),
                    RegisterSize::S16 => write!(f, "{}", self.name),
                    RegisterSize::S8 => write!(f, "{}l", self.name),
                }
            }
            RegisterName::R8
            | RegisterName::R9
            | RegisterName::R10
            | RegisterName::R11
            | RegisterName::R12
            | RegisterName::R13
            | RegisterName::R14
            | RegisterName::R15 => match self.size {
                RegisterSize::S64 => write!(f, "r{}", self.name),
                RegisterSize::S32 => write!(f, "r{}d", self.name),
                RegisterSize::S16 => write!(f, "r{}w", self.name),
                RegisterSize::S8 => write!(f, "r{}b", self.name),
            },
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidRegister;
impl FromStr for Register {
    type Err = InvalidRegister;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rax" => Ok(Self {
                name: RegisterName::A,
                size: RegisterSize::S64,
            }),
            "eax" => Ok(Self {
                name: RegisterName::A,
                size: RegisterSize::S32,
            }),
            "ax" => Ok(Self {
                name: RegisterName::A,
                size: RegisterSize::S16,
            }),
            "al" => Ok(Self {
                name: RegisterName::A,
                size: RegisterSize::S8,
            }),
            "rbx" => Ok(Self {
                name: RegisterName::B,
                size: RegisterSize::S64,
            }),
            "ebx" => Ok(Self {
                name: RegisterName::B,
                size: RegisterSize::S32,
            }),
            "bx" => Ok(Self {
                name: RegisterName::B,
                size: RegisterSize::S16,
            }),
            "bl" => Ok(Self {
                name: RegisterName::B,
                size: RegisterSize::S8,
            }),
            "rcx" => Ok(Self {
                name: RegisterName::C,
                size: RegisterSize::S64,
            }),
            "ecx" => Ok(Self {
                name: RegisterName::C,
                size: RegisterSize::S32,
            }),
            "cx" => Ok(Self {
                name: RegisterName::C,
                size: RegisterSize::S16,
            }),
            "cl" => Ok(Self {
                name: RegisterName::C,
                size: RegisterSize::S8,
            }),
            "rdx" => Ok(Self {
                name: RegisterName::D,
                size: RegisterSize::S64,
            }),
            "edx" => Ok(Self {
                name: RegisterName::D,
                size: RegisterSize::S32,
            }),
            "dx" => Ok(Self {
                name: RegisterName::D,
                size: RegisterSize::S16,
            }),
            "dl" => Ok(Self {
                name: RegisterName::D,
                size: RegisterSize::S8,
            }),
            "rsp" => Ok(Self {
                name: RegisterName::SP,
                size: RegisterSize::S64,
            }),
            "esp" => Ok(Self {
                name: RegisterName::SP,
                size: RegisterSize::S32,
            }),
            "sp" => Ok(Self {
                name: RegisterName::SP,
                size: RegisterSize::S16,
            }),
            "spl" => Ok(Self {
                name: RegisterName::SP,
                size: RegisterSize::S8,
            }),
            "rbp" => Ok(Self {
                name: RegisterName::BP,
                size: RegisterSize::S64,
            }),
            "ebp" => Ok(Self {
                name: RegisterName::BP,
                size: RegisterSize::S32,
            }),
            "bp" => Ok(Self {
                name: RegisterName::BP,
                size: RegisterSize::S16,
            }),
            "bpl" => Ok(Self {
                name: RegisterName::BP,
                size: RegisterSize::S8,
            }),
            "rsi" => Ok(Self {
                name: RegisterName::SI,
                size: RegisterSize::S64,
            }),
            "esi" => Ok(Self {
                name: RegisterName::SI,
                size: RegisterSize::S32,
            }),
            "si" => Ok(Self {
                name: RegisterName::SI,
                size: RegisterSize::S16,
            }),
            "sil" => Ok(Self {
                name: RegisterName::SI,
                size: RegisterSize::S8,
            }),
            "rdi" => Ok(Self {
                name: RegisterName::DI,
                size: RegisterSize::S64,
            }),
            "edi" => Ok(Self {
                name: RegisterName::DI,
                size: RegisterSize::S32,
            }),
            "di" => Ok(Self {
                name: RegisterName::DI,
                size: RegisterSize::S16,
            }),
            "dil" => Ok(Self {
                name: RegisterName::DI,
                size: RegisterSize::S8,
            }),
            "r9" => Ok(Self {
                name: RegisterName::R9,
                size: RegisterSize::S64,
            }),
            "r9d" => Ok(Self {
                name: RegisterName::R9,
                size: RegisterSize::S32,
            }),
            "r9w" => Ok(Self {
                name: RegisterName::R9,
                size: RegisterSize::S16,
            }),
            "r9b" => Ok(Self {
                name: RegisterName::R9,
                size: RegisterSize::S8,
            }),
            "r10" => Ok(Self {
                name: RegisterName::R10,
                size: RegisterSize::S64,
            }),
            "r10d" => Ok(Self {
                name: RegisterName::R10,
                size: RegisterSize::S32,
            }),
            "r10w" => Ok(Self {
                name: RegisterName::R10,
                size: RegisterSize::S16,
            }),
            "r10b" => Ok(Self {
                name: RegisterName::R10,
                size: RegisterSize::S8,
            }),
            "r11" => Ok(Self {
                name: RegisterName::R11,
                size: RegisterSize::S64,
            }),
            "r11d" => Ok(Self {
                name: RegisterName::R11,
                size: RegisterSize::S32,
            }),
            "r11w" => Ok(Self {
                name: RegisterName::R11,
                size: RegisterSize::S16,
            }),
            "r11b" => Ok(Self {
                name: RegisterName::R11,
                size: RegisterSize::S8,
            }),
            "r12" => Ok(Self {
                name: RegisterName::R12,
                size: RegisterSize::S64,
            }),
            "r12d" => Ok(Self {
                name: RegisterName::R12,
                size: RegisterSize::S32,
            }),
            "r12w" => Ok(Self {
                name: RegisterName::R12,
                size: RegisterSize::S16,
            }),
            "r12b" => Ok(Self {
                name: RegisterName::R12,
                size: RegisterSize::S8,
            }),
            "r13" => Ok(Self {
                name: RegisterName::R13,
                size: RegisterSize::S64,
            }),
            "r13d" => Ok(Self {
                name: RegisterName::R13,
                size: RegisterSize::S32,
            }),
            "r13w" => Ok(Self {
                name: RegisterName::R13,
                size: RegisterSize::S16,
            }),
            "r13b" => Ok(Self {
                name: RegisterName::R13,
                size: RegisterSize::S8,
            }),
            "r14" => Ok(Self {
                name: RegisterName::R14,
                size: RegisterSize::S64,
            }),
            "r14d" => Ok(Self {
                name: RegisterName::R14,
                size: RegisterSize::S32,
            }),
            "r14w" => Ok(Self {
                name: RegisterName::R14,
                size: RegisterSize::S16,
            }),
            "r14b" => Ok(Self {
                name: RegisterName::R14,
                size: RegisterSize::S8,
            }),
            "r15" => Ok(Self {
                name: RegisterName::R15,
                size: RegisterSize::S64,
            }),
            "r15d" => Ok(Self {
                name: RegisterName::R15,
                size: RegisterSize::S32,
            }),
            "r15w" => Ok(Self {
                name: RegisterName::R15,
                size: RegisterSize::S16,
            }),
            "r15b" => Ok(Self {
                name: RegisterName::R15,
                size: RegisterSize::S8,
            }),
            _ => Err(InvalidRegister),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum Destination {
    Register(Register),
    Memory {
        data_type: DataType,
        at: usize,
    },
    MemoryRegister {
        data_type: DataType,
        register: Register,
    },
    MemoryOffset {
        data_type: DataType,
        register: Register,
        offset: usize,
        scale: usize,
    },
}
impl Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Destination::Register(register) => write!(f, "{register}"),
            Destination::Memory { data_type, at } => write!(f, "{data_type} PTR [{at}]"),
            Destination::MemoryRegister {
                data_type,
                register,
            } => write!(f, "{data_type} PTR [{register}]"),
            Destination::MemoryOffset {
                data_type,
                register,
                offset,
                scale,
            } => write!(f, "{data_type} PTR [{register}+{offset}*{scale}]"),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    Register(Register),
    Memory {
        data_type: DataType,
        at: usize,
    },
    MemoryRegister {
        data_type: DataType,
        register: Register,
    },
    MemoryOffset {
        data_type: DataType,
        register: Register,
        offset: usize,
        scale: usize,
    },
    Int(i32),
    Name(String),
    Amount(usize),
}
impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Register(register) => write!(f, "{register}"),
            Source::Memory { data_type, at } => write!(f, "{data_type} PTR [{at}]"),
            Source::MemoryRegister {
                data_type,
                register,
            } => write!(f, "{data_type} PTR [{register}]"),
            Source::MemoryOffset {
                data_type,
                register,
                offset,
                scale,
            } => write!(f, "{data_type} PTR [{register}+{offset}*{scale}]"),
            Source::Name(name) => write!(f, "{name}"),
            Source::Int(int) => write!(f, "${int}"),
            Source::Amount(amount) => write!(f, "{amount}"),
        }
    }
}
impl From<Destination> for Source {
    fn from(value: Destination) -> Self {
        match value {
            Destination::Register(register) => Self::Register(register),
            Destination::Memory { data_type, at } => Self::Memory { data_type, at },
            Destination::MemoryRegister {
                data_type,
                register,
            } => Self::MemoryRegister {
                data_type,
                register,
            },
            Destination::MemoryOffset {
                data_type,
                register,
                offset,
                scale,
            } => Self::MemoryOffset {
                data_type,
                register,
                offset,
                scale,
            },
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataType {
    Byte,
    Word,
    DoubleWord,
    QuadWord,
}
impl Into<RegisterSize> for DataType {
    fn into(self) -> RegisterSize {
        match self {
            DataType::Byte => RegisterSize::S8,
            DataType::Word => RegisterSize::S16,
            DataType::DoubleWord => RegisterSize::S32,
            DataType::QuadWord => RegisterSize::S64,
        }
    }
}
impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Byte => write!(f, "BYTE"),
            DataType::Word => write!(f, "WORD"),
            DataType::DoubleWord => write!(f, "DWORD"),
            DataType::QuadWord => write!(f, "QWORD"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default)]
#[repr(u8)]
pub enum Instruction {
    #[default]
    NOp,

    Mov {
        dest: Destination,
        src: Source,
    },

    Push {
        src: Source,
    },
    Pop {
        dest: Destination,
    },
    Call {
        func: String,
    },
    Leave,
    Ret,

    Label(String),
    Jmp {
        label: String,
    },
    JOp {
        op: ComparisonOperator,
        label: String,
    },
    Cmp {
        a: Source,
        b: Source,
    },

    Add {
        dest: Destination,
        src: Source,
    },
    Mul {
        src: Source,
    },
    Div {
        src: Source,
    },
}
impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::NOp => write!(f, "\tnop"),
            Instruction::Mov { dest, src } => write!(f, "\tmov {dest}, {src}"),
            Instruction::Push { src } => write!(f, "\tpush {src}"),
            Instruction::Pop { dest } => write!(f, "\tpop {dest}"),
            Instruction::Call { func } => write!(f, "\tcall {func}"),
            Instruction::Leave => write!(f, "\tleave"),
            Instruction::Ret => write!(f, "\tret"),
            Instruction::Label(label) => write!(f, ".{label}:"),
            Instruction::Jmp { label } => write!(f, "\tjmp {label}"),
            Instruction::JOp { op, label } => write!(f, "\tj{op} {label}"),
            Instruction::Cmp { a, b } => write!(f, "\tcmp {a}, {b}"),
            Instruction::Add { dest, src } => write!(f, "\tadd {dest}, {src}"),
            Instruction::Mul { src } => write!(f, "\tmul {src}"),
            Instruction::Div { src } => write!(f, "\tdiv {src}"),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    LessUnsigned,
    GreaterUnsigned,
    LessEqualUnsigned,
    GreaterEqualUnsigned,
}
impl Display for ComparisonOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComparisonOperator::Equal => write!(f, "e"),
            ComparisonOperator::NotEqual => write!(f, "ne"),
            ComparisonOperator::Less => write!(f, "l"),
            ComparisonOperator::Greater => write!(f, "g"),
            ComparisonOperator::LessEqual => write!(f, "le"),
            ComparisonOperator::GreaterEqual => write!(f, "ge"),
            ComparisonOperator::LessUnsigned => write!(f, "b"),
            ComparisonOperator::GreaterUnsigned => write!(f, "a"),
            ComparisonOperator::LessEqualUnsigned => write!(f, "be"),
            ComparisonOperator::GreaterEqualUnsigned => write!(f, "ae"),
        }
    }
}
