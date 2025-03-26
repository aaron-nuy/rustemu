use std::{clone, f32::INFINITY};

use super::Memory;

pub struct Cpu<'a> {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    memory: &'a mut Memory,
}

enum Instruction {
    ADD(Register),
    LD(Register, Register),
    LdImm(Register, u8),
    LdFromHLInd(Register),
    LdToHLInd(Register),
    LdToHlIndImm(u8),
    LdFromBCToAInd(),
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

impl<'a> Cpu<'a> {
    const F_ZERO_FLAG_MASK: u8 = 0x80;
    const F_SUB_FLAG_MASK: u8 = 0x40;
    const F_HALF_CARRY_FLAG_MASK: u8 = 0x20;
    const F_CARRY_FLAG_MASK: u8 = 0x10;
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
            Register::F => self.f = value,
            Register::H => self.h = value,
            Register::L => self.l = value,
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
            Register::F => self.f,
            Register::H => self.h,
            Register::L => self.l,
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
        ((self.a as u16) << 8) | (self.f as u16)
    }

    fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xff00) >> 8) as u8;
        self.f = (value & 0xff) as u8;
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

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::ADD(register) => self.add(register),
            Instruction::LD(register_to,register_from ) => self.ld(register_to, register_from),
            Instruction::LdImm(register_to,imm ) => self.ld_imm(register_to, imm),
            Instruction::LdFromHLInd(register_to ) => self.ld_from_hl_ind(register_to),
            Instruction::LdToHLInd(register_from) => self.ld_to_hl_ind(register_from),
            Instruction::LdToHlIndImm(imm) => self.ld_to_hl_ind_imm(imm),
            Instruction::LdFromBCToAInd() => self.ld_from_bc_to_a_ind(),
            _ => panic!("Uknown instruction"),
        }
    }
}
