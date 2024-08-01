use std::{collections::HashMap, fmt::Display};

use crate::{
    code::{
        Destination, Function, Instruction, Program, Register, RegisterName, RegisterSize, Source,
    },
    parser::{Located, SExpr},
    typ::{IntType, Type},
};

#[derive(Debug, Default)]
pub struct Compiler {
    pub program: Program,
    pub frames: Vec<Frame>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub function: Function,
    pub scopes: Vec<Scope>,
    pub registers: usize,
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Scope {
    pub locals: HashMap<String, u8>,
    pub offset: u8,
}
#[derive(Debug, Clone, PartialEq)]
pub enum CompileError {
    NotFound(String),
    ExpectedArgs(usize),
    InvalidHead,
    InvalidType(Type),
    InvalidTypeExpected { expected: Type, got: Type },
    UnknownSize,
}
impl Frame {
    pub fn write(&mut self, instr: Instruction) -> usize {
        let addr = self.function.body.len();
        self.function.body.push(instr);
        addr
    }
    pub fn new_string(&mut self, string: String) -> String {
        let idx = self.function.strings.len();
        self.function.strings.push(string);
        format!("{}_c{idx}", self.function.name)
    }
}
impl Compiler {
    pub fn frame(&self) -> &Frame {
        self.frames.last().expect("no frame on stack")
    }
    pub fn frame_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().expect("no frame on stack")
    }
    pub fn push_frame(&mut self, name: String) {
        self.frames.push(Frame {
            function: Function {
                name,
                registers: 0,
                return_type: Type::default(),
                body: vec![],
                strings: vec![],
            },
            scopes: vec![Scope::default()],
            registers: 0,
        });
        self.write(Instruction::Push {
            src: Source::Register(Register {
                name: RegisterName::BP,
                size: RegisterSize::S32,
            }),
        });
        self.write(Instruction::Mov {
            dest: Destination::Register(Register {
                name: RegisterName::BP,
                size: RegisterSize::S32,
            }),
            src: Source::Register(Register {
                name: RegisterName::SP,
                size: RegisterSize::S32,
            }),
        });
    }
    pub fn pop_frame(&mut self) {
        let registers = self.frame().registers;
        if registers > 0 {
            self.write(Instruction::Add {
                dest: Destination::Register(Register {
                    name: RegisterName::SP,
                    size: RegisterSize::S32,
                }),
                src: Source::Amount(registers),
            });
        }
        self.write(Instruction::Leave);
        self.write(Instruction::Ret);
        let Frame {
            function,
            scopes: _,
            registers: _,
        } = self.frames.pop().expect("no frame on stack");
        self.program.functions.push(function);
    }
    pub fn write(&mut self, instr: Instruction) -> usize {
        self.frame_mut().write(instr)
    }
    pub fn new_string(&mut self, string: String) -> String {
        self.frame_mut().new_string(string)
    }
    pub fn new_extern(&mut self, name: String) {
        self.program.externs.push(name)
    }
    pub fn compile_program(
        &mut self,
        program: Vec<Located<SExpr>>,
    ) -> Result<Type, Located<CompileError>> {
        self.push_frame("main".to_string());
        for sexpr in program {
            self.compile(sexpr)?;
        }
        self.pop_frame();
        Ok(Type::default())
    }
    pub fn compile(
        &mut self,
        Located { value: sexpr, pos }: Located<SExpr>,
    ) -> Result<Type, Located<CompileError>> {
        match sexpr {
            SExpr::Expr(mut sexprs) => {
                if sexprs.is_empty() {
                    return Ok(Type::default());
                }
                let Located {
                    value: head,
                    pos: head_pos,
                } = sexprs.remove(0);
                match head {
                    SExpr::Word(word) => match word.as_str() {
                        "+" => {
                            if sexprs.len() != 2 {
                                return Err(Located {
                                    value: CompileError::ExpectedArgs(2),
                                    pos,
                                });
                            }
                            let left = sexprs.remove(0);
                            let left_pos = left.pos.clone();
                            let right = sexprs.remove(0);
                            let right_pos = right.pos.clone();

                            let left_typ = self.compile(left)?;
                            let Some(size) = RegisterSize::typ(&left_typ) else {
                                return Err(Located {
                                    value: CompileError::InvalidType(left_typ),
                                    pos: left_pos,
                                });
                            };
                            self.write(Instruction::Push {
                                src: Source::Register(Register {
                                    name: RegisterName::A,
                                    size,
                                }),
                            });

                            let right_typ = self.compile(right)?;
                            if right_typ != left_typ {
                                return Err(Located {
                                    value: CompileError::InvalidTypeExpected {
                                        expected: left_typ,
                                        got: right_typ,
                                    },
                                    pos: right_pos,
                                });
                            }
                            self.write(Instruction::Mov {
                                dest: Destination::Register(Register {
                                    name: RegisterName::B,
                                    size,
                                }),
                                src: Source::Register(Register {
                                    name: RegisterName::A,
                                    size,
                                }),
                            });
                            self.write(Instruction::Pop {
                                dest: Destination::Register(Register {
                                    name: RegisterName::A,
                                    size,
                                }),
                            });
                            self.write(Instruction::Add {
                                dest: Destination::Register(Register {
                                    name: RegisterName::A,
                                    size,
                                }),
                                src: Source::Register(Register {
                                    name: RegisterName::B,
                                    size,
                                }),
                            });

                            Ok(left_typ)
                        }
                        "extern" => {
                            for Located { value: sexpr, pos } in sexprs.into_iter().rev() {
                                match sexpr {
                                    SExpr::Word(name) | SExpr::String(name) => {
                                        self.new_extern(name);
                                    }
                                    sexpr => {
                                        return Err(Located {
                                            value: CompileError::InvalidType(
                                                self.compile(Located { value: sexpr, pos })?,
                                            ),
                                            pos,
                                        });
                                    }
                                }
                            }
                            Ok(Type::default())
                        }
                        _ => {
                            let mut args = 0;
                            for sexpr in sexprs.into_iter().rev() {
                                let pos = sexpr.pos.clone();
                                let typ = self.compile(sexpr)?;
                                match typ {
                                    Type::Array { typ, size } => {
                                        let Some(size) = size else {
                                            return Err(Located {
                                                value: CompileError::UnknownSize,
                                                pos,
                                            });
                                        };
                                        let Some(single_size) = RegisterSize::typ(&typ) else {
                                            return Err(Located {
                                                value: CompileError::InvalidType(*typ),
                                                pos,
                                            });
                                        };
                                        args += single_size.bytes() * size;
                                    }
                                    typ => {
                                        let Some(size) = RegisterSize::typ(&typ) else {
                                            return Err(Located {
                                                value: CompileError::InvalidType(typ),
                                                pos,
                                            });
                                        };
                                        args += size.bytes();
                                        self.write(Instruction::Push {
                                            src: Source::Register(Register {
                                                name: RegisterName::A,
                                                size,
                                            }),
                                        });
                                    }
                                }
                            }
                            self.write(Instruction::Call { func: word });
                            self.write(Instruction::Add {
                                dest: Destination::Register(Register {
                                    name: RegisterName::SP,
                                    size: RegisterSize::S32,
                                }),
                                src: Source::Amount(args),
                            });
                            Ok(Type::None)
                        }
                    },
                    _ => Err(Located {
                        value: CompileError::InvalidHead,
                        pos: head_pos,
                    }),
                }
            }
            SExpr::Word(_) => todo!(),
            SExpr::Int(int) => {
                self.write(Instruction::Mov {
                    dest: Destination::Register(Register {
                        name: RegisterName::A,
                        size: RegisterSize::S32,
                    }),
                    src: Source::Int(int),
                });
                Ok(Type::Int(IntType::S32))
            }
            SExpr::Float(_) => todo!(),
            SExpr::String(string) => {
                let size = string.len() + 1; // \0 at the end
                let constant = self.new_string(string);
                self.write(Instruction::Push {
                    src: Source::Name(constant),
                });
                Ok(Type::Array {
                    typ: Box::new(Type::UInt(IntType::S8)),
                    size: Some(size),
                })
            }
        }
    }
}
impl Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::NotFound(word) => write!(f, "{word:?} not found"),
            CompileError::ExpectedArgs(amount) => write!(f, "expected {amount} arguments"),
            CompileError::InvalidHead => write!(f, "invalid head"),
            CompileError::InvalidType(typ) => write!(f, "invalid type {typ}"),
            CompileError::InvalidTypeExpected { expected, got } => {
                write!(f, "expected {expected}, got {got}")
            }
            CompileError::UnknownSize => write!(f, "unknown size"),
        }
    }
}
impl Display for Located<CompileError> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}: {}",
            self.pos.ln + 1,
            self.pos.col + 1,
            self.value
        )
    }
}

pub fn compile_program(program: Vec<Located<SExpr>>) -> Result<Program, Located<CompileError>> {
    let mut compiler = Compiler::default();
    compiler.compile_program(program)?;
    Ok(compiler.program)
}
