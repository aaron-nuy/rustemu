#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum R8Operand {
    B,
    C,
    D,
    E,
    H,
    L,
    HLInd,
    A
}

impl R8Operand {
    pub fn from_byte(byte: u8) -> R8Operand {
        match byte {
            0 => R8Operand::B,
            1 => R8Operand::C,
            2 => R8Operand::D,
            3 => R8Operand::E,
            4 => R8Operand::H,
            5 => R8Operand::L,
            6 => R8Operand::HLInd,
            7 => R8Operand::A,
            _ => panic!("Unknown R8Operand {}", byte)
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            R8Operand::B => 0,
            R8Operand::C => 1,
            R8Operand::D => 2,
            R8Operand::E => 3,
            R8Operand::H => 4,
            R8Operand::L => 5,
            R8Operand::HLInd => 6,
            R8Operand::A => 7,
        }
    }

}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum R16Operand {
    BC,
    DE,
    HL,
    SP
}

impl R16Operand {
    pub fn from_byte(byte: u8) -> R16Operand {
        match byte {
            0 => R16Operand::BC,
            1 => R16Operand::DE,
            2 => R16Operand::HL,
            3 => R16Operand::SP,
            _ => panic!("Unknown R16Operand {}", byte)
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            R16Operand::BC => 0,
            R16Operand::DE => 1,
            R16Operand::HL => 2,
            R16Operand::SP => 3,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum R16StkOperand {
    BC,
    DE,
    HL,
    AF
}

impl R16StkOperand {
    pub fn from_byte(byte: u8) -> R16StkOperand {
        match byte {
            0 => R16StkOperand::BC,
            1 => R16StkOperand::DE,
            2 => R16StkOperand::HL,
            3 => R16StkOperand::AF,
            _ => panic!("Unknown R16StkOperand {}", byte)
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            R16StkOperand::BC => 0,
            R16StkOperand::DE => 1,
            R16StkOperand::HL => 2,
            R16StkOperand::AF => 3,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum R16MemOperand {
    BC,
    DE,
    HLI,
    HLD
}

impl R16MemOperand {
    pub fn from_byte(byte: u8) -> R16MemOperand {
        match byte {
            0 => R16MemOperand::BC,
            1 => R16MemOperand::DE,
            2 => R16MemOperand::HLI,
            3 => R16MemOperand::HLD,
            _ => panic!("Unknown R16MemOperand {}", byte)
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            R16MemOperand::BC => 0,
            R16MemOperand::DE => 1,
            R16MemOperand::HLI => 2,
            R16MemOperand::HLD => 3,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FlowCondition {
    NotZero,
    Zero,
    NotCarry,
    Carry
}

impl FlowCondition {
    pub fn from_byte(byte: u8) -> FlowCondition {
        match byte {
            0 => FlowCondition::NotZero,
            1 => FlowCondition::Zero,
            2 => FlowCondition::NotCarry,
            3 => FlowCondition::Carry,
            _ => panic!("Unknown FlowCondition byte"),
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            FlowCondition::NotZero => 0,
            FlowCondition::Zero => 1,
            FlowCondition::NotCarry => 2,
            FlowCondition::Carry => 3
        }
    }
}
