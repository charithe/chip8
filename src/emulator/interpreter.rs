use super::common::{Error, Result};
use std::fmt;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Address(pub u16);

impl From<Address> for usize {
    fn from(addr: Address) -> Self {
        addr.0 as usize
    }
}

impl From<Address> for u16 {
    fn from(addr: Address) -> Self {
        addr.0
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Register(pub u8);

impl std::ops::Index<Register> for [u8] {
    type Output = u8;

    fn index(&self, reg: Register) -> &Self::Output {
        &self[reg.0 as usize]
    }
}

impl std::ops::IndexMut<Register> for [u8] {
    fn index_mut(&mut self, reg: Register) -> &mut Self::Output {
        &mut self[reg.0 as usize]
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Value(pub u8);

impl From<Value> for u8 {
    fn from(val: Value) -> Self {
        val.0
    }
}

#[derive(Debug, PartialEq)]
pub enum Op {
    ADD(Register, Value),
    ADDI(Register),
    ADDR(Register, Register),
    AND(Register, Register),
    CALL(Address),
    CLS,
    CPDT(Register),
    DRW(Register, Register, Value),
    JP(Address),
    JPREL(Address),
    LD(Register, Value),
    LDDT(Register),
    LDI(Address),
    LDIB(Register),
    LDIM(Register),
    LDIR(Register),
    LDIS(Register),
    LDKP(Register),
    LDR(Register, Register),
    LDST(Register),
    OR(Register, Register),
    RET,
    RND(Register, Value),
    SE(Register, Value),
    SER(Register, Register),
    SHL(Register),
    SHR(Register),
    SKNP(Register),
    SKP(Register),
    SNE(Register, Value),
    SNER(Register, Register),
    SUB(Register, Register),
    SUBN(Register, Register),
    SYS(Address),
    XOR(Register, Register),
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Op::SYS(Address(addr)) => f.write_fmt(format_args!("SYS #{}", addr)),
            Op::CLS => f.write_str("CLS"),
            Op::RET => f.write_str("RET"),
            Op::JP(Address(addr)) => f.write_fmt(format_args!("JP #{}", addr)),
            Op::CALL(Address(addr)) => f.write_fmt(format_args!("CALL #{}", addr)),
            Op::SE(Register(reg), Value(val)) => f.write_fmt(format_args!("SE $V{} {}", reg, val)),
            Op::SNE(Register(reg), Value(val)) => {
                f.write_fmt(format_args!("SNE $V{} {}", reg, val))
            }
            Op::SER(Register(reg1), Register(reg2)) => {
                f.write_fmt(format_args!("SE $V{} $V{}", reg1, reg2))
            }
            Op::LD(Register(reg), Value(val)) => f.write_fmt(format_args!("LD $V{} {}", reg, val)),
            Op::ADD(Register(reg), Value(val)) => {
                f.write_fmt(format_args!("ADD $V{} {}", reg, val))
            }
            Op::LDR(Register(reg1), Register(reg2)) => {
                f.write_fmt(format_args!("LD $V{} $V{}", reg1, reg2))
            }
            Op::OR(Register(reg1), Register(reg2)) => {
                f.write_fmt(format_args!("OR $V{} $V{}", reg1, reg2))
            }
            Op::AND(Register(reg1), Register(reg2)) => {
                f.write_fmt(format_args!("AND $V{} $V{}", reg1, reg2))
            }
            Op::XOR(Register(reg1), Register(reg2)) => {
                f.write_fmt(format_args!("XOR $V{} $V{}", reg1, reg2))
            }
            Op::ADDR(Register(reg1), Register(reg2)) => {
                f.write_fmt(format_args!("ADD $V{} $V{}", reg1, reg2))
            }
            Op::SUB(Register(reg1), Register(reg2)) => {
                f.write_fmt(format_args!("SUB $V{} $V{}", reg1, reg2))
            }
            Op::SHR(Register(reg)) => f.write_fmt(format_args!("SHR $V{}", reg)),
            Op::SUBN(Register(reg1), Register(reg2)) => {
                f.write_fmt(format_args!("SUBN $V{} $V{}", reg1, reg2))
            }
            Op::SHL(Register(reg)) => f.write_fmt(format_args!("SHL $V{}", reg)),
            Op::SNER(Register(reg1), Register(reg2)) => {
                f.write_fmt(format_args!("SNE $V{} $V{}", reg1, reg2))
            }
            Op::LDI(Address(addr)) => f.write_fmt(format_args!("LDI #{}", addr)),
            Op::JPREL(Address(addr)) => f.write_fmt(format_args!("JPREL #{}", addr)),
            Op::RND(Register(reg), Value(val)) => {
                f.write_fmt(format_args!("RND $V{} {}", reg, val))
            }
            Op::DRW(Register(reg1), Register(reg2), Value(val)) => {
                f.write_fmt(format_args!("DRW $V{} $V{} {}", reg1, reg2, val))
            }
            Op::SKP(Register(reg)) => f.write_fmt(format_args!("SKP $V{}", reg)),
            Op::SKNP(Register(reg)) => f.write_fmt(format_args!("SKNP $V{}", reg)),
            Op::CPDT(Register(reg)) => f.write_fmt(format_args!("CPDT $V{}", reg)),
            Op::LDKP(Register(reg)) => f.write_fmt(format_args!("LDKP $V{}", reg)),
            Op::LDDT(Register(reg)) => f.write_fmt(format_args!("LDDT $V{}", reg)),
            Op::LDST(Register(reg)) => f.write_fmt(format_args!("LDST $V{}", reg)),
            Op::ADDI(Register(reg)) => f.write_fmt(format_args!("ADDI $V{}", reg)),
            Op::LDIS(Register(reg)) => f.write_fmt(format_args!("LDIS $V{}", reg)),
            Op::LDIB(Register(reg)) => f.write_fmt(format_args!("LDIB $V{}", reg)),
            Op::LDIR(Register(reg)) => f.write_fmt(format_args!("LDIR $V{}", reg)),
            Op::LDIM(Register(reg)) => f.write_fmt(format_args!("LDIM $V{}", reg)),
        }
    }
}

pub struct Instruction(pub u16);

impl Instruction {
    pub fn interpret(&self) -> Result<Op> {
        match self.0 & 0xF000 {
            0x0000 => match self.0 {
                0x00E0 => Ok(Op::CLS),
                0x00EE => Ok(Op::RET),
                _ => Ok(Op::SYS(self.addr())),
            },
            0x1000 => Ok(Op::JP(self.addr())),
            0x2000 => Ok(Op::CALL(self.addr())),
            0x3000 => Ok(Op::SE(self.second_nibble(), self.last_byte())),
            0x4000 => Ok(Op::SNE(self.second_nibble(), self.last_byte())),
            0x5000 => Ok(Op::SER(self.second_nibble(), self.third_nibble())),
            0x6000 => Ok(Op::LD(self.second_nibble(), self.last_byte())),
            0x7000 => Ok(Op::ADD(self.second_nibble(), self.last_byte())),
            0x8000 => match self.0 & 0x000F {
                0x0 => Ok(Op::LDR(self.second_nibble(), self.third_nibble())),
                0x1 => Ok(Op::OR(self.second_nibble(), self.third_nibble())),
                0x2 => Ok(Op::AND(self.second_nibble(), self.third_nibble())),
                0x3 => Ok(Op::XOR(self.second_nibble(), self.third_nibble())),
                0x4 => Ok(Op::ADDR(self.second_nibble(), self.third_nibble())),
                0x5 => Ok(Op::SUB(self.second_nibble(), self.third_nibble())),
                0x6 => Ok(Op::SHR(self.second_nibble())),
                0x7 => Ok(Op::SUBN(self.second_nibble(), self.third_nibble())),
                0xE => Ok(Op::SHL(self.second_nibble())),
                _ => Err(Error::UnknownInstruction(self.0)),
            },
            0x9000 => match self.0 & 0x000F {
                0x0 => Ok(Op::SNER(self.second_nibble(), self.third_nibble())),
                _ => Err(Error::UnknownInstruction(self.0)),
            },
            0xA000 => Ok(Op::LDI(self.addr())),
            0xB000 => Ok(Op::JPREL(self.addr())),
            0xC000 => Ok(Op::RND(self.second_nibble(), self.last_byte())),
            0xD000 => {
                let n = (self.0 & 0x000F) as u8;
                Ok(Op::DRW(self.second_nibble(), self.third_nibble(), Value(n)))
            }
            0xE000 => match self.0 & 0x00FF {
                0x9E => Ok(Op::SKP(self.second_nibble())),
                0xA1 => Ok(Op::SKNP(self.second_nibble())),
                _ => Err(Error::UnknownInstruction(self.0)),
            },
            0xF000 => match self.0 & 0x00FF {
                0x07 => Ok(Op::CPDT(self.second_nibble())),
                0x0A => Ok(Op::LDKP(self.second_nibble())),
                0x15 => Ok(Op::LDDT(self.second_nibble())),
                0x18 => Ok(Op::LDST(self.second_nibble())),
                0x1E => Ok(Op::ADDI(self.second_nibble())),
                0x29 => Ok(Op::LDIS(self.second_nibble())),
                0x33 => Ok(Op::LDIB(self.second_nibble())),
                0x55 => Ok(Op::LDIR(self.second_nibble())),
                0x65 => Ok(Op::LDIM(self.second_nibble())),
                _ => Err(Error::UnknownInstruction(self.0)),
            },
            _ => Err(Error::UnknownInstruction(self.0)),
        }
    }

    // Consider an instruction such as ABCD
    // second_nibble = B
    // third_nibble = C
    // last_byte = CD
    // addr = BCD

    fn second_nibble(&self) -> Register {
        Register(((self.0 & 0x0F00) >> 8) as u8)
    }

    fn third_nibble(&self) -> Register {
        Register(((self.0 & 0x00F0) >> 4) as u8)
    }

    fn last_byte(&self) -> Value {
        Value((self.0 & 0x00FF) as u8)
    }

    fn addr(&self) -> Address {
        Address((self.0 & 0x0FFF) as u16)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{:01$X}", self.0, 4))
    }
}

pub fn to_bcd(v: u8) -> [u8; 3] {
    let mut r = [0u8; 3];
    r[0] = v / 100;
    r[1] = (v % 100) / 10;
    r[2] = (v % 100) % 10;

    r
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_instruction_ok {
        ($name:ident, $input:literal, $want:expr) => {
            #[test]
            fn $name() {
                let got_op = Instruction($input).interpret();
                assert!(got_op.is_ok());
                assert_eq!($want, got_op.unwrap());
            }
        };
    }

    test_instruction_ok!(test_cls, 0x00E0, Op::CLS);
    test_instruction_ok!(test_ret, 0x00EE, Op::RET);
    test_instruction_ok!(test_sys, 0x0123, Op::SYS(Address(0x123)));
    test_instruction_ok!(test_jp, 0x1123, Op::JP(Address(0x123)));
    test_instruction_ok!(test_call, 0x2123, Op::CALL(Address(0x123)));
    test_instruction_ok!(test_sev, 0x3456, Op::SE(Register(0x4), Value(0x56)));
    test_instruction_ok!(test_snev, 0x4456, Op::SNE(Register(0x4), Value(0x56)));
    test_instruction_ok!(test_ser, 0x5860, Op::SER(Register(0x8), Register(0x6)));
    test_instruction_ok!(test_ld, 0x6876, Op::LD(Register(0x8), Value(0x76)));
    test_instruction_ok!(test_add, 0x7876, Op::ADD(Register(0x8), Value(0x76)));
    test_instruction_ok!(test_ldr, 0x8870, Op::LDR(Register(0x8), Register(0x7)));
    test_instruction_ok!(test_or, 0x8871, Op::OR(Register(0x8), Register(0x7)));
    test_instruction_ok!(test_and, 0x8872, Op::AND(Register(0x8), Register(0x7)));
    test_instruction_ok!(test_xor, 0x8873, Op::XOR(Register(0x8), Register(0x7)));
    test_instruction_ok!(test_addr, 0x8874, Op::ADDR(Register(0x8), Register(0x7)));
    test_instruction_ok!(test_sub, 0x8875, Op::SUB(Register(0x8), Register(0x7)));
    test_instruction_ok!(test_shr, 0x8876, Op::SHR(Register(0x8)));
    test_instruction_ok!(test_subn, 0x8877, Op::SUBN(Register(0x8), Register(0x7)));
    test_instruction_ok!(test_shl, 0x887E, Op::SHL(Register(0x8)));
    test_instruction_ok!(test_sner, 0x9870, Op::SNER(Register(0x8), Register(0x7)));
    test_instruction_ok!(test_ldi, 0xA870, Op::LDI(Address(0x870)));
    test_instruction_ok!(test_jprel, 0xB870, Op::JPREL(Address(0x870)));
    test_instruction_ok!(test_rnd, 0xC870, Op::RND(Register(0x8), Value(0x70)));
    test_instruction_ok!(
        test_drw,
        0xD875,
        Op::DRW(Register(0x8), Register(0x7), Value(0x5))
    );
    test_instruction_ok!(test_skp, 0xE89E, Op::SKP(Register(0x8)));
    test_instruction_ok!(test_sknp, 0xE8A1, Op::SKNP(Register(0x8)));
    test_instruction_ok!(test_cpdt, 0xF807, Op::CPDT(Register(0x8)));
    test_instruction_ok!(test_ldkp, 0xF80A, Op::LDKP(Register(0x8)));
    test_instruction_ok!(test_lddt, 0xF815, Op::LDDT(Register(0x8)));
    test_instruction_ok!(test_ldst, 0xF818, Op::LDST(Register(0x8)));
    test_instruction_ok!(test_addi, 0xF81E, Op::ADDI(Register(0x8)));
    test_instruction_ok!(test_ldis, 0xF829, Op::LDIS(Register(0x8)));
    test_instruction_ok!(test_ldib, 0xF833, Op::LDIB(Register(0x8)));
    test_instruction_ok!(test_ldir, 0xF855, Op::LDIR(Register(0x8)));
    test_instruction_ok!(test_ldim, 0xF865, Op::LDIM(Register(0x8)));

    #[test]
    fn test_to_bcd() {
        assert_eq!([1, 2, 3], to_bcd(123));
        assert_eq!([0, 2, 3], to_bcd(23));
        assert_eq!([0, 0, 3], to_bcd(3));
        assert_eq!([0, 0, 0], to_bcd(0));
    }
}
