pub use crate::console::cpu::instruction_operands::*;
pub use crate::console::cpu::register::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    LD(R8Operand, R8Operand),
    LDImm8(R8Operand, u8),
    LDFromMemToA(R16MemOperand),
    LDToMemFromA(R16MemOperand),
    LDFromImmIndToA16(u16),
    LDToImmIndFromA16(u16),
    LDToAFromCInd(),
    LDFromAToCInd(),
    LDFromImmIndToA8(u8),
    LDToImmIndFromA8(u8),
    LDImm16(R16Operand, u16),
    LDToImmIndFromSP(u16),
    LDSPFromHL(),
    PUSH(R16StkOperand),
    POP(R16StkOperand),
    LDHLFromAdjustedSP(i8),
    ADD(R8Operand),
    ADDImm(u8),
    ADC(R8Operand),
    ADCImm(u8),
    SUB(R8Operand),
    SUBImm(u8),
    SBC(R8Operand),
    SBCImm(u8),
    CP(R8Operand),
    CPImm(u8),
    INC8(R8Operand),
    DEC8(R8Operand),
    AND(R8Operand),
    ANDImm(u8),
    OR(R8Operand),
    ORImm(u8),
    XOR(R8Operand),
    XORImm(u8),
    CCF(),
    SCF(),
    DAA(),
    CPL(),
    RLCA(),
    RRCA(),
    RRA(),
    RLA(),
    INC16(R16Operand),
    DEC16(R16Operand),
    ADDHL(R16Operand),
    ADDSPImm(i8),
    RLC(R8Operand),
    RRC(R8Operand),
    RL(R8Operand),
    RR(R8Operand),
    SRA(R8Operand),
    SLA(R8Operand),
    SRL(R8Operand),
    SWAP(R8Operand),
    BIT(u8, R8Operand),
    RESET(u8, R8Operand),
    SET(u8, R8Operand),
    NOP(),
    JP(u16),
    JPHL(),
    JPCC(FlowCondition, u16),
    JR(i8),
    JRCC(FlowCondition, i8),
    CALL(u16),
    CALLCC(FlowCondition, u16),
    RET(),
    RETCC(FlowCondition),
    RETI(),
    RST(u8),
    HALT(),
    STOP(),
    DI(),
    EI(),
}

impl Instruction {
    pub fn decode(first_byte: u8, second_byte: u8, third_byte: u8) -> (Instruction, u16) {
        if first_byte == 0xcb {
            Instruction::decode_cb(second_byte)
        } else {
            let imm16_operand = u16::from_le_bytes([second_byte, third_byte]);
            Instruction::decode_generic(first_byte, second_byte, imm16_operand)
        }
    }

    fn decode_cb(opcode: u8) -> (Instruction, u16) {
        let operand = opcode & 0b0000_0111;
        let bit_index = (opcode & 0b0011_1000) >> 3;
        let r8_operand = R8Operand::from_byte(operand);

        match (opcode & 0b11000000) >> 6 {
            0 => match (opcode & 0b00111000) >> 3 {
                0 => (Instruction::RLC(r8_operand), 2),
                1 => (Instruction::RRC(r8_operand), 2),
                2 => (Instruction::RL(r8_operand), 2),
                3 => (Instruction::RR(r8_operand), 2),
                4 => (Instruction::SLA(r8_operand), 2),
                5 => (Instruction::SRA(r8_operand), 2),
                6 => (Instruction::SWAP(r8_operand), 2),
                7 => (Instruction::SRL(r8_operand), 2),
                _ => panic!("Unknown instruction: 0xCB {}", opcode),
            },
            1 => (Instruction::BIT(bit_index, r8_operand), 2),
            2 => (Instruction::RESET(bit_index, r8_operand), 2),
            3 => (Instruction::SET(bit_index, r8_operand), 2),
            _ => panic!("Unknown instruction: 0xCB {}", opcode),
        }
    }

    fn decode_generic(opcode: u8, imm_8: u8, imm_16: u16) -> (Instruction, u16) {
        match (opcode & 0b11000000) >> 6 {
            0 => Instruction::decode_generic_block_0(opcode, imm_8, imm_16),
            1 => Instruction::decode_generic_block_1(opcode, imm_8, imm_16),
            2 => Instruction::decode_generic_block_2(opcode, imm_8, imm_16),
            3 => Instruction::decode_generic_block_3(opcode, imm_8, imm_16),
            _ => panic!("Uknown block"),
        }
    }

    fn decode_generic_block_0(opcode: u8, imm_8: u8, imm_16: u16) -> (Instruction, u16) {
        match opcode {
            0b0000_0000 => (Instruction::NOP(), 1),
            0b0000_0111 => (Instruction::RLCA(), 1),
            0b0000_1111 => (Instruction::RRCA(), 1),
            0b0001_0111 => (Instruction::RLA(), 1),
            0b0001_1111 => (Instruction::RRA(), 1),
            0b0010_0111 => (Instruction::DAA(), 1),
            0b0010_1111 => (Instruction::CPL(), 1),
            0b0011_0111 => (Instruction::SCF(), 1),
            0b0011_1111 => (Instruction::CCF(), 1),
            0b0001_0000 => (Instruction::STOP(), 2),
            0b0001_1000 => (Instruction::JR(imm_8 as i8), 2),
            0b0000_1000 => (Instruction::LDToImmIndFromSP(imm_16), 3),
            opcode if (opcode & 0b1110_0111) == 0b0010_0000 => {
                let operand = (opcode & 0b0001_1000) >> 3;
                let cond_operand = FlowCondition::from_byte(operand);
                (Instruction::JRCC(cond_operand, imm_8 as i8), 2)
            }
            opcode if (opcode & 0b1100_0111) == 0b0000_0110 => {
                let operand = (opcode & 0b0011_1000) >> 3;
                let r8_operand = R8Operand::from_byte(operand);
                (Instruction::LDImm8(r8_operand, imm_8), 2)
            }
            opcode if (opcode & 0b1100_0111) == 0b0000_0100 => {
                let operand = (opcode & 0b0011_1000) >> 3;
                let r8_operand = R8Operand::from_byte(operand);
                (Instruction::INC8(r8_operand), 1)
            }
            opcode if (opcode & 0b1100_0111) == 0b0000_0101 => {
                let operand = (opcode & 0b0011_1000) >> 3;
                let r8_operand = R8Operand::from_byte(operand);
                (Instruction::DEC8(r8_operand), 1)
            }
            opcode if (opcode & 0b1100_1111) == 0b0000_0011 => {
                let operand = (opcode & 0b0011_0000) >> 4;
                let r16_operand = R16Operand::from_byte(operand);
                (Instruction::INC16(r16_operand), 1)
            }
            opcode if (opcode & 0b1100_1111) == 0b0000_1011 => {
                let operand = (opcode & 0b0011_0000) >> 4;
                let r16_operand = R16Operand::from_byte(operand);
                (Instruction::DEC16(r16_operand), 1)
            }
            opcode if (opcode & 0b1100_1111) == 0b0000_1001 => {
                let operand = (opcode & 0b0011_0000) >> 4;
                let r16_operand = R16Operand::from_byte(operand);
                (Instruction::ADDHL(r16_operand), 1)
            }
            opcode if (opcode & 0b1100_1111) == 0b0000_0001 => {
                let operand = (opcode & 0b0011_0000) >> 4;
                let r16_operand = R16Operand::from_byte(operand);
                (Instruction::LDImm16(r16_operand, imm_16), 3)
            }
            opcode if (opcode & 0b1100_1111) == 0b0000_0010 => {
                let operand = (opcode & 0b0011_0000) >> 4;
                let r16mem_operand = R16MemOperand::from_byte(operand);
                (Instruction::LDToMemFromA(r16mem_operand), 1)
            }
            opcode if (opcode & 0b1100_1111) == 0b0000_1010 => {
                let operand = (opcode & 0b0011_0000) >> 4;
                let r16mem_operand = R16MemOperand::from_byte(operand);
                (Instruction::LDFromMemToA(r16mem_operand), 1)
            }
            _ => panic!(
                "Unknown instruction Block Zero Hex: {:#x} | Binary: {:#b}",
                opcode, opcode
            ),
        }
    }

    fn decode_generic_block_1(opcode: u8, _: u8, _: u16) -> (Instruction, u16) {
        let operand_dest = (opcode & 0b0011_1000) >> 3;
        let operand_source = opcode & 0b0000_0111;
        let r8_operand_dest = R8Operand::from_byte(operand_dest);
        let r8_operand_source = R8Operand::from_byte(operand_source);

        match opcode {
            0b0111_0110 => (Instruction::HALT(), 1),
            opcode if (opcode & 0b1100_0000) == 0b0100_0000 => {
                match (r8_operand_dest.clone(), r8_operand_source.clone()) {
                    (R8Operand::HLInd, R8Operand::HLInd) => (Instruction::HALT(), 1),
                    (_, _) => (Instruction::LD(r8_operand_dest, r8_operand_source), 1),
                }
            }
            _ => panic!(
                "Unknown instruction Block One Hex: {:#x} | Binary: {:#b}",
                opcode, opcode
            ),
        }
    }

    fn decode_generic_block_2(opcode: u8, _: u8, _: u16) -> (Instruction, u16) {
        let operand = opcode & 0b000_0111;
        let r8_operand = R8Operand::from_byte(operand);

        match (opcode & 0b0111_1000) >> 3 {
            0 => (Instruction::ADD(r8_operand), 1),
            1 => (Instruction::ADC(r8_operand), 1),
            2 => (Instruction::SUB(r8_operand), 1),
            3 => (Instruction::SBC(r8_operand), 1),
            4 => (Instruction::AND(r8_operand), 1),
            5 => (Instruction::XOR(r8_operand), 1),
            6 => (Instruction::OR(r8_operand), 1),
            7 => (Instruction::CP(r8_operand), 1),
            _ => panic!(
                "Unknown instruction Block Two Hex: {:#x} | Binary: {:#b}",
                opcode, opcode
            ),
        }
    }

    fn decode_generic_block_3(opcode: u8, imm_8: u8, imm_16: u16) -> (Instruction, u16) {
        match opcode {
            //Table 1
            0b1100_0110 => (Instruction::ADDImm(imm_8), 2),
            0b1100_1110 => (Instruction::ADCImm(imm_8), 2),
            0b1101_0110 => (Instruction::SUBImm(imm_8), 2),
            0b1101_1110 => (Instruction::SBCImm(imm_8), 2),
            0b1110_0110 => (Instruction::ANDImm(imm_8), 2),
            0b1110_1110 => (Instruction::XORImm(imm_8), 2),
            0b1111_0110 => (Instruction::ORImm(imm_8), 2),
            0b1111_1110 => (Instruction::CPImm(imm_8), 2),
            // Table 2
            0b1100_1001 => (Instruction::RET(), 1),
            0b1101_1001 => (Instruction::RETI(), 1),
            0b1100_0011 => (Instruction::JP(imm_16), 3),
            0b1110_1001 => (Instruction::JPHL(), 1),
            0b1100_1101 => (Instruction::CALL(imm_16), 3),
            opcode if (opcode & 0b1110_0111) == 0b1100_0000 => {
                let operand = (opcode & 0b0001_1000) >> 3;
                let cond = FlowCondition::from_byte(operand);

                (Instruction::RETCC(cond), 1)
            }
            opcode if (opcode & 0b1110_0111) == 0b1100_0010 => {
                let operand = (opcode & 0b0001_1000) >> 3;
                let cond = FlowCondition::from_byte(operand);

                (Instruction::JPCC(cond, imm_16), 3)
            }
            opcode if (opcode & 0b1110_0111) == 0b1100_0100 => {
                let operand = (opcode & 0b0001_1000) >> 3;
                let cond = FlowCondition::from_byte(operand);

                (Instruction::CALLCC(cond, imm_16), 3)
            }
            opcode if (opcode & 0b1100_0111) == 0b1100_0111 => {
                let operand = (opcode & 0b0011_1000) >> 3;

                (Instruction::RST(operand), 1)
            }
            // Table 3
            opcode if (opcode & 0b1100_1111) == 0b1100_0001 => {
                let operand = (opcode & 0b0011_0000) >> 4;
                let r16stk_operand = R16StkOperand::from_byte(operand);
                (Instruction::POP(r16stk_operand), 1)
            }
            opcode if (opcode & 0b1100_1111) == 0b1100_0101 => {
                let operand = (opcode & 0b0011_0000) >> 4;
                let r16stk_operand = R16StkOperand::from_byte(operand);
                (Instruction::PUSH(r16stk_operand), 1)
            }
            // Table 4
            0b1110_0010 => (Instruction::LDFromAToCInd(), 1),
            0b1110_0000 => (Instruction::LDToImmIndFromA8(imm_8), 2),
            0b1110_1010 => (Instruction::LDToImmIndFromA16(imm_16), 3),
            0b1111_0010 => (Instruction::LDToAFromCInd(), 1),
            0b1111_0000 => (Instruction::LDFromImmIndToA8(imm_8), 2),
            0b1111_1010 => (Instruction::LDFromImmIndToA16(imm_16), 3),
            // Table 5
            0b1110_1000 => (Instruction::ADDSPImm(imm_8 as i8), 2),
            0b1111_1000 => (Instruction::LDHLFromAdjustedSP(imm_8 as i8), 2),
            0b1111_1001 => (Instruction::LDSPFromHL(), 1),
            // Table 6
            0b1111_0011 => (Instruction::DI(), 1),
            0b1111_1011 => (Instruction::EI(), 1),
            _ => panic!(
                "Unknown instruction Block Three Hex: {:#x} | Binary: {:#b}",
                opcode, opcode
            ),
        }
    }

    pub fn encode(instruction: Instruction) -> [u8; 3] {
        match instruction {
            // CB
            Instruction::RLC(r8_operand) => {
                let operand = r8_operand.to_byte();
                [0xCB, (0b0000_0000 | operand), 0]
            }
            Instruction::RRC(r8_operand) => {
                let operand = r8_operand.to_byte();
                [0xCB, (0b0000_1000 | operand), 0]
            }
            Instruction::RL(r8_operand) => {
                let operand = r8_operand.to_byte();
                [0xCB, (0b0001_0000 | operand), 0]
            }
            Instruction::RR(r8_operand) => {
                let operand = r8_operand.to_byte();
                [0xCB, (0b0001_1000 | operand), 0]
            }
            Instruction::SLA(r8_operand) => {
                let operand = r8_operand.to_byte();
                [0xCB, (0b0010_0000 | operand), 0]
            }
            Instruction::SRA(r8_operand) => {
                let operand = r8_operand.to_byte();
                [0xCB, (0b0010_1000 | operand), 0]
            }
            Instruction::SWAP(r8_operand) => {
                let operand = r8_operand.to_byte();
                [0xCB, (0b0011_0000 | operand), 0]
            }
            Instruction::SRL(r8_operand) => {
                let operand = r8_operand.to_byte();
                [0xCB, (0b0011_1000 | operand), 0]
            }
            Instruction::BIT(bit_index, r8_operand) => {
                let operand = r8_operand.to_byte();
                let adjusted_bit_index = bit_index << 3;
                [0xCB, (0b0100_0000 | adjusted_bit_index | operand), 0]
            }
            Instruction::RESET(bit_index, r8_operand) => {
                let operand = r8_operand.to_byte();
                let adjusted_bit_index = bit_index << 3;
                [0xCB, (0b1000_0000 | adjusted_bit_index | operand), 0]
            }
            Instruction::SET(bit_index, r8_operand) => {
                let operand = r8_operand.to_byte();
                let adjusted_bit_index = bit_index << 3;
                [0xCB, (0b1100_0000 | adjusted_bit_index | operand), 0]
            }
            // Block 0
            Instruction::NOP() => [0b0000_0000, 0, 0],
            Instruction::RLCA() => [0b0000_0111, 0, 0],
            Instruction::RRCA() => [0b0000_1111, 0, 0],
            Instruction::RLA() => [0b0001_0111, 0, 0],
            Instruction::RRA() => [0b0001_1111, 0, 0],
            Instruction::DAA() => [0b0010_0111, 0, 0],
            Instruction::CPL() => [0b0010_1111, 0, 0],
            Instruction::SCF() => [0b0011_0111, 0, 0],
            Instruction::CCF() => [0b0011_1111, 0, 0],
            Instruction::STOP() => [0b0001_0000, 0, 0],
            Instruction::INC8(r8_operand) => {
                let operand = r8_operand.to_byte();
                let instruction = 0b0000_0100 | (operand << 3);
                [instruction, 0, 0]
            }
            Instruction::DEC8(r8_operand) => {
                let operand = r8_operand.to_byte();
                let instruction = 0b0000_0101 | (operand << 3);
                [instruction, 0, 0]
            }
            Instruction::JR(imm_8) => [0b0001_1000, imm_8 as u8, 0],
            Instruction::JRCC(cond, imm_8) => {
                let operand = cond.to_byte();
                let instruction = 0b0010_0000 | (operand << 3);
                [instruction, imm_8 as u8, 0]
            }
            Instruction::INC16(r16_operand) => {
                let operand = r16_operand.to_byte();
                let instruction = 0b0000_0011 | (operand << 4);
                [instruction, 0, 0]
            }
            Instruction::DEC16(r16_operand) => {
                let operand = r16_operand.to_byte();
                let instruction = 0b0000_1011 | (operand << 4);
                [instruction, 0, 0]
            }
            Instruction::ADDHL(r16_operand) => {
                let operand = r16_operand.to_byte();
                let instruction = 0b0000_1001 | (operand << 4);
                [instruction, 0, 0]
            }
            Instruction::LDToImmIndFromSP(imm_16) => {
                let instruction = 0b0000_1000;
                let imm_bytes = imm_16.to_le_bytes();
                [instruction, imm_bytes[0], imm_bytes[1]]
            }
            Instruction::LDImm8(r8_operand, imm_8) => {
                let operand = r8_operand.to_byte();
                let instruction = 0b0000_0110 | (operand << 3);
                [instruction, imm_8, 0]
            }
            Instruction::LDImm16(r16_operand, imm_16) => {
                let operand = r16_operand.to_byte();
                let instruction = 0b0000_0001 | (operand << 4);
                let imm_bytes = imm_16.to_le_bytes();
                [instruction, imm_bytes[0], imm_bytes[1]]
            }
            Instruction::LDToMemFromA(r16mem_opernad) => {
                let operand = r16mem_opernad.to_byte();
                let instruction = 0b0000_0010 | (operand << 4);
                [instruction, 0, 0]
            }
            Instruction::LDFromMemToA(r16mem_opernad) => {
                let operand = r16mem_opernad.to_byte();
                let instruction = 0b0000_1010 | (operand << 4);
                [instruction, 0, 0]
            }
            // Block 1
            Instruction::HALT() => [0b0111_0110, 0, 0],
            Instruction::LD(r8_operand_dest, r8_operand_source) => {
                let operand_dest = r8_operand_dest.to_byte();
                let operand_source = r8_operand_source.to_byte();
                [0b0100_0000 | (operand_dest << 3) | operand_source, 0, 0]
            }
            // Block 2
            Instruction::ADD(r8_operand) => {
                let operand = r8_operand.to_byte();
                [(0b1000_0000 | operand), 0, 0]
            }
            Instruction::ADC(r8_operand) => {
                let operand = r8_operand.to_byte();
                [(0b1000_1000 | operand), 0, 0]
            }
            Instruction::SUB(r8_operand) => {
                let operand = r8_operand.to_byte();
                [(0b1001_0000 | operand), 0, 0]
            }
            Instruction::SBC(r8_operand) => {
                let operand = r8_operand.to_byte();
                [(0b1001_1000 | operand), 0, 0]
            }
            Instruction::AND(r8_operand) => {
                let operand = r8_operand.to_byte();
                [(0b1010_0000 | operand), 0, 0]
            }
            Instruction::XOR(r8_operand) => {
                let operand = r8_operand.to_byte();
                [(0b1010_1000 | operand), 0, 0]
            }
            Instruction::OR(r8_operand) => {
                let operand = r8_operand.to_byte();
                [(0b1011_0000 | operand), 0, 0]
            }
            Instruction::CP(r8_operand) => {
                let operand = r8_operand.to_byte();
                [(0b1011_1000 | operand), 0, 0]
            }
            // Block 3
            Instruction::ADDImm(imm_8) => [0b1100_0110, imm_8, 0],
            Instruction::ADCImm(imm_8) => [0b1100_1110, imm_8, 0],
            Instruction::SUBImm(imm_8) => [0b1101_0110, imm_8, 0],
            Instruction::SBCImm(imm_8) => [0b1101_1110, imm_8, 0],
            Instruction::ANDImm(imm_8) => [0b1110_0110, imm_8, 0],
            Instruction::XORImm(imm_8) => [0b1110_1110, imm_8, 0],
            Instruction::ORImm(imm_8) => [0b1111_0110, imm_8, 0],
            Instruction::CPImm(imm_8) => [0b1111_1110, imm_8, 0],
            Instruction::RET() => [0b1100_1001, 0, 0],
            Instruction::RETI() => [0b1101_1001, 0, 0],
            Instruction::DI() => [0b1111_0011, 0, 0],
            Instruction::EI() => [0b1111_1011, 0, 0],
            Instruction::ADDSPImm(imm_8) => [0b1110_1000, imm_8 as u8, 0],
            Instruction::LDHLFromAdjustedSP(imm_8) => [0b1111_1000, imm_8 as u8, 0],
            Instruction::LDSPFromHL() => [0b1111_1001, 0, 0],
            Instruction::LDFromAToCInd() => [0b1110_0010, 0, 0],
            Instruction::LDToImmIndFromA8(imm_8) => [0b1110_0000, imm_8, 0],
            Instruction::LDToAFromCInd() => [0b1111_0010, 0, 0],
            Instruction::LDFromImmIndToA8(imm_8) => [0b1111_0000, imm_8, 0],
            Instruction::JPHL() => [0b1110_1001, 0, 0],
            Instruction::LDToImmIndFromA16(imm_16) => [
                0b1110_1010,
                imm_16.to_le_bytes()[0],
                imm_16.to_le_bytes()[1],
            ],
            Instruction::LDFromImmIndToA16(imm_16) => [
                0b1111_1010,
                imm_16.to_le_bytes()[0],
                imm_16.to_le_bytes()[1],
            ],
            Instruction::CALL(imm_16) => [
                0b1100_1101,
                imm_16.to_le_bytes()[0],
                imm_16.to_le_bytes()[1],
            ],
            Instruction::JP(imm_16) => [
                0b1100_0011,
                imm_16.to_le_bytes()[0],
                imm_16.to_le_bytes()[1],
            ],
            Instruction::RST(imm_8) => {
                let operand = imm_8 << 3;
                [0b1100_0111 | operand, 0, 0]
            }
            Instruction::RETCC(cond) => {
                let operand = cond.to_byte() << 3;
                [0b1100_0000 | operand, 0, 0]
            }
            Instruction::JPCC(cond, imm_16) => {
                let operand = cond.to_byte() << 3;
                [
                    0b1100_0010 | operand,
                    imm_16.to_le_bytes()[0],
                    imm_16.to_le_bytes()[1],
                ]
            }
            Instruction::CALLCC(cond, imm_16) => {
                let operand = cond.to_byte() << 3;
                [
                    0b1100_0100 | operand,
                    imm_16.to_le_bytes()[0],
                    imm_16.to_le_bytes()[1],
                ]
            }
            Instruction::POP(r16stk_operand) => {
                let operand = r16stk_operand.to_byte() << 4;
                [0b1100_0001 | operand, 0, 0]
            }
            Instruction::PUSH(r16stk_operand) => {
                let operand = r16stk_operand.to_byte() << 4;
                [0b1100_0101 | operand, 0, 0]
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::console::cpu::instruction::Instruction;
    use crate::console::cpu::instruction::Instruction::*;
    use crate::console::cpu::instruction_operands::*;

    fn assert_decode(instruction: Instruction, expected_size: u16) {
        let [byte1, byte2, byte3] = Instruction::encode(instruction.clone());
        let (decoded_instr, decoded_size) = Instruction::decode(byte1, byte2, byte3);
        assert_eq!(
            decoded_instr, instruction,
            "Instruction mismatch for 0x{:02X} 0x{:02X} 0x{:02X}",
            byte1, byte2, byte3
        );
        assert_eq!(
            decoded_size, expected_size,
            "Size mismatch for 0x{:02X} 0x{:02X} 0x{:02X}",
            byte1, byte2, byte3
        );
    }

    #[test]
    fn test_decode_cb() {
        let expected_size = 2;

        let operands_r8 = [
            R8Operand::A,
            R8Operand::B,
            R8Operand::C,
            R8Operand::D,
            R8Operand::E,
            R8Operand::H,
            R8Operand::L,
            R8Operand::HLInd,
        ];

        for operand in &operands_r8 {
            let instructions = [
                RLC(operand.clone()),
                RRC(operand.clone()),
                RL(operand.clone()),
                RR(operand.clone()),
                SLA(operand.clone()),
                SRA(operand.clone()),
                SWAP(operand.clone()),
                SRL(operand.clone()),
            ];

            for instruction in instructions {
                assert_decode(instruction, expected_size);
            }
        }

        for bit_index in 0..=7 {
            for operand in &operands_r8 {
                let instructions = [
                    BIT(bit_index, operand.clone()),
                    SET(bit_index, operand.clone()),
                    RESET(bit_index, operand.clone()),
                ];

                for instruction in instructions {
                    assert_decode(instruction, expected_size);
                }
            }
        }
    }

    #[test]
    fn test_decode_block_0() {
        // 1 byte, no params instructions
        {
            let expected_size = 1;
            let instructions = [
                NOP(),
                RLCA(),
                RRCA(),
                RLA(),
                RRA(),
                DAA(),
                CPL(),
                SCF(),
                CCF(),
            ];

            for instruction in instructions {
                assert_decode(instruction, expected_size);
            }
        }

        // Inc/Dec 8bit
        {
            let expected_size = 1;

            for operand in 0..=7 {
                let r8_operand = R8Operand::from_byte(operand);

                let instruction = INC8(r8_operand.clone());
                assert_decode(instruction, expected_size);

                let instruction = DEC8(r8_operand);
                assert_decode(instruction, expected_size);
            }
        }

        // Stop
        {
            assert_decode(STOP(), 2);
        }

        // LD r, n
        {
            let expected_size = 2;

            for operand in 0..=7 {
                let r8_operand = R8Operand::from_byte(operand);

                let instruction = LDImm8(r8_operand, 239);
                assert_decode(instruction, expected_size);
            }
        }

        // jump
        {
            let expected_size = 2;

            let random_u8 = 104;
            let instruction = JR(random_u8);

            assert_decode(instruction, expected_size);

            for operand in 0..=3 {
                let cond = FlowCondition::from_byte(operand);

                let random_u8 = -80;
                let instruction = JRCC(cond, random_u8);
                assert_decode(instruction, expected_size);
            }
        }

        // inc/dec/addhl 16
        {
            let expected_size = 1;

            for operand in 0..=3 {
                let r16_operand = R16Operand::from_byte(operand);

                let instructions = [
                    INC16(r16_operand.clone()),
                    DEC16(r16_operand.clone()),
                    ADDHL(r16_operand.clone()),
                ];

                for instruction in instructions {
                    assert_decode(instruction, expected_size);
                }
            }
        }

        // ld [imm16], sp/ld rr, nn
        {
            let expected_size = 3;

            for operand in 0..=3 {
                let r16_operand = R16Operand::from_byte(operand);

                let instruction = LDImm16(r16_operand, 32450);
                assert_decode(instruction, expected_size);
            }

            let instruction = LDToImmIndFromSP(12450);
            assert_decode(instruction, expected_size);
        }

        // ld [r16mem], a/ld a, [r16mem]
        {
            let expected_size = 1;


            for operand in 0..=3 {
                let r16mem_operand = R16MemOperand::from_byte(operand);

                let instruction = LDToMemFromA(r16mem_operand.clone());
                assert_decode(instruction, expected_size);

                let instruction = LDFromMemToA(r16mem_operand);
                assert_decode(instruction, expected_size);
            }
        }
    }

    #[test]
    fn test_decode_block_1() {
        let expected_size = 1;

        let operands_r8 = [
            R8Operand::A,
            R8Operand::B,
            R8Operand::C,
            R8Operand::D,
            R8Operand::E,
            R8Operand::H,
            R8Operand::L,
            R8Operand::HLInd,
        ];

        for register_source in &operands_r8 {
            for register_dest in &operands_r8 {
                match (register_source, register_dest) {
                    (R8Operand::HLInd, R8Operand::HLInd) => {
                        let instruction = HALT();
                        assert_decode(instruction, expected_size);
                    }
                    _ => {
                        let instruction = LD(register_dest.clone(), register_source.clone());
                        assert_decode(instruction, expected_size);
                    }
                }
            }
        }
    }

    #[test]
    fn test_decode_block_2() {
        let expected_size = 1;

        let operands_r8 = [
            R8Operand::A,
            R8Operand::B,
            R8Operand::C,
            R8Operand::D,
            R8Operand::E,
            R8Operand::H,
            R8Operand::L,
            R8Operand::HLInd,
        ];

        for reg in operands_r8 {
            let instructions = [
                ADD(reg.clone()),
                ADC(reg.clone()),
                SUB(reg.clone()),
                SUB(reg.clone()),
                AND(reg.clone()),
                XOR(reg.clone()),
                OR(reg.clone()),
                CP(reg.clone()),
            ];

            for instruction in instructions {
                assert_decode(instruction, expected_size);
            }
        }
    }

    #[test]
    fn test_decode_block_3() {
        // test 1 byte, no params instructions
        {
            let expected_size = 1;
            let instructions = [
                DI(),
                EI(),
                LDSPFromHL(),
                LDFromAToCInd(),
                LDToAFromCInd(),
                RET(),
                RETI(),
                JPHL(),
            ];

            for instruction in instructions {
                assert_decode(instruction, expected_size);
            }
        }

        // 8bit imm
        {
            let expected_size = 2;

            let numbers: [u8; 4] = [200, 127, 0, 50];

            for number in numbers {
                let instructions = [
                    ADDImm(number),
                    ADCImm(number),
                    SUBImm(number),
                    SBCImm(number),
                    ANDImm(number),
                    ORImm(number),
                    XORImm(number),
                    CPImm(number),
                    ADDSPImm(number as i8),
                    LDHLFromAdjustedSP(number as i8),
                    LDFromImmIndToA8(number),
                    LDToImmIndFromA8(number),
                ];

                for instruction in instructions {
                    assert_decode(instruction, expected_size);
                }
            }
        }

        // Stack
        {
            let expected_size = 1;

            for operand in 0..=3 {
                let r16stk_operand = R16StkOperand::from_byte(operand);

                let instructions = [POP(r16stk_operand.clone()), PUSH(r16stk_operand.clone())];

                for instruction in instructions {
                    assert_decode(instruction, expected_size);
                }
            }
        }

        // reset
        {
            let expected_size = 1;
            let instruction = RST(7);
            assert_decode(instruction, expected_size);
        }

        // ret cond
        {
            let expected_size = 1;

            for operand in 0..=3 {
                let cond = FlowCondition::from_byte(operand);
                let instruction = RETCC(cond);
                assert_decode(instruction, expected_size);
            }
        }

        // 16-bit cond
        {
            let expected_size = 3;

            for operand in 0..=3 {
                let cond = FlowCondition::from_byte(operand);
                let instructions = [JPCC(cond.clone(), 33402), CALLCC(cond.clone(), 10490)];

                for instruction in instructions {
                    assert_decode(instruction, expected_size);
                }
            }
        }

        // 16-bit
        {
            let expected_size = 3;

            let instructions = [
                JP(33402),
                CALL(10490),
                LDFromImmIndToA16(23400),
                LDToImmIndFromA16(10000),
            ];

            for instruction in instructions {
                assert_decode(instruction, expected_size);
            }
        }
    }
}
