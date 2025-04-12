use crate::console::cpu::instruction_operands::*;
#[derive(Copy, Clone)]
pub enum Flag {
    Carry = 0x10,
    Zero = 0x80,
    Sub = 0x40,
    HalfCarry = 0x20
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L
}

impl Register {
    pub fn from_r8_operand(r8_operand: R8Operand) -> Self {
        match r8_operand {
            R8Operand::B => Register::B,
            R8Operand::C => Register::C,
            R8Operand::D => Register::D,
            R8Operand::E => Register::E,
            R8Operand::H => Register::H,
            R8Operand::L => Register::L,
            R8Operand::A => Register::A,
            R8Operand::HLInd => panic!("Attempting to construct register from HLInd")
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Register16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC
}

impl Register16 {
    pub fn from_r16_operand(r16_operand: R16Operand) -> Self {
        match r16_operand {
            R16Operand::BC => Register16::BC,
            R16Operand::DE => Register16::DE,
            R16Operand::HL => Register16::HL,
            R16Operand::SP => Register16::SP
        }
    }

    pub fn from_r16stk_operand(r16stk_operand: R16StkOperand) -> Self {
        match r16stk_operand {
            R16StkOperand::BC => Register16::BC,
            R16StkOperand::DE => Register16::DE,
            R16StkOperand::HL => Register16::HL,
            R16StkOperand::AF => Register16::AF
        }
    }
}