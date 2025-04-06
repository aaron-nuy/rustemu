pub use crate::console::cpu::instruction_operands::*;
pub use crate::console::cpu::register::*;

// TODO: Take InstructionOperands as parameter instead of registers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    LD(Register, Register),
    LDImm(Register, u8),
    LDFromHLInd(Register),
    LDToHLInd(Register),
    LDToHlIndImm(u8),
    LDFromBCIndToA(),
    LDFromDEIndToA(),
    LDToBCIndFromA(),
    LDToDEIndFromA(),
    LDFromImmIndToA(u16),
    LDToImmIndFromA(u16),
    LDToAFromCInd(),
    LDFromAToCInd(),
    LDFromImmIndToA8(u8),
    LDToImmIndFromA8(u8),
    LDFromHLIndDecToA(),
    LDToHLIndDecFromA(),
    LDFromHLIndIncToA(),
    LDToHLIndIncFromA(),
    LDImm16(Register16, u16),
    LDToImmIndFromSP(u16),
    LDSPFromHL(),
    PUSH(Register16),
    POP(Register16),
    LDHLFromAdjustedSP(i8),
    ADD(Register),
    ADDHLInd(),
    ADDImm(u8),
    ADDC(Register),
    ADDCHLInd(),
    ADDCImm(u8),
    SUB(Register),
    SUBHLInd(),
    SUBImm(u8),
    SUBC(Register),
    SUBCHLInd(),
    SUBCImm(u8),
    CP(Register),
    CPHLInd(),
    CPImm(u8),
    INC(Register),
    INCHLInd(),
    DEC(Register),
    DECHLInd(),
    AND(Register),
    ANDHLInd(),
    ANDImm(u8),
    OR(Register),
    ORHLInd(),
    ORImm(u8),
    XOR(Register),
    XORHLInd(),
    XORImm(u8),
    CCF(),
    SCF(),
    DAA(),
    CPL(),
    INC16(Register16),
    DEC16(Register16),
    ADDHL(Register16),
    ADDSPImm(i8),
    RLCA(),
    RRCA(),
    RRA(),
    RLA(),
    RLCR(Register),
    RRCR(Register),
    RLCHLInd(),
    RRCHLInd(),
    RLR(Register),
    RLHLInd(),
    RRR(Register),
    RRHLInd(),
    SRAR(Register),
    SRAHLInd(),
    SLAR(Register),
    SLAHLInd(),
    SRLR(Register),
    SRLHLInd(),
    SWAPR(Register),
    SWAPHLInd(),
    BITR(u8, Register),
    BITHLInd(u8),
    RESETR(u8, Register),
    RESETHLInd(u8),
    SETR(u8, Register),
    SETHLInd(u8),
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
                0 => match r8_operand {
                    R8Operand::HLInd => (Instruction::RLCHLInd(), 2),
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        (Instruction::RLCR(register), 2)
                    }
                },
                1 => match r8_operand {
                    R8Operand::HLInd => (Instruction::RRCHLInd(), 2),
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        (Instruction::RRCR(register), 2)
                    }
                },
                2 => match r8_operand {
                    R8Operand::HLInd => (Instruction::RLHLInd(), 2),
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        (Instruction::RLR(register), 2)
                    }
                },
                3 => match r8_operand {
                    R8Operand::HLInd => (Instruction::RRHLInd(), 2),
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        (Instruction::RRR(register), 2)
                    }
                },
                4 => match r8_operand {
                    R8Operand::HLInd => (Instruction::SLAHLInd(), 2),
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        (Instruction::SLAR(register), 2)
                    }
                },
                5 => match r8_operand {
                    R8Operand::HLInd => (Instruction::SRAHLInd(), 2),
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        (Instruction::SRAR(register), 2)
                    }
                },
                6 => match r8_operand {
                    R8Operand::HLInd => (Instruction::SWAPHLInd(), 2),
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        (Instruction::SWAPR(register), 2)
                    }
                },
                7 => match r8_operand {
                    R8Operand::HLInd => (Instruction::SRLHLInd(), 2),
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        (Instruction::SRLR(register), 2)
                    }
                },
                _ => panic!("Unknown CB instruction {}", opcode),
            },
            1 => match r8_operand {
                R8Operand::HLInd => (Instruction::BITHLInd(bit_index), 2),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::BITR(bit_index, register), 2)
                }
            },
            2 => match r8_operand {
                R8Operand::HLInd => (Instruction::RESETHLInd(bit_index), 2),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::RESETR(bit_index, register), 2)
                }
            },
            3 => match r8_operand {
                R8Operand::HLInd => (Instruction::SETHLInd(bit_index), 2),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::SETR(bit_index, register), 2)
                }
            },
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

                match r8_operand {
                    R8Operand::HLInd => (Instruction::LDToHlIndImm(imm_8), 2),
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        (Instruction::LDImm(register, imm_8), 2)
                    }
                }
            }
            opcode if (opcode & 0b1100_0111) == 0b0000_0100 => {
                let operand = (opcode & 0b0011_1000) >> 3;
                let r8_operand = R8Operand::from_byte(operand);

                match r8_operand {
                    R8Operand::HLInd => (Instruction::INCHLInd(), 1),
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        (Instruction::INC(register), 1)
                    }
                }
            }
            opcode if (opcode & 0b1100_0111) == 0b0000_0101 => {
                let operand = (opcode & 0b0011_1000) >> 3;
                let r8_operand = R8Operand::from_byte(operand);

                match r8_operand {
                    R8Operand::HLInd => (Instruction::DECHLInd(), 1),
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        (Instruction::DEC(register), 1)
                    }
                }
            }
            opcode if (opcode & 0b1100_1111) == 0b0000_0011 => {
                let operand = (opcode & 0b0011_0000) >> 4;
                let r16_operand = R16Operand::from_byte(operand);
                let register_16 = Register16::from_r16_operand(r16_operand);
                (Instruction::INC16(register_16), 1)
            }
            opcode if (opcode & 0b1100_1111) == 0b0000_1011 => {
                let operand = (opcode & 0b0011_0000) >> 4;
                let r16_operand = R16Operand::from_byte(operand);
                let register_16 = Register16::from_r16_operand(r16_operand);
                (Instruction::DEC16(register_16), 1)
            }
            opcode if (opcode & 0b1100_1111) == 0b0000_1001 => {
                let operand = (opcode & 0b0011_0000) >> 4;
                let r16_operand = R16Operand::from_byte(operand);
                let register_16 = Register16::from_r16_operand(r16_operand);
                (Instruction::ADDHL(register_16), 1)
            }
            opcode if (opcode & 0b1100_1111) == 0b0000_0001 => {
                let operand = (opcode & 0b0011_0000) >> 4;
                let r16_operand = R16Operand::from_byte(operand);
                let register_16 = Register16::from_r16_operand(r16_operand);
                (Instruction::LDImm16(register_16, imm_16), 3)
            }
            opcode if (opcode & 0b1100_1111) == 0b0000_0010 => {
                let operand = (opcode & 0b0011_0000) >> 4;
                let r16mem_operand = R16MemOperand::from_byte(operand);

                match r16mem_operand {
                    R16MemOperand::BC => (Instruction::LDToBCIndFromA(), 1),
                    R16MemOperand::DE => (Instruction::LDToDEIndFromA(), 1),
                    R16MemOperand::HLI => (Instruction::LDToHLIndIncFromA(), 1),
                    R16MemOperand::HLD => (Instruction::LDToHLIndDecFromA(), 1),
                }
            }
            opcode if (opcode & 0b1100_1111) == 0b0000_1010 => {
                let operand = (opcode & 0b0011_0000) >> 4;
                let r16mem_operand = R16MemOperand::from_byte(operand);

                match r16mem_operand {
                    R16MemOperand::BC => (Instruction::LDFromBCIndToA(), 1),
                    R16MemOperand::DE => (Instruction::LDFromDEIndToA(), 1),
                    R16MemOperand::HLI => (Instruction::LDFromHLIndIncToA(), 1),
                    R16MemOperand::HLD => (Instruction::LDFromHLIndDecToA(), 1),
                }
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
                    (R8Operand::HLInd, _) => {
                        let register_source = Register::from_r8_operand(r8_operand_source);
                        (Instruction::LDToHLInd(register_source), 1)
                    }
                    (_, R8Operand::HLInd) => {
                        let register_dest = Register::from_r8_operand(r8_operand_dest);
                        (Instruction::LDFromHLInd(register_dest), 1)
                    }
                    (_, _) => {
                        let register_dest = Register::from_r8_operand(r8_operand_dest);
                        let register_source = Register::from_r8_operand(r8_operand_source);
                        (Instruction::LD(register_dest, register_source), 1)
                    }
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
            0 => match r8_operand {
                R8Operand::HLInd => (Instruction::ADDHLInd(), 1),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::ADD(register), 1)
                }
            },
            1 => match r8_operand {
                R8Operand::HLInd => (Instruction::ADDCHLInd(), 1),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::ADDC(register), 1)
                }
            },
            2 => match r8_operand {
                R8Operand::HLInd => (Instruction::SUBHLInd(), 1),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::SUB(register), 1)
                }
            },
            3 => match r8_operand {
                R8Operand::HLInd => (Instruction::SUBCHLInd(), 1),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::SUBC(register), 1)
                }
            },
            4 => match r8_operand {
                R8Operand::HLInd => (Instruction::ANDHLInd(), 1),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::AND(register), 1)
                }
            },
            5 => match r8_operand {
                R8Operand::HLInd => (Instruction::XORHLInd(), 1),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::XOR(register), 1)
                }
            },
            6 => match r8_operand {
                R8Operand::HLInd => (Instruction::ORHLInd(), 1),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::OR(register), 1)
                }
            },
            7 => match r8_operand {
                R8Operand::HLInd => (Instruction::CPHLInd(), 1),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::CP(register), 1)
                }
            },
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
            0b1100_1110 => (Instruction::ADDCImm(imm_8), 2),
            0b1101_0110 => (Instruction::SUBImm(imm_8), 2),
            0b1101_1110 => (Instruction::SUBCImm(imm_8), 2),
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
                let register_16 = Register16::from_r16stk_operand(r16stk_operand);

                (Instruction::POP(register_16), 1)
            }
            opcode if (opcode & 0b1100_1111) == 0b1100_0101 => {
                let operand = (opcode & 0b0011_0000) >> 4;
                let r16stk_operand = R16StkOperand::from_byte(operand);
                let register_16 = Register16::from_r16stk_operand(r16stk_operand);

                (Instruction::PUSH(register_16), 1)
            }
            // Table 4
            0b1110_0010 => (Instruction::LDFromAToCInd(), 1),
            0b1110_0000 => (Instruction::LDToImmIndFromA8(imm_8), 2),
            0b1110_1010 => (Instruction::LDToImmIndFromA(imm_16), 3),
            0b1111_0010 => (Instruction::LDToAFromCInd(), 1),
            0b1111_0000 => (Instruction::LDFromImmIndToA8(imm_8), 2),
            0b1111_1010 => (Instruction::LDFromImmIndToA(imm_16), 3),
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
        let hl_ind_r8_op_byte = R8Operand::HLInd.to_byte();

        match instruction {
            // CB
            Instruction::RLCR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                [0xCB, (0b0000_0000 | operand), 0]
            }
            Instruction::RRCR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                [0xCB, (0b0000_1000 | operand), 0]
            }
            Instruction::RLR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                [0xCB, (0b0001_0000 | operand), 0]
            }
            Instruction::RRR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                [0xCB, (0b0001_1000 | operand), 0]
            }
            Instruction::SLAR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                [0xCB, (0b0010_0000 | operand), 0]
            }
            Instruction::SRAR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                [0xCB, (0b0010_1000 | operand), 0]
            }
            Instruction::SWAPR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                [0xCB, (0b0011_0000 | operand), 0]
            }
            Instruction::SRLR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                [0xCB, (0b0011_1000 | operand), 0]
            }
            Instruction::RLCHLInd() => [0xCB, (0b0000_0000 | hl_ind_r8_op_byte), 0],
            Instruction::RRCHLInd() => [0xCB, (0b0000_1000 | hl_ind_r8_op_byte), 0],
            Instruction::RLHLInd() => [0xCB, (0b0001_0000 | hl_ind_r8_op_byte), 0],
            Instruction::RRHLInd() => [0xCB, (0b0001_1000 | hl_ind_r8_op_byte), 0],
            Instruction::SLAHLInd() => [0xCB, (0b0010_0000 | hl_ind_r8_op_byte), 0],
            Instruction::SRAHLInd() => [0xCB, (0b0010_1000 | hl_ind_r8_op_byte), 0],
            Instruction::SWAPHLInd() => [0xCB, (0b0011_0000 | hl_ind_r8_op_byte), 0],
            Instruction::SRLHLInd() => [0xCB, (0b0011_1000 | hl_ind_r8_op_byte), 0],
            Instruction::BITR(bit_index, register) => {
                let operand = R8Operand::to_byte_from_register(register);
                let adjusted_bit_index = bit_index << 3;
                [0xCB, (0b0100_0000 | adjusted_bit_index | operand), 0]
            }
            Instruction::RESETR(bit_index, register) => {
                let operand = R8Operand::to_byte_from_register(register);
                let adjusted_bit_index = bit_index << 3;
                [0xCB, (0b1000_0000 | adjusted_bit_index | operand), 0]
            }
            Instruction::SETR(bit_index, register) => {
                let operand = R8Operand::to_byte_from_register(register);
                let adjusted_bit_index = bit_index << 3;
                [0xCB, (0b1100_0000 | adjusted_bit_index | operand), 0]
            }
            Instruction::BITHLInd(bit_index) => {
                let adjusted_bit_index = bit_index << 3;
                [
                    0xCB,
                    (0b0100_0000 | adjusted_bit_index | hl_ind_r8_op_byte),
                    0,
                ]
            }
            Instruction::RESETHLInd(bit_index) => {
                let adjusted_bit_index = bit_index << 3;
                [
                    0xCB,
                    (0b1000_0000 | adjusted_bit_index | hl_ind_r8_op_byte),
                    0,
                ]
            }
            Instruction::SETHLInd(bit_index) => {
                let adjusted_bit_index = bit_index << 3;
                [
                    0xCB,
                    (0b1100_0000 | adjusted_bit_index | hl_ind_r8_op_byte),
                    0,
                ]
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
            Instruction::INC(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                let instruction = 0b0000_0100 | (operand << 3);
                [instruction, 0, 0]
            }
            Instruction::DEC(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                let instruction = 0b0000_0101 | (operand << 3);
                [instruction, 0, 0]
            }
            Instruction::INCHLInd() => {
                let operand = hl_ind_r8_op_byte;
                let instruction = 0b0000_0100 | (operand << 3);
                [instruction, 0, 0]
            }
            Instruction::DECHLInd() => {
                let operand = hl_ind_r8_op_byte;
                let instruction = 0b0000_0101 | (operand << 3);
                [instruction, 0, 0]
            }
            Instruction::JR(imm_8) => [0b0001_1000, imm_8 as u8, 0],
            Instruction::JRCC(cond, imm_8) => {
                let operand = cond.to_byte();
                let instruction = 0b0010_0000 | (operand << 3);
                [instruction, imm_8 as u8, 0]
            }
            Instruction::INC16(register_16) => {
                let operand = R16Operand::to_byte_from_register_16(register_16);
                let instruction = 0b0000_0011 | (operand << 4);
                [instruction, 0, 0]
            }
            Instruction::DEC16(register_16) => {
                let operand = R16Operand::to_byte_from_register_16(register_16);
                let instruction = 0b0000_1011 | (operand << 4);
                [instruction, 0, 0]
            }
            Instruction::ADDHL(register_16) => {
                let operand = R16Operand::to_byte_from_register_16(register_16);
                let instruction = 0b0000_1001 | (operand << 4);
                [instruction, 0, 0]
            }
            Instruction::LDToImmIndFromSP(imm_16) => {
                let instruction = 0b0000_1000;
                let imm_bytes = imm_16.to_le_bytes();
                [instruction, imm_bytes[0], imm_bytes[1]]
            }
            Instruction::LDImm(register, imm_8) => {
                let operand = R8Operand::to_byte_from_register(register);
                let instruction = 0b0000_0110 | (operand << 3);
                [instruction, imm_8, 0]
            }
            Instruction::LDToHlIndImm(imm_8) => {
                let operand = hl_ind_r8_op_byte;
                let instruction = 0b0000_0110 | (operand << 3);
                [instruction, imm_8, 0]
            }
            Instruction::LDImm16(register_16, imm_16) => {
                let operand = R16Operand::to_byte_from_register_16(register_16);
                let instruction = 0b0000_0001 | (operand << 4);
                let imm_bytes = imm_16.to_le_bytes();
                [instruction, imm_bytes[0], imm_bytes[1]]
            }
            Instruction::LDToBCIndFromA() => {
                let operand = R16MemOperand::BC.to_byte();
                let instruction = 0b0000_0010 | (operand << 4);
                [instruction, 0, 0]
            }
            Instruction::LDToDEIndFromA() => {
                let operand = R16MemOperand::DE.to_byte();
                let instruction = 0b0000_0010 | (operand << 4);
                [instruction, 0, 0]
            }
            Instruction::LDToHLIndIncFromA() => {
                let operand = R16MemOperand::HLI.to_byte();
                let instruction = 0b0000_0010 | (operand << 4);
                [instruction, 0, 0]
            }
            Instruction::LDToHLIndDecFromA() => {
                let operand = R16MemOperand::HLD.to_byte();
                let instruction = 0b0000_0010 | (operand << 4);
                [instruction, 0, 0]
            }
            Instruction::LDFromBCIndToA() => {
                let operand = R16MemOperand::BC.to_byte();
                let instruction = 0b0000_1010 | (operand << 4);
                [instruction, 0, 0]
            }
            Instruction::LDFromDEIndToA() => {
                let operand = R16MemOperand::DE.to_byte();
                let instruction = 0b0000_1010 | (operand << 4);
                [instruction, 0, 0]
            }
            Instruction::LDFromHLIndIncToA() => {
                let operand = R16MemOperand::HLI.to_byte();
                let instruction = 0b0000_1010 | (operand << 4);
                [instruction, 0, 0]
            }
            Instruction::LDFromHLIndDecToA() => {
                let operand = R16MemOperand::HLD.to_byte();
                let instruction = 0b0000_1010 | (operand << 4);
                [instruction, 0, 0]
            }
            // Block 1
            Instruction::HALT() => [0b0111_0110, 0, 0],
            Instruction::LD(register_dest, register_source) => {
                let operand_dest = R8Operand::to_byte_from_register(register_dest);
                let operand_source = R8Operand::to_byte_from_register(register_source);
                [0b0100_0000 | (operand_dest << 3) | operand_source, 0, 0]
            }
            Instruction::LDToHLInd(register_source) => {
                let operand_dest = hl_ind_r8_op_byte;
                let operand_source = R8Operand::to_byte_from_register(register_source);
                [0b0100_0000 | (operand_dest << 3) | operand_source, 0, 0]
            }
            // Block 2
            Instruction::ADD(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                [(0b1000_0000 | operand), 0, 0]
            }
            Instruction::ADDC(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                [(0b1000_1000 | operand), 0, 0]
            }
            Instruction::SUB(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                [(0b1001_0000 | operand), 0, 0]
            }
            Instruction::SUBC(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                [(0b1001_1000 | operand), 0, 0]
            }
            Instruction::AND(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                [(0b1010_0000 | operand), 0, 0]
            }
            Instruction::XOR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                [(0b1010_1000 | operand), 0, 0]
            }
            Instruction::OR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                [(0b1011_0000 | operand), 0, 0]
            }
            Instruction::CP(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                [(0b1011_1000 | operand), 0, 0]
            }
            Instruction::ADDHLInd() => [(0b1000_0000 | hl_ind_r8_op_byte), 0, 0],
            Instruction::ADDCHLInd() => [(0b1000_1000 | hl_ind_r8_op_byte), 0, 0],
            Instruction::SUBHLInd() => [(0b1001_0000 | hl_ind_r8_op_byte), 0, 0],
            Instruction::SUBCHLInd() => [(0b1001_1000 | hl_ind_r8_op_byte), 0, 0],
            Instruction::ANDHLInd() => [(0b1010_0000 | hl_ind_r8_op_byte), 0, 0],
            Instruction::XORHLInd() => [(0b1010_1000 | hl_ind_r8_op_byte), 0, 0],
            Instruction::ORHLInd() => [(0b1011_0000 | hl_ind_r8_op_byte), 0, 0],
            Instruction::CPHLInd() => [(0b1011_1000 | hl_ind_r8_op_byte), 0, 0],
            // Block 3
            Instruction::ADDImm(imm_8) => [0b1100_0110, imm_8, 0],
            Instruction::ADDCImm(imm_8) => [0b1100_1110, imm_8, 0],
            Instruction::SUBImm(imm_8) => [0b1101_0110, imm_8, 0],
            Instruction::SUBCImm(imm_8) => [0b1101_1110, imm_8, 0],
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
            Instruction::LDToImmIndFromA(imm_16) => [
                0b1110_1010,
                imm_16.to_le_bytes()[0],
                imm_16.to_le_bytes()[1],
            ],
            Instruction::LDFromImmIndToA(imm_16) => [
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
            Instruction::POP(register_16) => {
                let operand = R16StkOperand::to_byte_from_register_16(register_16) << 4;
                [0b1100_0001 | operand, 0, 0]
            }
            Instruction::PUSH(register_16) => {
                let operand = R16StkOperand::to_byte_from_register_16(register_16) << 4;
                [0b1100_0101 | operand, 0, 0]
            }
            _ => panic!("Not Block 3 instruction"),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::console::cpu::instruction::Instruction;
    use crate::console::cpu::instruction::Instruction::*;
    use crate::console::cpu::instruction_operands::*;
    use crate::console::cpu::register::*;

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
            Register::A,
            Register::B,
            Register::C,
            Register::D,
            Register::E,
            Register::H,
            Register::L,
        ];

        for reg in operands_r8 {
            let instructions = [
                RLCR(reg.clone()),
                RRCR(reg.clone()),
                RLR(reg.clone()),
                RRR(reg.clone()),
                SLAR(reg.clone()),
                SRAR(reg.clone()),
                SWAPR(reg.clone()),
                SRLR(reg.clone()),
            ];

            for instruction in instructions {
                assert_decode(instruction, expected_size);
            }
        }

        let instructions = [
            RLCHLInd(),
            RRCHLInd(),
            RLHLInd(),
            RRHLInd(),
            SLAHLInd(),
            SRAHLInd(),
            SWAPHLInd(),
            SRLHLInd(),
        ];

        for instruction in instructions {
            assert_decode(instruction, expected_size);
        }

        let operands_r8 = [
            Register::A,
            Register::B,
            Register::C,
            Register::D,
            Register::E,
            Register::H,
            Register::L,
        ];

        for bit_index in 0..=7 {
            for reg in &operands_r8 {
                let instructions = [
                    BITR(bit_index, reg.clone()),
                    SETR(bit_index, reg.clone()),
                    RESETR(bit_index, reg.clone()),
                ];

                for instruction in instructions {
                    assert_decode(instruction, expected_size);
                }
            }

            let instructions = [
                BITHLInd(bit_index),
                SETHLInd(bit_index),
                RESETHLInd(bit_index),
            ];

            for instruction in instructions {
                assert_decode(instruction, expected_size);
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

                match r8_operand {
                    R8Operand::HLInd => {
                        let instruction = INCHLInd();
                        assert_decode(instruction, expected_size);

                        let instruction = DECHLInd();
                        assert_decode(instruction, expected_size);
                    }
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);

                        let instruction = INC(register.clone());
                        assert_decode(instruction, expected_size);

                        let instruction = DEC(register);
                        assert_decode(instruction, expected_size);
                    }
                }
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

                match r8_operand {
                    R8Operand::HLInd => {
                        let random_u8: u8 = 233;
                        let instruction = LDToHlIndImm(random_u8);
                        assert_decode(instruction, expected_size);
                    }
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        let random_u8: u8 = 201;
                        let instruction = LDImm(register.clone(), random_u8);
                        assert_decode(instruction, expected_size);
                    }
                }
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
                let register_16 = Register16::from_r16_operand(r16_operand);

                let instructions = [
                    INC16(register_16.clone()),
                    DEC16(register_16.clone()),
                    ADDHL(register_16.clone()),
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
                let register_16 = Register16::from_r16_operand(r16_operand);

                let instruction = LDImm16(register_16, 32450);
                assert_decode(instruction, expected_size);
            }

            let instruction = LDToImmIndFromSP(12450);
            assert_decode(instruction, expected_size);
        }

        // ld [r16mem], a/ld a, [r16mem]
        {
            let expected_size = 1;

            let instruction = LDToBCIndFromA();
            assert_decode(instruction, expected_size);

            let instruction = LDToDEIndFromA();
            assert_decode(instruction, expected_size);

            let instruction = LDToHLIndIncFromA();
            assert_decode(instruction, expected_size);

            let instruction = LDToHLIndDecFromA();
            assert_decode(instruction, expected_size);

            let instruction = LDFromBCIndToA();
            assert_decode(instruction, expected_size);

            let instruction = LDFromDEIndToA();
            assert_decode(instruction, expected_size);

            let instruction = LDFromHLIndIncToA();
            assert_decode(instruction, expected_size);

            let instruction = LDFromHLIndDecToA();
            assert_decode(instruction, expected_size);
        }
    }

    #[test]
    fn test_decode_block_1() {
        let expected_size = 1;

        let operands_r8 = [
            Register::A,
            Register::B,
            Register::C,
            Register::D,
            Register::E,
            Register::H,
            Register::L,
        ];

        for register_source in &operands_r8 {
            for register_dest in &operands_r8 {
                let instruction = LD(register_dest.clone(), register_source.clone());
                let [instruction_byte, byte1, byte2] = Instruction::encode(instruction.clone());
                assert_decode(instruction, expected_size);
            }

            let instruction = LDToHLInd(register_source.clone());
            let [instruction_byte, byte1, byte2] = Instruction::encode(instruction.clone());
            assert_decode(instruction, expected_size);
        }

        let instruction = HALT();
        let [instruction_byte, byte1, byte2] = Instruction::encode(instruction.clone());
        assert_decode(instruction, expected_size);
    }

    #[test]
    fn test_decode_block_2() {
        let expected_size = 1;

        let operands_r8 = [
            Register::A,
            Register::B,
            Register::C,
            Register::D,
            Register::E,
            Register::H,
            Register::L,
        ];

        for reg in operands_r8 {
            let instructions = [
                ADD(reg.clone()),
                ADDC(reg.clone()),
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

        let instructions = [
            ADDHLInd(),
            ADDCHLInd(),
            SUBHLInd(),
            SUBCHLInd(),
            ANDHLInd(),
            XORHLInd(),
            ORHLInd(),
            CPHLInd(),
        ];

        for instruction in instructions {
            assert_decode(instruction, expected_size);
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
                    ADDCImm(number),
                    SUBImm(number),
                    SUBCImm(number),
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
                let register_16 = Register16::from_r16stk_operand(r16stk_operand);

                let instructions = [POP(register_16.clone()), PUSH(register_16.clone())];

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

        // 16-bit cond
        {
            let expected_size = 3;

            let instructions = [
                JP(33402),
                CALL(10490),
                LDFromImmIndToA(23400),
                LDToImmIndFromA(23400),
            ];

            for instruction in instructions {
                assert_decode(instruction, expected_size);
            }
        }
    }
}
