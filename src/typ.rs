use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Type {
    #[default]
    None,
    Never,
    UInt(IntType),
    Int(IntType),
    Float(FloatType),
    Array {
        typ: Box<Self>,
        size: Option<usize>,
    },
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvalidType;
impl FromStr for Type {
    type Err = InvalidType;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "!" => Ok(Self::Never),
            "usz" => Ok(Self::UInt(IntType::Size)),
            "u8" => Ok(Self::UInt(IntType::S8)),
            "u16" => Ok(Self::UInt(IntType::S16)),
            "u32" => Ok(Self::UInt(IntType::S32)),
            "u64" => Ok(Self::UInt(IntType::S64)),
            "isz" => Ok(Self::Int(IntType::Size)),
            "i8" => Ok(Self::Int(IntType::S8)),
            "i16" => Ok(Self::Int(IntType::S16)),
            "i32" => Ok(Self::Int(IntType::S32)),
            "i64" => Ok(Self::Int(IntType::S64)),
            "f32" => Ok(Self::Float(FloatType::S32)),
            "f64" => Ok(Self::Float(FloatType::S64)),
            _ => Err(InvalidType),
        }
    }
}
impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::None => write!(f, "none"),
            Type::Never => write!(f, "!"),
            Type::UInt(size) => write!(f, "u{size}"),
            Type::Int(size) => write!(f, "i{size}"),
            Type::Float(size) => write!(f, "f{size}"),
            Type::Array { typ, size } => write!(
                f,
                "{typ}[{}]",
                if let Some(size) = size {
                    size.to_string()
                } else {
                    "".to_string()
                }
            ),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum IntType {
    #[default]
    Size,
    S8 = 8,
    S16 = 16,
    S32 = 32,
    S64 = 64,
}
impl Display for IntType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntType::Size => write!(f, "sz"),
            IntType::S8 => write!(f, "8"),
            IntType::S16 => write!(f, "16"),
            IntType::S32 => write!(f, "32"),
            IntType::S64 => write!(f, "64"),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum FloatType {
    #[default]
    S32 = 32,
    S64 = 64,
}
impl Display for FloatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FloatType::S32 => write!(f, "32"),
            FloatType::S64 => write!(f, "64"),
        }
    }
}
