use std::{clone, f32::INFINITY};

use super::{memory, Memory};

pub struct Cpu<'a> {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
    memory: &'a mut Memory,
}

#[derive(Clone)]
enum Register {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
}

enum Register16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC
}

impl<'a> Cpu<'a> {
    const F_ZERO_FLAG_POS: u8 = 7;
    const F_SUB_FLAG_POS: u8 = 6;
    const F_HALF_CARRY_FLAG_POS: u8 = 5;
    const F_CARRY_FLAG_POS: u8 = 4;

    fn modify_bit(value: u8, bit_position: u8, on: bool) -> u8 {
        if on {
            value | (1 << bit_position)
        } else {
            value & !(1 << bit_position)
        }
    }

    fn set_register(&mut self, register: Register, value: u8) {
        match register {
            Register::A => self.a = value,
            Register::B => self.b = value,
            Register::C => self.c = value,
            Register::D => self.d = value,
            Register::E => self.e = value,
            Register::F => self.f = value & 0xf0,
            Register::H => self.h = value,
            Register::L => self.l = value,
            _ => panic!("Uknown register"),
        };
    }

    fn set_register_16(&mut self, register: Register16, value: u16) {
        match register {
            Register16::AF => self.set_af(value)
            Register16::BC => self.set_bc(value),
            Register16::DE => self.set_de(value),
            Register16::HL => self.set_hl(value),
            Register16::SP => self.sp = value,
            Register16::PC => self.pc = value,
            _ => panic!("Uknown register"),
        };
    }

    fn get_register(&self, register: Register) -> u8 {
        match register {
            Register::A => self.a,
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::F => self.f & 0xf0,
            Register::H => self.h,
            Register::L => self.l,
            _ => panic!("Uknown register"),
        }
    }

    fn get_register_16(&self, register: Register16) -> u16 {
        match register {
            Register16::AF => self.get_af(),
            Register16::BC => self.get_bc(),
            Register16::DE => self.get_de(),
            Register16::SP => self.sp,
            Register16::PC => self.pc,
            _ => panic!("Uknown register"),
        }
    }

    fn set_register_bit(&mut self, register: Register, bit_position: u8, on: bool) {
        let register_value = self.get_register(register.clone());
        let new_register_value = Cpu::modify_bit(register_value, bit_position, on);
        self.set_register(register, new_register_value);
    }

    fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xff00) >> 8) as u8;
        self.c = (value & 0xff) as u8;
    }

    fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | ((self.f & 0xf0) as u16)
    }

    fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xff00) >> 8) as u8;
        self.f = (value & 0xf0) as u8;
    }

    fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xff00) >> 8) as u8;
        self.l = (value & 0xff) as u8;
    }

    fn get_de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xff00) >> 8) as u8;
        self.e = (value & 0xff) as u8;
    }

    fn add(&mut self, register: Register) {
        let register_value = self.get_register(register);

        let (new_register_value, did_overflow) = self.a.overflowing_add(register_value);
        let half_carry = (self.a & 0x0F) + (register_value & 0x0F) > 0x0F;


        self.set_register_bit(Register::F, Cpu::F_CARRY_FLAG_POS, did_overflow);
        self.set_register_bit(Register::F, Cpu::F_ZERO_FLAG_POS, new_register_value == 0);
        self.set_register_bit(Register::F, Cpu::F_SUB_FLAG_POS, false);
        self.set_register_bit(Register::F, Cpu::F_HALF_CARRY_FLAG_POS, half_carry);
                

        self.a = new_register_value;
    }

    fn ld(&mut self, register_to: Register, register_from: Register) {
        let register_from_value = self.get_register(register_from);
        self.set_register(register_to, register_from_value);
    }
    
    fn ld_imm(&mut self, register_to: Register, imm_value: u8) {
        self.set_register(register_to, imm_value);
    }

    fn ld_from_hl_ind(&mut self, register_to: Register) {
        let memory_value = self.memory.read_from_8b(self.get_hl() as usize);
        self.set_register(register_to, memory_value);
    }

    fn ld_to_hl_ind(&mut self, register_from: Register) {
        let register_from_value = self.get_register(register_from);
        self.memory.write_to_8b(self.get_hl() as usize, register_from_value);
    }

    fn ld_to_hl_ind_imm(&mut self, value: u8) {
        self.memory.write_to_8b(self.get_hl() as usize, value);
    }


    fn ld_from_bc_to_a_ind(&mut self) {
        let memory_value = self.memory.read_from_8b(self.get_bc() as usize);
        self.a = memory_value;
    }

    fn ld_from_de_to_a_ind(&mut self) {
        let memory_value = self.memory.read_from_8b(self.get_de() as usize);
        self.a = memory_value;
    }

    fn ld_to_bc_ind_from_a(&mut self) {
        self.memory.write_to_8b(self.get_bc() as usize, self.a);
    }

    fn ld_to_de_ind_from_a(&mut self) {
        self.memory.write_to_8b(self.get_de() as usize, self.a);
    }

    fn ld_from_imm_to_a_16(&mut self, imm: u16) {
        self.a = self.memory.read_from_8b(imm as usize);
    }

    fn ld_to_imm_from_a_16(&mut self, imm: u16) {
        self.memory.write_to_8b(imm as usize, self.a);
    }

    fn ld_to_a_from_c_ind(&mut self) {
        let addr: u16 = 0xFF00 | (self.c as u16);
        self.a = self.memory.read_from_8b(addr as usize);
    }

    fn ld_from_a_to_c_ind(&mut self) {
        let addr: u16 = 0xFF00 | (self.c as u16);
        self.memory.write_to_8b(addr as usize, self.a);
    }

    fn ld_from_imm_to_a_8(&mut self, imm: u8) {
        let addr: u16 = 0xFF00 | (imm as u16);
        self.a = self.memory.read_from_8b(addr as usize);
    }

    fn ld_to_imm_from_a_8(&mut self, imm: u8) {
        let addr: u16 = 0xFF00 | (imm as u16);
        self.memory.write_to_8b(addr as usize, self.a);
    }

    fn ld_to_a_from_hl_ind_dec(&mut self) {
        let mut hl: u16 = self.get_hl();
        self.a = self.memory.read_from_8b(hl as usize);
        hl -= 1;
        self.set_hl(hl);
    }

    fn ld_to_hl_ind_dec_from_a(&mut self) {
        let mut hl: u16 = self.get_hl();
        self.memory.write_to_8b(hl as usize, self.a);
        hl -= 1;
        self.set_hl(hl);
    }

    fn ld_to_a_from_hl_ind_inc(&mut self) {
        let mut hl: u16 = self.get_hl();
        self.a = self.memory.read_from_8b(hl as usize);
        hl += 1;
        self.set_hl(hl);
    }

    fn ld_to_hl_ind_inc_from_a(&mut self) {
        let mut hl: u16 = self.get_hl();
        self.memory.write_to_8b(hl as usize, self.a);
        hl += 1;
        self.set_hl(hl);
    }

    fn ld_imm_16(&mut self, register_to: Register16, imm_value: u16) {
        self.set_register_16(register_to, imm_value);
    }

    fn ld_to_imm_from_sp(&mut self, imm_value: u16) {
        self.memory.write_to_16b(imm_value as usize, self.sp);
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::ADD(register) => self.add(register),
            Instruction::LD(register_to,register_from ) => self.ld(register_to, register_from),
            Instruction::LdImm(register_to,imm ) => self.ld_imm(register_to, imm),
            Instruction::LdFromHLInd(register_to ) => self.ld_from_hl_ind(register_to),
            Instruction::LdToHLInd(register_from) => self.ld_to_hl_ind(register_from),
            Instruction::LdToHlIndImm(imm) => self.ld_to_hl_ind_imm(imm),
            Instruction::LdFromBCToAInd() => self.ld_from_bc_to_a_ind(),
            Instruction::LdFromDEToAInd() => self.ld_from_de_to_a_ind(),
            Instruction::LdToBCIndFromA() => self.ld_to_bc_ind_from_a(),
            Instruction::LdToDEIndFromA() => self.ld_to_de_ind_from_a(),
            Instruction::LdFromImmToA16(imm) => self.ld_from_imm_to_a_16(imm),
            Instruction::LdToImmFromA16(imm) => self.ld_to_imm_from_a_16(imm),
            Instruction::LdToAFromCInd() => self.ld_to_a_from_c_ind(),
            Instruction::LdFromAToCInd() => self.ld_from_a_to_c_ind(),
            Instruction::LdFromImmToA8(imm) => self.ld_from_imm_to_a_8(imm),
            Instruction::LdToImmFromA8(imm) => self.ld_to_imm_from_a_8(imm),
            Instruction::LdToAFromHLIndDec() => self.ld_to_a_from_hl_ind_dec(),
            Instruction::LdToHLIndDecFromA() => self.ld_to_hl_ind_dec_from_a(),
            Instruction::LdToAFromHLIndInc() => self.ld_to_a_from_hl_ind_inc(),
            Instruction::LdToHLIndIncFromA() => self.ld_to_hl_ind_inc_from_a(),
            Instruction::LdImm16(register_to, imm) => self.ld_imm_16(register_to, imm),
            Instruction::LdToImmFromSP(imm) => self.ld_to_imm_from_sp(imm),
            _ => panic!("Uknown instruction"),
        }
    }
}

enum Instruction {
    ADD(Register),
    LD(Register, Register),
    LdImm(Register, u8),
    LdFromHLInd(Register),
    LdToHLInd(Register),
    LdToHlIndImm(u8),
    LdFromBCToAInd(),
    LdFromDEToAInd(),
    LdToBCIndFromA(),
    LdToDEIndFromA(),
    LdFromImmToA16(u16),
    LdToImmFromA16(u16),
    LdToAFromCInd(),
    LdFromAToCInd(),
    LdFromImmToA8(u8),
    LdToImmFromA8(u8),
    LdToAFromHLIndDec(),
    LdToHLIndDecFromA(),
    LdToAFromHLIndInc(),
    LdToHLIndIncFromA(),
    LdImm16(Register16, u16),
    LdToImmFromSP(u16),
}