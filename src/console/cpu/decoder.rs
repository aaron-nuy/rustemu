use crate::console::cpu::instruction::*;

pub fn decode(first_byte: u8, second_byte: u8, third_byte: u8) -> (Instruction, u16) {
    if first_byte == 0xcb {
        decode_cb(second_byte)
    } else {
        let imm16_operand = u16::from_le_bytes([second_byte, third_byte]);
        decode_generic(first_byte, second_byte, imm16_operand)
    }
}

fn get_operand_from_opcode(
    opcode: u8,
    operand_type: OperandType,
    is_destination: bool,
    is_cb: bool,
) -> u8 {
    let block = Block::from_byte(opcode, is_cb);

    let (mask, shift_right) = match operand_type {
        OperandType::R8Operand => match block {
            Block::ZERO => (0b00111000, 3),
            Block::ONE => {
                if is_destination {
                    (0b00111000, 3)
                } else {
                    (0b00000111, 0)
                }
            }
            Block::TWO | Block::CB => (0b00000111, 0),
            Block::THREE => panic!("Attempting to access R8Operand from block THREE"),
        },
        OperandType::R16Operand => match block {
            Block::ZERO => (0b00110000, 4),
            _ => panic!("Attempting to access R16Operand from block other than ZERO"),
        },
        OperandType::R16StkOperand => match block {
            Block::THREE => (0b00110000, 4),
            _ => panic!("Attempting to access R16StkOperand from block other than THREE"),
        },
        OperandType::R16MemOperand => match block {
            Block::ZERO => (0b00110000, 4),
            _ => panic!("Attempting to access R16MemOperand from block other than ZERO"),
        },
        OperandType::FlowCondition => match block {
            Block::ZERO | Block::THREE => (0b00011000, 3),
            _ => {
                panic!("Attempting to access FlowCondition from block other than ZERO or THREE")
            }
        },
        OperandType::BitIndex => match block {
            Block::CB => (0b00111000, 3),
            _ => panic!("Attempting to access BitIndex from block other than CB"),
        },
        OperandType::ResetTarget => match block {
            Block::THREE => (0b00111000, 3),
            _ => panic!("Attempting to access ResetTarget from block other than THREE"),
        },
    };

    (opcode & mask) >> shift_right
}

fn decode_cb(opcode: u8) -> (Instruction, u16) {
    match (opcode & 0b11000000) >> 6 {
        0 => match (opcode & 0b00111000) >> 3 {
            0 => {
                let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, true);
                let r8_operand = R8Operand::from_byte(operand);

                match r8_operand {
                    R8Operand::HLInd => (Instruction::RLCHLInd(), 2),
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        (Instruction::RLCR(register), 2)
                    }
                }
            }
            1 => {
                let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, true);
                let r8_operand = R8Operand::from_byte(operand);

                match r8_operand {
                    R8Operand::HLInd => (Instruction::RRCHLInd(), 2),
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        (Instruction::RRCR(register), 2)
                    }
                }
            }
            2 => {
                let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, true);
                let r8_operand = R8Operand::from_byte(operand);

                match r8_operand {
                    R8Operand::HLInd => (Instruction::RLHLInd(), 2),
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        (Instruction::RLR(register), 2)
                    }
                }
            }
            3 => {
                let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, true);
                let r8_operand = R8Operand::from_byte(operand);

                match r8_operand {
                    R8Operand::HLInd => (Instruction::RRHLInd(), 2),
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        (Instruction::RRR(register), 2)
                    }
                }
            }
            4 => {
                let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, true);
                let r8_operand = R8Operand::from_byte(operand);

                match r8_operand {
                    R8Operand::HLInd => (Instruction::SLAHLInd(), 2),
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        (Instruction::SLAR(register), 2)
                    }
                }
            }
            5 => {
                let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, true);
                let r8_operand = R8Operand::from_byte(operand);

                match r8_operand {
                    R8Operand::HLInd => (Instruction::SRAHLInd(), 2),
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        (Instruction::SRAR(register), 2)
                    }
                }
            }
            6 => {
                let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, true);
                let r8_operand = R8Operand::from_byte(operand);

                match r8_operand {
                    R8Operand::HLInd => (Instruction::SWAPHLInd(), 2),
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        (Instruction::SWAPR(register), 2)
                    }
                }
            }
            7 => {
                let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, true);
                let r8_operand = R8Operand::from_byte(operand);

                match r8_operand {
                    R8Operand::HLInd => (Instruction::SRLHLInd(), 2),
                    _ => {
                        let register = Register::from_r8_operand(r8_operand);
                        (Instruction::SRLR(register), 2)
                    }
                }
            }
            _ => panic!("Unknown CB instruction {}", opcode),
        },
        1 => {
            let bit_index = get_operand_from_opcode(opcode, OperandType::BitIndex, false, true);
            let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, true);

            let r8_operand = R8Operand::from_byte(operand);

            match r8_operand {
                R8Operand::HLInd => (Instruction::BITHLInd(bit_index), 2),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::BITR(bit_index, register), 2)
                }
            }
        }
        2 => {
            let bit_index = get_operand_from_opcode(opcode, OperandType::BitIndex, false, true);
            let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, true);

            let r8_operand = R8Operand::from_byte(operand);

            match r8_operand {
                R8Operand::HLInd => (Instruction::RESETHLInd(bit_index), 2),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::RESETR(bit_index, register), 2)
                }
            }
        }
        3 => {
            let bit_index = get_operand_from_opcode(opcode, OperandType::BitIndex, false, true);
            let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, true);

            let r8_operand = R8Operand::from_byte(operand);

            match r8_operand {
                R8Operand::HLInd => (Instruction::SETHLInd(bit_index), 2),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::SETR(bit_index, register), 2)
                }
            }
        }
        _ => panic!("Unknown instruction: 0xCB {}", opcode),
    }
}

fn decode_generic(opcode: u8, imm_8: u8, imm_16: u16) -> (Instruction, u16) {
    match (opcode & 0b11000000) >> 6 {
        0 => decode_generic_block_0(opcode, imm_8, imm_16),
        1 => decode_generic_block_1(opcode, imm_8, imm_16),
        2 => decode_generic_block_2(opcode, imm_8, imm_16),
        3 => decode_generic_block_3(opcode, imm_8, imm_16),
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
            let operand = get_operand_from_opcode(opcode, OperandType::FlowCondition, false, false);
            let cond_operand = FlowCondition::from_byte(operand);
            (Instruction::JRCC(cond_operand, imm_8 as i8), 2)
        }
        opcode if (opcode & 0b1100_0111) == 0b0000_0110 => {
            let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, false);
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
            let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, false);
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
            let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, false);
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
            let operand = get_operand_from_opcode(opcode, OperandType::R16Operand, false, false);
            let r16_operand = R16Operand::from_byte(operand);
            let register_16 = Register16::from_r16_operand(r16_operand);
            (Instruction::INC16(register_16), 1)
        }
        opcode if (opcode & 0b1100_1111) == 0b0000_1011 => {
            let operand = get_operand_from_opcode(opcode, OperandType::R16Operand, false, false);
            let r16_operand = R16Operand::from_byte(operand);
            let register_16 = Register16::from_r16_operand(r16_operand);
            (Instruction::DEC16(register_16), 1)
        }
        opcode if (opcode & 0b1100_1111) == 0b0000_1001 => {
            let operand = get_operand_from_opcode(opcode, OperandType::R16Operand, false, false);
            let r16_operand = R16Operand::from_byte(operand);
            let register_16 = Register16::from_r16_operand(r16_operand);
            (Instruction::ADDHL(register_16), 1)
        }
        opcode if (opcode & 0b1100_1111) == 0b0000_0001 => {
            let operand = get_operand_from_opcode(opcode, OperandType::R16Operand, false, false);
            let r16_operand = R16Operand::from_byte(operand);
            let register_16 = Register16::from_r16_operand(r16_operand);
            (Instruction::LDImm16(register_16, imm_16), 3)
        }
        opcode if (opcode & 0b1100_1111) == 0b0000_0010 => {
            let operand = get_operand_from_opcode(opcode, OperandType::R16MemOperand, false, false);
            let r16mem_operand = R16MemOperand::from_byte(operand);

            match r16mem_operand {
                R16MemOperand::BC => (Instruction::LDToBCIndFromA(), 1),
                R16MemOperand::DE => (Instruction::LDToDEIndFromA(), 1),
                R16MemOperand::HLI => (Instruction::LDToHLIndIncFromA(), 1),
                R16MemOperand::HLD => (Instruction::LDToHLIndDecFromA(), 1),
            }
        }
        opcode if (opcode & 0b1100_1111) == 0b0000_1010 => {
            let operand = get_operand_from_opcode(opcode, OperandType::R16MemOperand, false, false);
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
    match opcode {
        0b0111_0110 => (Instruction::HALT(), 1),
        opcode if (opcode & 0b1100_0000) == 0b0100_0000 => {
            let operand_dest = get_operand_from_opcode(opcode, OperandType::R8Operand, true, false);
            let operand_source =
                get_operand_from_opcode(opcode, OperandType::R8Operand, false, false);

            let r8_operand_dest = R8Operand::from_byte(operand_dest);
            let r8_operand_source = R8Operand::from_byte(operand_source);

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
    match (opcode & 0b0111_1000) >> 3 {
        0 => {
            let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, false);
            let r8_operand = R8Operand::from_byte(operand);

            match r8_operand {
                R8Operand::HLInd => (Instruction::ADDHLInd(), 1),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::ADD(register), 1)
                }
            }
        }
        1 => {
            let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, false);
            let r8_operand = R8Operand::from_byte(operand);

            match r8_operand {
                R8Operand::HLInd => (Instruction::ADDCHLInd(), 1),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::ADDC(register), 1)
                }
            }
        }
        2 => {
            let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, false);
            let r8_operand = R8Operand::from_byte(operand);

            match r8_operand {
                R8Operand::HLInd => (Instruction::SUBHLInd(), 1),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::SUB(register), 1)
                }
            }
        }
        3 => {
            let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, false);
            let r8_operand = R8Operand::from_byte(operand);

            match r8_operand {
                R8Operand::HLInd => (Instruction::SUBCHLInd(), 1),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::SUBC(register), 1)
                }
            }
        }
        4 => {
            let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, false);
            let r8_operand = R8Operand::from_byte(operand);

            match r8_operand {
                R8Operand::HLInd => (Instruction::ANDHLInd(), 1),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::AND(register), 1)
                }
            }
        }
        5 => {
            let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, false);
            let r8_operand = R8Operand::from_byte(operand);

            match r8_operand {
                R8Operand::HLInd => (Instruction::XORHLInd(), 1),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::XOR(register), 1)
                }
            }
        }
        6 => {
            let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, false);
            let r8_operand = R8Operand::from_byte(operand);

            match r8_operand {
                R8Operand::HLInd => (Instruction::ORHLInd(), 1),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::OR(register), 1)
                }
            }
        }
        7 => {
            let operand = get_operand_from_opcode(opcode, OperandType::R8Operand, false, false);
            let r8_operand = R8Operand::from_byte(operand);

            match r8_operand {
                R8Operand::HLInd => (Instruction::CPHLInd(), 1),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    (Instruction::CP(register), 1)
                }
            }
        }
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
            let operand = get_operand_from_opcode(opcode, OperandType::FlowCondition, false, false);
            let cond = FlowCondition::from_byte(operand);

            (Instruction::RETCC(cond), 1)
        }
        opcode if (opcode & 0b1110_0111) == 0b1100_0010 => {
            let operand = get_operand_from_opcode(opcode, OperandType::FlowCondition, false, false);
            let cond = FlowCondition::from_byte(operand);

            (Instruction::JPCC(cond, imm_16), 3)
        }
        opcode if (opcode & 0b1110_0111) == 0b1100_0100 => {
            let operand = get_operand_from_opcode(opcode, OperandType::FlowCondition, false, false);
            let cond = FlowCondition::from_byte(operand);

            (Instruction::CALLCC(cond, imm_16), 3)
        }
        opcode if (opcode & 0b1100_0111) == 0b1100_0111 => {
            let operand = get_operand_from_opcode(opcode, OperandType::ResetTarget, false, false);

            (Instruction::RST(operand), 1)
        }
        // Table 3
        opcode if (opcode & 0b1100_1111) == 0b1100_0001 => {
            let operand = get_operand_from_opcode(opcode, OperandType::R16StkOperand, false, false);
            let r16stk_operand = R16StkOperand::from_byte(operand);
            let register_16 = Register16::from_r16stk_operand(r16stk_operand);

            (Instruction::POP(register_16), 1)
        }
        opcode if (opcode & 0b1100_1111) == 0b1100_0101 => {
            let operand = get_operand_from_opcode(opcode, OperandType::R16StkOperand, false, false);
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

#[cfg(test)]
mod tests {
    use crate::console::cpu::{decoder::*, register};

    fn encode_cb_table_1_inst(instruction: Instruction) -> u8 {
        use Instruction::*;
        let hl_ind_r8_op_byte = R8Operand::HLInd.to_byte();

        match instruction {
            RLCR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                (0b0000_0000 | operand)
            }
            RRCR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                (0b0000_1000 | operand)
            }
            RLR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                (0b0001_0000 | operand)
            }
            RRR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                (0b0001_1000 | operand)
            }
            SLAR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                (0b0010_0000 | operand)
            }
            SRAR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                (0b0010_1000 | operand)
            }
            SWAPR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                (0b0011_0000 | operand)
            }
            SRLR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                (0b0011_1000 | operand)
            }
            RLCHLInd() => (0b0000_0000 | hl_ind_r8_op_byte),
            RRCHLInd() => (0b0000_1000 | hl_ind_r8_op_byte),
            RLHLInd() => (0b0001_0000 | hl_ind_r8_op_byte),
            RRHLInd() => (0b0001_1000 | hl_ind_r8_op_byte),
            SLAHLInd() => (0b0010_0000 | hl_ind_r8_op_byte),
            SRAHLInd() => (0b0010_1000 | hl_ind_r8_op_byte),
            SWAPHLInd() => (0b0011_0000 | hl_ind_r8_op_byte),
            SRLHLInd() => (0b0011_1000 | hl_ind_r8_op_byte),
            _ => panic!("Not CB Table 1 instruction"),
        }
    }

    fn encode_cb_table_2_inst(instruction: Instruction) -> u8 {
        use Instruction::*;
        let hl_ind_r8_op_byte = R8Operand::HLInd.to_byte();

        match instruction {
            BITR(bit_index, register) => {
                let operand = R8Operand::to_byte_from_register(register);
                let adjusted_bit_index = bit_index << 3;
                (0b0100_0000 | adjusted_bit_index | operand)
            }
            RESETR(bit_index, register) => {
                let operand = R8Operand::to_byte_from_register(register);
                let adjusted_bit_index = bit_index << 3;
                (0b1000_0000 | adjusted_bit_index | operand)
            }
            SETR(bit_index, register) => {
                let operand = R8Operand::to_byte_from_register(register);
                let adjusted_bit_index = bit_index << 3;
                (0b1100_0000 | adjusted_bit_index | operand)
            }
            BITHLInd(bit_index) => {
                let adjusted_bit_index = bit_index << 3;
                (0b0100_0000 | adjusted_bit_index | hl_ind_r8_op_byte)
            }
            RESETHLInd(bit_index) => {
                let adjusted_bit_index = bit_index << 3;
                (0b1000_0000 | adjusted_bit_index | hl_ind_r8_op_byte)
            }
            SETHLInd(bit_index) => {
                let adjusted_bit_index = bit_index << 3;
                (0b1100_0000 | adjusted_bit_index | hl_ind_r8_op_byte)
            }
            _ => panic!("Not CB Table 2 instruction"),
        }
    }

    fn encode_block_2_inst(instruction: Instruction) -> u8 {
        use Instruction::*;
        let hl_ind_r8_op_byte = R8Operand::HLInd.to_byte();

        match instruction {
            ADD(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                (0b1000_0000 | operand)
            }
            ADDC(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                (0b1000_1000 | operand)
            }
            SUB(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                (0b1001_0000 | operand)
            }
            SUBC(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                (0b1001_1000 | operand)
            }
            AND(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                (0b1010_0000 | operand)
            }
            XOR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                (0b1010_1000 | operand)
            }
            OR(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                (0b1011_0000 | operand)
            }
            CP(register) => {
                let operand = R8Operand::to_byte_from_register(register);
                (0b1011_1000 | operand)
            }
            ADDHLInd() => (0b1000_0000 | hl_ind_r8_op_byte),
            ADDCHLInd() => (0b1000_1000| hl_ind_r8_op_byte),
            SUBHLInd() => (0b1001_0000 | hl_ind_r8_op_byte),
            SUBCHLInd() => (0b1001_1000 | hl_ind_r8_op_byte),
            ANDHLInd() => (0b1010_0000 | hl_ind_r8_op_byte),
            XORHLInd() => (0b1010_1000 | hl_ind_r8_op_byte),
            ORHLInd() => (0b1011_0000 | hl_ind_r8_op_byte),
            CPHLInd() => (0b1011_1000 | hl_ind_r8_op_byte),
            _ => panic!("Not Block 2 instruction"),
        }
    }

    fn assert_decode(
        expected_instr: Instruction,
        expected_size: u16,
        byte1: u8,
        byte2: u8,
        byte3: u8,
    ) {
        let (decoded_instr, decoded_size) = decode(byte1, byte2, byte3);
        assert_eq!(
            decoded_instr, expected_instr,
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
    fn test_decode_cb_table_1() {
        use Instruction::*;
        let expected_size = 2;
        let byte1 = 0xCB;

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
                let instruction_byte = encode_cb_table_1_inst(instruction.clone());
                assert_decode(instruction, expected_size, byte1, instruction_byte, 0);
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
            let instruction_byte = encode_cb_table_1_inst(instruction.clone());
            assert_decode(instruction, expected_size, byte1, instruction_byte, 0);
        }
    }

    #[test]
    fn test_decode_cb_table_2() {
        use Instruction::*;
        let expected_size = 2;
        let byte1 = 0xCB;

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
                    let instruction_byte = encode_cb_table_2_inst(instruction.clone());
                    assert_decode(instruction, expected_size, byte1, instruction_byte, 0);
                }
            }

            let instructions = [
                BITHLInd(bit_index),
                SETHLInd(bit_index),
                RESETHLInd(bit_index),
            ];

            for instruction in instructions {
                let instruction_byte = encode_cb_table_2_inst(instruction.clone());
                assert_decode(instruction, expected_size, byte1, instruction_byte, 0);
            }
        }
    }

    #[test]
    fn test_decode_block_2() {
        use Instruction::*;
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
                let instruction_byte = encode_block_2_inst(instruction.clone());
                assert_decode(instruction, expected_size, instruction_byte, 0, 0);
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
            let instruction_byte = encode_block_2_inst(instruction.clone());
            assert_decode(instruction, expected_size, instruction_byte, 0, 0);
        }
    }

}
