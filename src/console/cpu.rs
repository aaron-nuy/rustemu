use std::{clone};

use super::{ Memory };

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
    interrupts_enabled: bool,
    halted: bool,
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

#[derive(Clone)]
enum Register16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

enum FlowCondition {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always
}

impl<'a> Cpu<'a> {
    const F_ZERO_FLAG_POS: u8 = 7;
    const F_SUB_FLAG_POS: u8 = 6;
    const F_HALF_CARRY_FLAG_POS: u8 = 5;
    const F_CARRY_FLAG_POS: u8 = 4;

    // Utility

    fn get_bit(value: u8, bit_position: u8) -> bool {
        (value & (0b1 << bit_position)) != 0x0
    }

    fn modify_bit(value: u8, bit_position: u8, on: bool) -> u8 {
        if on { value | (1 << bit_position) } else { value & !(1 << bit_position) }
    }

    fn set_register(&mut self, register: Register, value: u8) {
        match register {
            Register::A => {
                self.a = value;
            }

            Register::B => {
                self.b = value;
            }

            Register::C => {
                self.c = value;
            }

            Register::D => {
                self.d = value;
            }

            Register::E => {
                self.e = value;
            }

            Register::F => {
                self.f = value & 0xf0;
            }

            Register::H => {
                self.h = value;
            }

            Register::L => {
                self.l = value;
            }
            _ => panic!("Uknown register"),
        }
    }

    fn set_register_16(&mut self, register: Register16, value: u16) {
        match register {
            Register16::AF => self.set_af(value),
            Register16::BC => self.set_bc(value),
            Register16::DE => self.set_de(value),
            Register16::HL => self.set_hl(value),
            Register16::SP => {
                self.sp = value;
            }
            Register16::PC => {
                self.pc = value;
            }
            _ => panic!("Uknown register"),
        }
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
            Register16::HL => self.get_hl(),
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

    fn get_register_bit(&self, register: Register, bit_position: u8) -> bool {
        let register_value = self.get_register(register.clone());
        Cpu::get_bit(register_value, bit_position)
    }

    fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xff00) >> 8) as u8;
        self.c = (value & 0x00ff) as u8;
    }

    fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | ((self.f & 0xf0) as u16)
    }

    fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xff00) >> 8) as u8;
        self.f = (value & 0x00f0) as u8;
    }

    fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xff00) >> 8) as u8;
        self.l = (value & 0x00ff) as u8;
    }

    fn get_de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xff00) >> 8) as u8;
        self.e = (value & 0x00ff) as u8;
    }

    fn set_f_flags(&mut self, carry: bool, half_carry: bool, sub: bool, zero: bool) {
        self.set_register_bit(Register::F, Cpu::F_CARRY_FLAG_POS, carry);
        self.set_register_bit(Register::F, Cpu::F_ZERO_FLAG_POS, zero);
        self.set_register_bit(Register::F, Cpu::F_SUB_FLAG_POS, sub);
        self.set_register_bit(Register::F, Cpu::F_HALF_CARRY_FLAG_POS, half_carry);
    }

    fn evaluate_flow_condition(&self, condtion: FlowCondition) -> bool {
        match condtion {
            FlowCondition::NotZero => !Self::get_bit(self.f, Cpu::F_ZERO_FLAG_POS),
            FlowCondition::Zero => Self::get_bit(self.f, Cpu::F_ZERO_FLAG_POS),
            FlowCondition::NotCarry => !Self::get_bit(self.f, Cpu::F_CARRY_FLAG_POS),
            FlowCondition::Carry => Self::get_bit(self.f, Cpu::F_CARRY_FLAG_POS),
            FlowCondition::Always => true,
            _ => panic!("Unknown condtion")
        }
    }

    fn push_to_stack_16b(&mut self, value: u16) {
        let addr = self.sp.wrapping_sub(2);
        self.memory.write_to_16b(addr, value);
        self.sp = addr;
    }

    fn pop_from_stack_16b(&mut self) -> u16 {
        let stack_value = self.memory.read_from_16b(self.sp);
        self.sp = self.sp.wrapping_add(2);
        stack_value
    }

    // Instructions

    fn ld(&mut self, register_to: Register, register_from: Register) {
        let register_from_value = self.get_register(register_from);
        self.set_register(register_to, register_from_value);
    }

    fn ld_imm(&mut self, register_to: Register, imm_value: u8) {
        self.set_register(register_to, imm_value);
    }

    fn ld_from_hl_ind(&mut self, register_to: Register) {
        let memory_value = self.memory.read_from_8b(self.get_hl());
        self.set_register(register_to, memory_value);
    }

    fn ld_to_hl_ind(&mut self, register_from: Register) {
        let register_from_value = self.get_register(register_from);
        self.memory.write_to_8b(self.get_hl(), register_from_value);
    }

    fn ld_to_hl_ind_imm(&mut self, value: u8) {
        self.memory.write_to_8b(self.get_hl(), value);
    }

    fn ld_from_bc_to_a_ind(&mut self) {
        let memory_value = self.memory.read_from_8b(self.get_bc());
        self.a = memory_value;
    }

    fn ld_from_de_to_a_ind(&mut self) {
        let memory_value = self.memory.read_from_8b(self.get_de());
        self.a = memory_value;
    }

    fn ld_to_bc_ind_from_a(&mut self) {
        self.memory.write_to_8b(self.get_bc(), self.a);
    }

    fn ld_to_de_ind_from_a(&mut self) {
        self.memory.write_to_8b(self.get_de(), self.a);
    }

    fn ld_from_imm_to_a_16(&mut self, imm: u16) {
        self.a = self.memory.read_from_8b(imm);
    }

    fn ld_to_imm_from_a_16(&mut self, imm: u16) {
        self.memory.write_to_8b(imm, self.a);
    }

    fn ld_to_a_from_c_ind(&mut self) {
        let addr: u16 = 0xff00 | (self.c as u16);
        self.a = self.memory.read_from_8b(addr);
    }

    fn ld_from_a_to_c_ind(&mut self) {
        let addr: u16 = 0xff00 | (self.c as u16);
        self.memory.write_to_8b(addr, self.a);
    }

    fn ld_from_imm_to_a_8(&mut self, imm: u8) {
        let addr: u16 = 0xff00 | (imm as u16);
        self.a = self.memory.read_from_8b(addr);
    }

    fn ld_to_imm_from_a_8(&mut self, imm: u8) {
        let addr: u16 = 0xff00 | (imm as u16);
        self.memory.write_to_8b(addr, self.a);
    }

    fn ld_to_a_from_hl_ind_dec(&mut self) {
        let mut hl: u16 = self.get_hl();
        self.a = self.memory.read_from_8b(hl);
        hl = hl.wrapping_sub(1);
        self.set_hl(hl);
    }

    fn ld_to_hl_ind_dec_from_a(&mut self) {
        let mut hl: u16 = self.get_hl();
        self.memory.write_to_8b(hl, self.a);
        hl = hl.wrapping_sub(1);
        self.set_hl(hl);
    }

    fn ld_to_a_from_hl_ind_inc(&mut self) {
        let mut hl: u16 = self.get_hl();
        self.a = self.memory.read_from_8b(hl);
        hl = hl.wrapping_add(1);
        self.set_hl(hl);
    }

    fn ld_to_hl_ind_inc_from_a(&mut self) {
        let mut hl: u16 = self.get_hl();
        self.memory.write_to_8b(hl, self.a);
        hl = hl.wrapping_add(1);
        self.set_hl(hl);
    }

    fn ld_imm_16(&mut self, register_to: Register16, imm_value: u16) {
        self.set_register_16(register_to, imm_value);
    }

    fn ld_to_imm_from_sp(&mut self, imm_value: u16) {
        self.memory.write_to_16b(imm_value, self.sp);
    }

    fn ld_sp_from_hl(&mut self) {
        self.sp = self.get_hl();
    }

    fn push(&mut self, register_from: Register16) {
        let register_value = self.get_register_16(register_from);
        self.push_to_stack_16b(register_value);
    }

    fn pop(&mut self, register: Register16) {
        let stack_value = self.pop_from_stack_16b();
        self.set_register_16(register, stack_value);
    }

    fn ld_hl_from_adjusted_sp(&mut self, imm: i8) {
        let adjusted_sp = self.sp.wrapping_add_signed(imm as i8 as i16);
        self.set_hl(adjusted_sp);

        let sp = self.sp;

        let half_carry = (sp & 0x000f).wrapping_add(((imm as u8) & 0x0f) as u16) > 0x000f;
        let did_overflow = (sp & 0x00ff).wrapping_add(imm as u8 as u16) > 0x00ff;

        self.set_f_flags(did_overflow, half_carry, false, false);
    }

    fn _add(&mut self, value: u8) {
        let (new_value, did_overflow) = self.a.overflowing_add(value);
        let half_carry = (self.a & 0x0f) + (value & 0x0f) > 0x0f;

        self.set_f_flags(did_overflow, half_carry, false, new_value == 0);

        self.a = new_value;
    }

    fn _adc(&mut self, value: u8) {
        let carry_flag = if Self::get_bit(self.f, Cpu::F_CARRY_FLAG_POS) { 1 } else { 0 };

        let (first_add, did_overflow1) = self.a.overflowing_add(value);
        let (second_add, did_overflow2) = first_add.overflowing_add(carry_flag);

        let half_carry = (self.a & 0x0f) + (value & 0x0f) + carry_flag > 0x0f;
        let did_overflow = did_overflow1 || did_overflow2;

        self.set_f_flags(did_overflow, half_carry, false, second_add == 0);

        self.a = second_add;
    }

    fn add(&mut self, register: Register) {
        let value = self.get_register(register);
        self._add(value);
    }

    fn add_hl_ind(&mut self) {
        let value = self.memory.read_from_8b(self.get_hl());
        self._add(value);
    }

    fn add_imm(&mut self, imm: u8) {
        self._add(imm);
    }

    fn add_c(&mut self, register: Register) {
        let value = self.get_register(register);
        self._adc(value);
    }

    fn add_c_hl_ind(&mut self) {
        let value = self.memory.read_from_8b(self.get_hl());
        self._adc(value);
    }

    fn add_c_imm(&mut self, imm: u8) {
        self._adc(imm);
    }

    fn _sub(&mut self, value: u8) {
        let (new_register_value, did_borrow) = self.a.overflowing_sub(value);
        let half_carry = self.a & 0x0f < value & 0x0f;

        self.set_f_flags(did_borrow, half_carry, true, new_register_value == 0);

        self.a = new_register_value;
    }

    fn _sbc(&mut self, value: u8) {
        let carry_flag = if Cpu::get_bit(self.f, Cpu::F_CARRY_FLAG_POS) { 1 } else { 0 };

        let (fist_sub, did_borrow1) = self.a.overflowing_sub(value);
        let (second_sub, did_borrow2) = fist_sub.overflowing_sub(carry_flag);

        let half_carry = self.a & 0x0f < (value & 0x0f) + carry_flag;
        let did_borrow = did_borrow1 || did_borrow2;

        self.set_f_flags(did_borrow, half_carry, true, second_sub == 0);

        self.a = second_sub;
    }

    fn sub(&mut self, register: Register) {
        let value = self.get_register(register);
        self._sub(value);
    }

    fn sub_hl_ind(&mut self) {
        let value = self.memory.read_from_8b(self.get_hl());
        self._sub(value);
    }

    fn sub_imm(&mut self, imm: u8) {
        self._sub(imm);
    }

    fn sub_c(&mut self, register: Register) {
        let value = self.get_register(register);
        self._sbc(value);
    }

    fn sub_c_hl_ind(&mut self) {
        let value = self.memory.read_from_8b(self.get_hl());
        self._sbc(value);
    }

    fn sub_c_imm(&mut self, imm: u8) {
        self._sbc(imm);
    }

    fn _cp(&mut self, value: u8) {
        let (new_register_value, did_borrow) = self.a.overflowing_sub(value);
        let half_carry = self.a & 0x0f < value & 0x0f;

        self.set_f_flags(did_borrow, half_carry, true, new_register_value == 0);
    }

    fn cp(&mut self, register: Register) {
        let value = self.get_register(register);
        self._cp(value);
    }

    fn cp_hl_ind(&mut self) {
        let value = self.memory.read_from_8b(self.get_hl());
        self._cp(value);
    }

    fn cp_imm(&mut self, imm: u8) {
        self._cp(imm);
    }

    fn inc(&mut self, register: Register) {
        let value = self.get_register(register.clone());

        let (new_value, _) = value.overflowing_add(1);
        let half_carry = (value & 0x0f) + 0b1 > 0x0f;
        let current_carry = self.get_register_bit(Register::F, Cpu::F_CARRY_FLAG_POS);

        self.set_f_flags(current_carry, half_carry, false, new_value == 0);

        self.set_register(register, new_value);
    }

    fn inc_hl_ind(&mut self) {
        let value = self.memory.read_from_8b(self.get_hl());

        let (new_value, _) = value.overflowing_add(1);
        let half_carry = (value & 0x0f) + 0b1 > 0x0f;
        let current_carry = self.get_register_bit(Register::F, Cpu::F_CARRY_FLAG_POS);

        self.set_f_flags(current_carry, half_carry, false, new_value == 0);

        self.memory.write_to_8b(self.get_hl(), new_value);
    }

    fn dec(&mut self, register: Register) {
        let value = self.get_register(register.clone());

        let (new_value, _) = value.overflowing_sub(1);
        let half_carry = (value & 0x0f) == 0;
        let current_carry = self.get_register_bit(Register::F, Cpu::F_CARRY_FLAG_POS);

        self.set_f_flags(current_carry, half_carry, true, new_value == 0);

        self.set_register(register, new_value);
    }

    fn dec_hl_ind(&mut self) {
        let value = self.memory.read_from_8b(self.get_hl());

        let (new_value, _) = value.overflowing_sub(1);
        let half_carry = (value & 0x0f) == 0;
        let current_carry = self.get_register_bit(Register::F, Cpu::F_CARRY_FLAG_POS);

        self.set_f_flags(current_carry, half_carry, true, new_value == 0);

        self.memory.write_to_8b(self.get_hl(), new_value);
    }

    fn _and(&mut self, value: u8) {
        let new_value = self.a & value;

        self.set_f_flags(false, true, false, new_value == 0);

        self.a = new_value;
    }

    fn and(&mut self, register: Register) {
        let value = self.get_register(register);
        self._and(value);
    }

    fn and_hl_ind(&mut self) {
        let value = self.memory.read_from_8b(self.get_hl());
        self._and(value);
    }

    fn and_imm(&mut self, imm: u8) {
        self._and(imm);
    }

    fn _or(&mut self, value: u8) {
        let new_value = self.a | value;

        self.set_f_flags(false, false, false, new_value == 0);

        self.a = new_value;
    }

    fn or(&mut self, register: Register) {
        let value = self.get_register(register);
        self._or(value);
    }

    fn or_hl_ind(&mut self) {
        let value = self.memory.read_from_8b(self.get_hl());
        self._or(value);
    }

    fn or_imm(&mut self, imm: u8) {
        self._or(imm);
    }

    fn _xor(&mut self, value: u8) {
        let new_value = self.a ^ value;

        self.set_f_flags(false, false, false, new_value == 0);

        self.a = new_value;
    }

    fn xor(&mut self, register: Register) {
        let value = self.get_register(register);
        self._xor(value);
    }

    fn xor_hl_ind(&mut self) {
        let value = self.memory.read_from_8b(self.get_hl());
        self._xor(value);
    }

    fn xor_imm(&mut self, imm: u8) {
        self._xor(imm);
    }

    fn ccf(&mut self) {
        let current_carry = self.get_register_bit(Register::F, Cpu::F_CARRY_FLAG_POS);
        let current_zero = self.get_register_bit(Register::F, Cpu::F_ZERO_FLAG_POS);

        self.set_f_flags(!current_carry, false, false, current_zero);
    }

    fn scf(&mut self) {
        let current_zero = self.get_register_bit(Register::F, Cpu::F_ZERO_FLAG_POS);
        self.set_f_flags(true, false, false, current_zero);
    }

    fn daa(&mut self) {
        let current_sub = self.get_register_bit(Register::F, Cpu::F_SUB_FLAG_POS);
        let current_half_carry = self.get_register_bit(Register::F, Cpu::F_HALF_CARRY_FLAG_POS);
        let mut carry = self.get_register_bit(Register::F, Cpu::F_CARRY_FLAG_POS);

        let mut adjust = 0;
        let mut new_value: u8 = 0;
        if !current_sub {
            if current_half_carry || self.a & 0x0f > 9 {
                adjust |= 0x06;
            }
            if carry || self.a > 0x99 {
                adjust |= 0x60;
                carry = true;
            }
            new_value = self.a.wrapping_add(adjust);
        } else {
            if current_half_carry {
                adjust |= 0x06;
            }
            if carry {
                adjust |= 0x60;
            }
            new_value = self.a.wrapping_sub(adjust);
        }

        self.set_f_flags(carry, false, current_sub, new_value == 0);

        self.a = new_value;
    }

    fn cpl(&mut self) {
        let current_carry = self.get_register_bit(Register::F, Cpu::F_CARRY_FLAG_POS);
        let current_zero = self.get_register_bit(Register::F, Cpu::F_ZERO_FLAG_POS);

        self.set_f_flags(current_carry, true, true, current_zero);

        self.a = !self.a;
    }

    fn inc_16(&mut self, register: Register16) {
        let value = self.get_register_16(register.clone());
        self.set_register_16(register, value.wrapping_add(1));
    }

    fn dec_16(&mut self, register: Register16) {
        let value = self.get_register_16(register.clone());
        self.set_register_16(register, value.wrapping_sub(1));
    }

    fn add_hl(&mut self, register: Register16) {
        let register_value = self.get_register_16(register);
        let hl = self.get_hl();

        let (new_value, did_overflow) = hl.overflowing_add(register_value);
        let current_zero = self.get_register_bit(Register::F, Cpu::F_ZERO_FLAG_POS);
        let half_carry = (hl & 0x0fff) + (register_value & 0x0fff) > 0x0fff;

        self.set_f_flags(did_overflow, half_carry, false, current_zero);

        self.set_hl(new_value);
    }

    fn add_spp_imm(&mut self, imm: u8) {
        let new_value = self.sp.wrapping_add_signed(imm as i8 as i16);

        let sp = self.sp;

        let half_carry = (sp & 0x000f).wrapping_add(((imm as u8) & 0x0f) as u16) > 0x000f;
        let did_overflow = (sp & 0x00ff).wrapping_add(imm as u8 as u16) > 0x00ff;

        self.set_f_flags(did_overflow, half_carry, false, false);

        self.sp = new_value;
    }

    fn _rotate(&mut self, value: u8, circular: bool, right: bool, accumulator: bool) -> u8 {
        let carry_bit_pos: u8 = if right { 0x01 } else { 0x80 };
        let old_carry = self.get_register_bit(Register::F, Cpu::F_CARRY_FLAG_POS);
        let carry = (value & carry_bit_pos) != 0;

        let new_value = {
            if circular && !right {
                value.rotate_left(1)
            } else if circular && right {
                value.rotate_right(1)
            } else if !circular && !right {
                (value << 1) | (old_carry as u8)
            } else {
                (value >> 1) | ((old_carry as u8) << 0x07)
            }
        };

        let zero = if accumulator { false } else { new_value == 0 };

        self.set_f_flags(carry, false, false, zero);

        new_value
    }

    fn rlca(&mut self) {
        self.a = self._rotate(self.a, true, false, true);
    }

    fn rrca(&mut self) {
        self.a = self._rotate(self.a, true, true, true);
    }

    fn rra(&mut self) {
        self.a = self._rotate(self.a, false, true, true);
    }

    fn rla(&mut self) {
        self.a = self._rotate(self.a, false, false, true);
    }

    fn rlcr(&mut self, register: Register) {
        let register_value = self.get_register(register.clone());

        let new_value = self._rotate(register_value, true, false, false);

        self.set_register(register, new_value);
    }

    fn rlc_hl_ind(&mut self) {
        let register_value = self.memory.read_from_8b(self.get_hl());

        let new_value = self._rotate(register_value, true, false, false);

        self.memory.write_to_8b(self.get_hl(), new_value);
    }

    fn rrcr(&mut self, register: Register) {
        let register_value = self.get_register(register.clone());

        let new_value = self._rotate(register_value, true, true, false);

        self.set_register(register, new_value);
    }

    fn rrc_hl_ind(&mut self) {
        let register_value = self.memory.read_from_8b(self.get_hl());

        let new_value = self._rotate(register_value, true, true, false);

        self.memory.write_to_8b(self.get_hl(), new_value);
    }

    fn rrr(&mut self, register: Register) {
        let register_value = self.get_register(register.clone());

        let new_value = self._rotate(register_value, false, true, false);

        self.set_register(register, new_value);
    }

    fn rlr(&mut self, register: Register) {
        let register_value = self.get_register(register.clone());

        let new_value = self._rotate(register_value, false, false, false);

        self.set_register(register, new_value);
    }

    fn rr_hl_ind(&mut self) {
        let register_value = self.memory.read_from_8b(self.get_hl());

        let new_value = self._rotate(register_value, false, true, false);

        self.memory.write_to_8b(self.get_hl(), new_value);
    }

    fn rl_hl_ind(&mut self) {
        let register_value = self.memory.read_from_8b(self.get_hl());

        let new_value = self._rotate(register_value, false, false, false);

        self.memory.write_to_8b(self.get_hl(), new_value);
    }

    fn _sa(&mut self, value: u8, right: bool) -> u8 {
        let msb = value & 0x80;
        let lsb = value & 0x01;

        let carry = if right { lsb != 0 } else { msb != 0 };
        let new_value = if right { (value >> 1) | msb } else { value << 1 };

        self.set_f_flags(carry, false, false, new_value == 0);

        new_value
    }

    fn srar(&mut self, register: Register) {
        let register_value = self.get_register(register.clone());

        let new_value = self._sa(register_value, true);

        self.set_register(register, new_value);
    }

    fn sra_hl_ind(&mut self) {
        let register_value = self.memory.read_from_8b(self.get_hl());

        let new_value = self._sa(register_value, true);

        self.memory.write_to_8b(self.get_hl(), new_value);
    }

    fn slar(&mut self, register: Register) {
        let register_value = self.get_register(register.clone());

        let new_value = self._sa(register_value, false);

        self.set_register(register, new_value);
    }

    fn sla_hl_ind(&mut self) {
        let register_value = self.memory.read_from_8b(self.get_hl());

        let new_value = self._sa(register_value, false);

        self.memory.write_to_8b(self.get_hl(), new_value);
    }

    fn _sl(&mut self, value: u8, right: bool) -> u8 {
        let msb = value & 0x80;
        let lsb = value & 0x01;

        let carry = if right { lsb != 0 } else { msb != 0 };
        let new_value = if right { value >> 1 } else { value << 1 };

        self.set_f_flags(carry, false, false, new_value == 0);

        new_value
    }

    fn srlr(&mut self, register: Register) {
        let register_value = self.get_register(register.clone());

        let new_value = self._sl(register_value, true);

        self.set_register(register, new_value);
    }

    fn srl_hl_ind(&mut self) {
        let register_value = self.memory.read_from_8b(self.get_hl());

        let new_value = self._sl(register_value, true);

        self.memory.write_to_8b(self.get_hl(), new_value);
    }

    fn _swap(&mut self, value: u8) -> u8 {
        let new_value = value.rotate_left(4);

        self.set_f_flags(false, false, false, new_value == 0);

        new_value
    }

    fn swapr(&mut self, register: Register) {
        let register_value = self.get_register(register.clone());

        let new_value = self._swap(register_value);

        self.set_register(register, new_value);
    }

    fn swap_hl_ind(&mut self) {
        let register_value = self.memory.read_from_8b(self.get_hl());

        let new_value = self._swap(register_value);

        self.memory.write_to_8b(self.get_hl(), new_value);
    }

    fn _bit(&mut self, bit_position: u8, value: u8) {
        let old_carry = self.get_register_bit(Register::F, Cpu::F_CARRY_FLAG_POS);

        let bit_is_set = value & (1 << bit_position);

        self.set_f_flags(old_carry, true, false, bit_is_set == 0);
    }

    fn bitr(&mut self, bit_position: u8, register: Register) {
        let register_value = self.get_register(register);
        self._bit(bit_position, register_value);
    }

    fn bit_hl_ind(&mut self, bit_position: u8) {
        let register_value = self.memory.read_from_8b(self.get_hl());
        self._bit(bit_position, register_value);
    }

    fn setr(&mut self, bit_position: u8, register: Register) {
        let register_value = self.get_register(register.clone());
        let new_value = Self::modify_bit(register_value, bit_position, true);
        self.set_register(register, new_value);
    }

    fn set_hl_ind(&mut self, bit_position: u8) {
        let register_value = self.memory.read_from_8b(self.get_hl());
        let new_value = Self::modify_bit(register_value, bit_position, true);
        self.memory.write_to_8b(self.get_hl(), new_value);
    }

    fn resetr(&mut self, bit_position: u8, register: Register) {
        let register_value = self.get_register(register.clone());
        let new_value = Self::modify_bit(register_value, bit_position, false);
        self.set_register(register, new_value);
    }

    fn reset_hl_ind(&mut self, bit_position: u8) {
        let register_value = self.memory.read_from_8b(self.get_hl());
        let new_value = Self::modify_bit(register_value, bit_position, false);
        self.memory.write_to_8b(self.get_hl(), new_value);
    }

    fn nop(&self) {}

    fn jp(&mut self, addr: u16) {
        self.pc = addr;
    }

    fn jp_hl(&mut self) {
        self.pc = self.get_hl();
    }

    fn jp_cc(&mut self, cond: FlowCondition, addr: u16) {
        if self.evaluate_flow_condition(cond) {
            self.jp(addr);
        }
    }

    fn jr(&mut self, imm: u8) {
        self.pc = self.pc.wrapping_add_signed(imm as i8 as i16);
    }

    fn jr_cc(&mut self, cond: FlowCondition, imm: u8) {
        if self.evaluate_flow_condition(cond) {
            self.jr(imm);
        }
    }

    fn call(&mut self, addr: u16) {
        self.push_to_stack_16b(self.pc);
        self.pc = addr;
    }

    fn call_cc(&mut self, cond: FlowCondition, addr: u16) {
        if self.evaluate_flow_condition(cond) {
            self.call(addr);
        }
    }

    fn ret(&mut self) {
        self.pc = self.pop_from_stack_16b();
    }

    fn ret_cc(&mut self, cond: FlowCondition) {
        if self.evaluate_flow_condition(cond) {
            self.ret();
        }
    }

    fn ret_i(&mut self) {
        self.ret();
        self.ei();
    }

    fn rst(&mut self, addr: u8) {
        self.push_to_stack_16b(self.pc);
        self.pc = 0x0000 & (addr as u16);
    }

    fn halt(&mut self) {
        self.halted = true;
    }

    fn stop(&mut self) {
        self.halted = true;
    }

    fn di(&mut self) {
        self.interrupts_enabled = false;
    }

    fn ei(&mut self) {
        self.interrupts_enabled = true;
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::LD(register_to, register_from) => self.ld(register_to, register_from),
            Instruction::LDImm(register_to, imm) => self.ld_imm(register_to, imm),
            Instruction::LDFromHLInd(register_to) => self.ld_from_hl_ind(register_to),
            Instruction::LDToHLInd(register_from) => self.ld_to_hl_ind(register_from),
            Instruction::LDToHlIndImm(imm) => self.ld_to_hl_ind_imm(imm),
            Instruction::LDFromBCToAInd() => self.ld_from_bc_to_a_ind(),
            Instruction::LDFromDEToAInd() => self.ld_from_de_to_a_ind(),
            Instruction::LDToBCIndFromA() => self.ld_to_bc_ind_from_a(),
            Instruction::LDToDEIndFromA() => self.ld_to_de_ind_from_a(),
            Instruction::LDFromImmToA16(imm) => self.ld_from_imm_to_a_16(imm),
            Instruction::LDToImmFromA16(imm) => self.ld_to_imm_from_a_16(imm),
            Instruction::LDToAFromCInd() => self.ld_to_a_from_c_ind(),
            Instruction::LDFromAToCInd() => self.ld_from_a_to_c_ind(),
            Instruction::LDFromImmToA8(imm) => self.ld_from_imm_to_a_8(imm),
            Instruction::LDToImmFromA8(imm) => self.ld_to_imm_from_a_8(imm),
            Instruction::LDToAFromHLIndDec() => self.ld_to_a_from_hl_ind_dec(),
            Instruction::LDToHLIndDecFromA() => self.ld_to_hl_ind_dec_from_a(),
            Instruction::LDToAFromHLIndInc() => self.ld_to_a_from_hl_ind_inc(),
            Instruction::LDToHLIndIncFromA() => self.ld_to_hl_ind_inc_from_a(),
            Instruction::LDImm16(register_to, imm) => self.ld_imm_16(register_to, imm),
            Instruction::LDToImmFromSP(imm) => self.ld_to_imm_from_sp(imm),
            Instruction::LDSPFromHL() => self.ld_sp_from_hl(),
            Instruction::PUSH(register_from) => self.push(register_from),
            Instruction::POP(register_from) => self.pop(register_from),
            Instruction::LDHLFromAdjustedSP(imm) => self.ld_hl_from_adjusted_sp(imm),
            Instruction::ADD(register) => self.add(register),
            Instruction::ADDHLInd() => self.add_hl_ind(),
            Instruction::ADDImm(imm) => self.add_imm(imm),
            Instruction::ADDC(register) => self.add_c(register),
            Instruction::ADDCHLInd() => self.add_c_hl_ind(),
            Instruction::ADDCImm(imm) => self.add_c_imm(imm),
            Instruction::SUB(register) => self.sub(register),
            Instruction::SUBHLInd() => self.sub_hl_ind(),
            Instruction::SUBImm(imm) => self.sub_imm(imm),
            Instruction::SUBC(register) => self.sub_c(register),
            Instruction::SUBCHLInd() => self.sub_c_hl_ind(),
            Instruction::SUBCImm(imm) => self.sub_c_imm(imm),
            Instruction::CP(register) => self.cp(register),
            Instruction::CPHLInd() => self.cp_hl_ind(),
            Instruction::CPImm(imm) => self.cp_imm(imm),
            Instruction::INC(register) => self.inc(register),
            Instruction::INCHLInd() => self.inc_hl_ind(),
            Instruction::DEC(register) => self.dec(register),
            Instruction::DECHLInd() => self.dec_hl_ind(),
            Instruction::AND(register) => self.and(register),
            Instruction::ANDHLInd() => self.and_hl_ind(),
            Instruction::ANDImm(imm) => self.and_imm(imm),
            Instruction::OR(register) => self.or(register),
            Instruction::ORHLInd() => self.or_hl_ind(),
            Instruction::ORImm(imm) => self.or_imm(imm),
            Instruction::XOR(register) => self.xor(register),
            Instruction::XORHLInd() => self.xor_hl_ind(),
            Instruction::XORImm(imm) => self.xor_imm(imm),
            Instruction::CCF() => self.ccf(),
            Instruction::SCF() => self.scf(),
            Instruction::DAA() => self.daa(),
            Instruction::CPL() => self.cpl(),
            Instruction::INC16(register) => self.inc_16(register),
            Instruction::DEC16(register) => self.dec_16(register),
            Instruction::ADDHL(register) => self.add_hl(register),
            Instruction::ADDSPImm(imm) => self.add_spp_imm(imm),
            Instruction::RLCA() => self.rlca(),
            Instruction::RRCA() => self.rrca(),
            Instruction::RRA() => self.rra(),
            Instruction::RLA() => self.rla(),
            Instruction::RLCR(register) => self.rlcr(register),
            Instruction::RRCR(register) => self.rrcr(register),
            Instruction::RLCHLInd() => self.rlc_hl_ind(),
            Instruction::RRCHLInd() => self.rrc_hl_ind(),
            Instruction::RLR(register) => self.rlr(register),
            Instruction::RRR(register) => self.rrr(register),
            Instruction::RLHLInd() => self.rl_hl_ind(),
            Instruction::RRHLInd() => self.rr_hl_ind(),
            Instruction::SRAR(register) => self.srar(register),
            Instruction::SRAHLInd() => self.sra_hl_ind(),
            Instruction::SLAR(register) => self.slar(register),
            Instruction::SLAHLInd() => self.sla_hl_ind(),
            Instruction::SRLR(register) => self.srlr(register),
            Instruction::SRLHLInd() => self.srl_hl_ind(),
            Instruction::SWAPR(register) => self.swapr(register),
            Instruction::SWAPHLInd() => self.swap_hl_ind(),
            Instruction::BITR(bit, register) => self.bitr(bit, register),
            Instruction::BITHLInd(bit) => self.bit_hl_ind(bit),
            Instruction::SETR(bit, register) => self.setr(bit, register),
            Instruction::SETHLInd(bit) => self.set_hl_ind(bit),
            Instruction::RESETR(bit, register) => self.resetr(bit, register),
            Instruction::RESETHLInd(bit) => self.reset_hl_ind(bit),
            Instruction::NOP() => self.nop(),
            Instruction::JP(addr) => self.jp(addr),
            Instruction::JPHL() => self.jp_hl(),
            Instruction::JPCC(cond, addr) => self.jp_cc(cond, addr),
            Instruction::JR(imm) => self.jr(imm),
            Instruction::JRCC(cond, imm) => self.jr_cc(cond, imm),
            Instruction::CALL(addr) => self.call(addr),
            Instruction::CALLCC(cond, addr) => self.call_cc(cond, addr),
            Instruction::RET() => self.ret(),
            Instruction::RETCC(cond) => self.ret_cc(cond),
            Instruction::RETI() => self.ret_i(),
            Instruction::RST(addr) => self.rst(addr),
            Instruction::HALT() => self.halt(),
            Instruction::STOP() => self.stop(),
            Instruction::DI() => self.di(),
            Instruction::EI() => self.ei(),
            _ => panic!("Uknown instruction"),
        }
    }
}

enum Instruction {
    LD(Register, Register),
    LDImm(Register, u8),
    LDFromHLInd(Register),
    LDToHLInd(Register),
    LDToHlIndImm(u8),
    LDFromBCToAInd(),
    LDFromDEToAInd(),
    LDToBCIndFromA(),
    LDToDEIndFromA(),
    LDFromImmToA16(u16),
    LDToImmFromA16(u16),
    LDToAFromCInd(),
    LDFromAToCInd(),
    LDFromImmToA8(u8),
    LDToImmFromA8(u8),
    LDToAFromHLIndDec(),
    LDToHLIndDecFromA(),
    LDToAFromHLIndInc(),
    LDToHLIndIncFromA(),
    LDImm16(Register16, u16),
    LDToImmFromSP(u16),
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
    ADDSPImm(u8),
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
    JR(u8),
    JRCC(FlowCondition, u8),
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
