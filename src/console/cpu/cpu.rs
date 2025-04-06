use crate::console::bus::*;
use crate::console::cpu::instruction::*;
use crate::console::utils::bit_utils;

#[derive(Default)]
pub struct Cpu {
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
    clock: u64,
}

impl Cpu {
    pub fn new() -> Self {
        // Cpu {
        //     a: 0x01,
        //     b: 0x00,
        //     c: 0x13,
        //     d: 0x00,
        //     e: 0xD8,
        //     h: 0x01,
        //     l: 0x4D,
        //     f: 0xB0,
        //     pc: 0x100,
        //     sp: 0xFFFE,
        //     clock: 0,
        //     interrupts_enabled: true,
        //     halted: false,
        // }
        Self::default()
    }

    // Utility

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
        }
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

    fn set_flag(&mut self, flag: Flag, on: bool) {
        self.f = if on {
            self.f | (flag as u8)
        } else {
            self.f & !(flag as u8)
        }
    }

    fn get_flag(&self, flag: Flag) -> bool {
        (self.f & (flag as u8)) != 0
    }

    fn set_flags(&mut self, carry: bool, half_carry: bool, sub: bool, zero: bool) {
        self.set_flag(Flag::Carry, carry);
        self.set_flag(Flag::Zero, zero);
        self.set_flag(Flag::Sub, sub);
        self.set_flag(Flag::HalfCarry, half_carry);
    }

    fn evaluate_flow_condition(&self, condition: FlowCondition) -> bool {
        match condition {
            FlowCondition::NotZero => !self.get_flag(Flag::Zero),
            FlowCondition::Zero => self.get_flag(Flag::Zero),
            FlowCondition::NotCarry => !self.get_flag(Flag::Carry),
            FlowCondition::Carry => self.get_flag(Flag::Carry),
        }
    }

    fn push_to_stack_16b(&mut self, value: u16, bus: &mut Bus) {
        let addr = self.sp.wrapping_sub(2);
        bus.write_to_16b(addr, value);
        self.sp = addr;
    }

    fn pop_from_stack_16b(&mut self, bus: &Bus) -> u16 {
        let stack_value = bus.read_from_16b(self.sp);
        self.sp = self.sp.wrapping_add(2);
        stack_value
    }

    // Instructions

    fn ld(&mut self, register_to: Register, register_from: Register) -> u64 {
        let register_from_value = self.get_register(register_from);
        self.set_register(register_to, register_from_value);
        1
    }

    fn ld_imm(&mut self, register_to: Register, imm_value: u8) -> u64 {
        self.set_register(register_to, imm_value);
        2
    }

    fn ld_from_hl_ind(&mut self, register_to: Register, bus: &mut Bus) -> u64 {
        let bus_value = bus.read_from_8b(self.get_hl());
        self.set_register(register_to, bus_value);
        2
    }

    fn ld_to_hl_ind(&mut self, register_from: Register, bus: &mut Bus) -> u64 {
        let register_from_value = self.get_register(register_from);
        bus.write_to_8b(self.get_hl(), register_from_value);
        2
    }

    fn ld_to_hl_ind_imm(&mut self, value: u8, bus: &mut Bus) -> u64 {
        bus.write_to_8b(self.get_hl(), value);
        3
    }

    fn ld_from_bc_ind_to_a(&mut self, bus: &mut Bus) -> u64 {
        let bus_value = bus.read_from_8b(self.get_bc());
        self.a = bus_value;
        2
    }

    fn ld_from_de_ind_to_a(&mut self, bus: &mut Bus) -> u64 {
        let bus_value = bus.read_from_8b(self.get_de());
        self.a = bus_value;
        2
    }

    fn ld_to_bc_ind_from_a(&mut self, bus: &mut Bus) -> u64 {
        bus.write_to_8b(self.get_bc(), self.a);
        2
    }

    fn ld_to_de_ind_from_a(&mut self, bus: &mut Bus) -> u64 {
        bus.write_to_8b(self.get_de(), self.a);
        2
    }

    fn ld_from_imm_ind_to_a(&mut self, imm: u16, bus: &mut Bus) -> u64 {
        self.a = bus.read_from_8b(imm);
        4
    }

    fn ld_to_imm_ind_from_a(&mut self, imm: u16, bus: &mut Bus) -> u64 {
        bus.write_to_8b(imm, self.a);
        4
    }

    fn ld_to_a_from_c_ind(&mut self, bus: &mut Bus) -> u64 {
        let addr: u16 = 0xff00 | (self.c as u16);
        self.a = bus.read_from_8b(addr);
        2
    }

    fn ld_from_a_to_c_ind(&mut self, bus: &mut Bus) -> u64 {
        let addr: u16 = 0xff00 | (self.c as u16);
        bus.write_to_8b(addr, self.a);
        2
    }

    fn ld_from_imm_ind_to_a_8(&mut self, imm: u8, bus: &mut Bus) -> u64 {
        let addr: u16 = 0xff00 | (imm as u16);
        self.a = bus.read_from_8b(addr);
        3
    }

    fn ld_to_imm_ind_from_a_8(&mut self, imm: u8, bus: &mut Bus) -> u64 {
        let addr: u16 = 0xff00 | (imm as u16);
        bus.write_to_8b(addr, self.a);
        3
    }

    fn ld_from_hl_ind_dec_to_a(&mut self, bus: &mut Bus) -> u64 {
        let mut hl: u16 = self.get_hl();
        self.a = bus.read_from_8b(hl);
        hl = hl.wrapping_sub(1);
        self.set_hl(hl);
        2
    }

    fn ld_to_hl_ind_dec_from_a(&mut self, bus: &mut Bus) -> u64 {
        let mut hl: u16 = self.get_hl();
        bus.write_to_8b(hl, self.a);
        hl = hl.wrapping_sub(1);
        self.set_hl(hl);
        2
    }

    fn ld_from_hl_ind_inc_to_a(&mut self, bus: &mut Bus) -> u64 {
        let mut hl: u16 = self.get_hl();
        self.a = bus.read_from_8b(hl);
        hl = hl.wrapping_add(1);
        self.set_hl(hl);
        2
    }

    fn ld_to_hl_ind_inc_from_a(&mut self, bus: &mut Bus) -> u64 {
        let mut hl: u16 = self.get_hl();
        bus.write_to_8b(hl, self.a);
        hl = hl.wrapping_add(1);
        self.set_hl(hl);
        2
    }

    fn ld_imm_16(&mut self, register_to: Register16, imm_value: u16) -> u64 {
        self.set_register_16(register_to, imm_value);
        3
    }

    fn ld_to_imm_ind_from_sp(&mut self, imm_value: u16, bus: &mut Bus) -> u64 {
        bus.write_to_16b(imm_value, self.sp);
        5
    }

    fn ld_sp_from_hl(&mut self) -> u64 {
        self.sp = self.get_hl();
        2
    }

    fn push(&mut self, register_from: Register16, bus: &mut Bus) -> u64 {
        let register_value = self.get_register_16(register_from);
        self.push_to_stack_16b(register_value, bus);
        4
    }

    fn pop(&mut self, register: Register16, bus: &mut Bus) -> u64 {
        let stack_value = self.pop_from_stack_16b(bus);
        self.set_register_16(register, stack_value);
        3
    }

    fn ld_hl_from_adjusted_sp(&mut self, imm: i8) -> u64 {
        let adjusted_sp = self.sp.wrapping_add_signed(imm as i16);
        self.set_hl(adjusted_sp);

        let sp = self.sp;

        let half_carry = (sp & 0x000f).wrapping_add(((imm as u8) & 0x0f) as u16) > 0x000f;
        let did_overflow = (sp & 0x00ff).wrapping_add(imm as u8 as u16) > 0x00ff;

        self.set_flags(did_overflow, half_carry, false, false);
        3
    }

    fn _add(&mut self, value: u8) {
        let (new_value, did_overflow) = self.a.overflowing_add(value);
        let half_carry = (self.a & 0x0f) + (value & 0x0f) > 0x0f;

        self.set_flags(did_overflow, half_carry, false, new_value == 0);

        self.a = new_value;
    }

    fn _adc(&mut self, value: u8) {
        let carry_flag = if self.get_flag(Flag::Carry) { 1 } else { 0 };

        let (first_add, did_overflow1) = self.a.overflowing_add(value);
        let (second_add, did_overflow2) = first_add.overflowing_add(carry_flag);

        let half_carry = (self.a & 0x0f) + (value & 0x0f) + carry_flag > 0x0f;
        let did_overflow = did_overflow1 || did_overflow2;

        self.set_flags(did_overflow, half_carry, false, second_add == 0);

        self.a = second_add;
    }

    fn add(&mut self, register: Register) -> u64 {
        let value = self.get_register(register);
        self._add(value);
        1
    }

    fn add_hl_ind(&mut self, bus: &mut Bus) -> u64 {
        let value = bus.read_from_8b(self.get_hl());
        self._add(value);
        2
    }

    fn add_imm(&mut self, imm: u8) -> u64 {
        self._add(imm);
        2
    }

    fn add_c(&mut self, register: Register) -> u64 {
        let value = self.get_register(register);
        self._adc(value);
        1
    }

    fn add_c_hl_ind(&mut self, bus: &mut Bus) -> u64 {
        let value = bus.read_from_8b(self.get_hl());
        self._adc(value);
        2
    }

    fn add_c_imm(&mut self, imm: u8) -> u64 {
        self._adc(imm);
        2
    }

    fn _sub(&mut self, value: u8) {
        let (new_register_value, did_borrow) = self.a.overflowing_sub(value);
        let half_carry = self.a & 0x0f < value & 0x0f;

        self.set_flags(did_borrow, half_carry, true, new_register_value == 0);

        self.a = new_register_value;
    }

    fn _sbc(&mut self, value: u8) {
        let carry_flag = if self.get_flag(Flag::Carry) { 1 } else { 0 };

        let (fist_sub, did_borrow1) = self.a.overflowing_sub(value);
        let (second_sub, did_borrow2) = fist_sub.overflowing_sub(carry_flag);

        let half_carry = self.a & 0x0f < (value & 0x0f) + carry_flag;
        let did_borrow = did_borrow1 || did_borrow2;

        self.set_flags(did_borrow, half_carry, true, second_sub == 0);

        self.a = second_sub;
    }

    fn sub(&mut self, register: Register) -> u64 {
        let value = self.get_register(register);
        self._sub(value);
        1
    }

    fn sub_hl_ind(&mut self, bus: &mut Bus) -> u64 {
        let value = bus.read_from_8b(self.get_hl());
        self._sub(value);
        2
    }

    fn sub_imm(&mut self, imm: u8) -> u64 {
        self._sub(imm);
        2
    }

    fn sub_c(&mut self, register: Register) -> u64 {
        let value = self.get_register(register);
        self._sbc(value);
        1
    }

    fn sub_c_hl_ind(&mut self, bus: &mut Bus) -> u64 {
        let value = bus.read_from_8b(self.get_hl());
        self._sbc(value);
        2
    }

    fn sub_c_imm(&mut self, imm: u8) -> u64 {
        self._sbc(imm);
        2
    }

    fn _cp(&mut self, value: u8) {
        let (new_register_value, did_borrow) = self.a.overflowing_sub(value);
        let half_carry = self.a & 0x0f < value & 0x0f;

        self.set_flags(did_borrow, half_carry, true, new_register_value == 0);
    }

    fn cp(&mut self, register: Register) -> u64 {
        let value = self.get_register(register);
        self._cp(value);
        1
    }

    fn cp_hl_ind(&mut self, bus: &mut Bus) -> u64 {
        let value = bus.read_from_8b(self.get_hl());
        self._cp(value);
        2
    }

    fn cp_imm(&mut self, imm: u8) -> u64 {
        self._cp(imm);
        2
    }

    fn inc(&mut self, register: Register) -> u64 {
        let value = self.get_register(register.clone());

        let (new_value, _) = value.overflowing_add(1);
        let half_carry = (value & 0x0f) + 0b1 > 0x0f;
        let current_carry = self.get_flag(Flag::Carry);

        self.set_flags(current_carry, half_carry, false, new_value == 0);

        self.set_register(register, new_value);
        1
    }

    fn inc_hl_ind(&mut self, bus: &mut Bus) -> u64 {
        let value = bus.read_from_8b(self.get_hl());

        let (new_value, _) = value.overflowing_add(1);
        let half_carry = (value & 0x0f) + 0b1 > 0x0f;
        let current_carry = self.get_flag(Flag::Carry);

        self.set_flags(current_carry, half_carry, false, new_value == 0);

        bus.write_to_8b(self.get_hl(), new_value);
        3
    }

    fn dec(&mut self, register: Register) -> u64 {
        let value = self.get_register(register.clone());

        let (new_value, _) = value.overflowing_sub(1);
        let half_carry = (value & 0x0f) == 0;
        let current_carry = self.get_flag(Flag::Carry);

        self.set_flags(current_carry, half_carry, true, new_value == 0);

        self.set_register(register, new_value);
        1
    }

    fn dec_hl_ind(&mut self, bus: &mut Bus) -> u64 {
        let value = bus.read_from_8b(self.get_hl());

        let (new_value, _) = value.overflowing_sub(1);
        let half_carry = (value & 0x0f) == 0;
        let current_carry = self.get_flag(Flag::Carry);

        self.set_flags(current_carry, half_carry, true, new_value == 0);

        bus.write_to_8b(self.get_hl(), new_value);
        3
    }

    fn _and(&mut self, value: u8) {
        let new_value = self.a & value;

        self.set_flags(false, true, false, new_value == 0);

        self.a = new_value;
    }

    fn and(&mut self, register: Register) -> u64 {
        let value = self.get_register(register);
        self._and(value);
        1
    }

    fn and_hl_ind(&mut self, bus: &mut Bus) -> u64 {
        let value = bus.read_from_8b(self.get_hl());
        self._and(value);
        2
    }

    fn and_imm(&mut self, imm: u8) -> u64 {
        self._and(imm);
        2
    }

    fn _or(&mut self, value: u8) {
        let new_value = self.a | value;

        self.set_flags(false, false, false, new_value == 0);

        self.a = new_value;
    }

    fn or(&mut self, register: Register) -> u64 {
        let value = self.get_register(register);
        self._or(value);
        1
    }

    fn or_hl_ind(&mut self, bus: &mut Bus) -> u64 {
        let value = bus.read_from_8b(self.get_hl());
        self._or(value);
        2
    }

    fn or_imm(&mut self, imm: u8) -> u64 {
        self._or(imm);
        2
    }

    fn _xor(&mut self, value: u8) {
        let new_value = self.a ^ value;

        self.set_flags(false, false, false, new_value == 0);

        self.a = new_value;
    }

    fn xor(&mut self, register: Register) -> u64 {
        let value = self.get_register(register);
        self._xor(value);
        1
    }

    fn xor_hl_ind(&mut self, bus: &mut Bus) -> u64 {
        let value = bus.read_from_8b(self.get_hl());
        self._xor(value);
        2
    }

    fn xor_imm(&mut self, imm: u8) -> u64 {
        self._xor(imm);
        2
    }

    fn ccf(&mut self) -> u64 {
        let current_carry = self.get_flag(Flag::Carry);
        let current_zero = self.get_flag(Flag::Zero);

        self.set_flags(!current_carry, false, false, current_zero);
        1
    }

    fn scf(&mut self) -> u64 {
        let current_zero = self.get_flag(Flag::Zero);
        self.set_flags(true, false, false, current_zero);
        1
    }

    fn daa(&mut self) -> u64 {
        let current_sub = self.get_flag(Flag::Sub);
        let current_half_carry = self.get_flag(Flag::HalfCarry);
        let mut carry = self.get_flag(Flag::Carry);

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

        self.set_flags(carry, false, current_sub, new_value == 0);

        self.a = new_value;
        1
    }

    fn cpl(&mut self) -> u64 {
        let current_carry = self.get_flag(Flag::Carry);
        let current_zero = self.get_flag(Flag::Zero);

        self.set_flags(current_carry, true, true, current_zero);

        self.a = !self.a;
        1
    }

    fn inc_16(&mut self, register: Register16) -> u64 {
        let value = self.get_register_16(register.clone());
        self.set_register_16(register, value.wrapping_add(1));
        2
    }

    fn dec_16(&mut self, register: Register16) -> u64 {
        let value = self.get_register_16(register.clone());
        self.set_register_16(register, value.wrapping_sub(1));
        2
    }

    fn add_hl(&mut self, register: Register16) -> u64 {
        let register_value = self.get_register_16(register);
        let hl = self.get_hl();

        let (new_value, did_overflow) = hl.overflowing_add(register_value);
        let current_zero = self.get_flag(Flag::Zero);
        let half_carry = (hl & 0x0fff) + (register_value & 0x0fff) > 0x0fff;

        self.set_flags(did_overflow, half_carry, false, current_zero);

        self.set_hl(new_value);
        2
    }

    fn add_sp_imm(&mut self, imm: i8) -> u64 {
        let new_value = self.sp.wrapping_add_signed(imm as i16);

        let sp = self.sp;

        let half_carry = (sp & 0x000f).wrapping_add(((imm as u8) & 0x0f) as u16) > 0x000f;
        let did_overflow = (sp & 0x00ff).wrapping_add(imm as u8 as u16) > 0x00ff;

        self.set_flags(did_overflow, half_carry, false, false);

        self.sp = new_value;
        4
    }

    fn _rotate(&mut self, value: u8, circular: bool, right: bool, accumulator: bool) -> u8 {
        let carry_bit_pos: u8 = if right { 0x01 } else { 0x80 };
        let old_carry = self.get_flag(Flag::Carry);
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

        self.set_flags(carry, false, false, zero);

        new_value
    }

    fn rlca(&mut self) -> u64 {
        self.a = self._rotate(self.a, true, false, true);
        1
    }

    fn rrca(&mut self) -> u64 {
        self.a = self._rotate(self.a, true, true, true);
        1
    }

    fn rra(&mut self) -> u64 {
        self.a = self._rotate(self.a, false, true, true);
        1
    }

    fn rla(&mut self) -> u64 {
        self.a = self._rotate(self.a, false, false, true);
        1
    }

    fn rlcr(&mut self, register: Register) -> u64 {
        let register_value = self.get_register(register.clone());

        let new_value = self._rotate(register_value, true, false, false);

        self.set_register(register, new_value);
        2
    }

    fn rlc_hl_ind(&mut self, bus: &mut Bus) -> u64 {
        let register_value = bus.read_from_8b(self.get_hl());

        let new_value = self._rotate(register_value, true, false, false);

        bus.write_to_8b(self.get_hl(), new_value);
        4
    }

    fn rrcr(&mut self, register: Register) -> u64 {
        let register_value = self.get_register(register.clone());

        let new_value = self._rotate(register_value, true, true, false);

        self.set_register(register, new_value);
        2
    }

    fn rrc_hl_ind(&mut self, bus: &mut Bus) -> u64 {
        let register_value = bus.read_from_8b(self.get_hl());

        let new_value = self._rotate(register_value, true, true, false);

        bus.write_to_8b(self.get_hl(), new_value);
        4
    }

    fn rrr(&mut self, register: Register) -> u64 {
        let register_value = self.get_register(register.clone());

        let new_value = self._rotate(register_value, false, true, false);

        self.set_register(register, new_value);
        2
    }

    fn rlr(&mut self, register: Register) -> u64 {
        let register_value = self.get_register(register.clone());

        let new_value = self._rotate(register_value, false, false, false);

        self.set_register(register, new_value);
        2
    }

    fn rr_hl_ind(&mut self, bus: &mut Bus) -> u64 {
        let register_value = bus.read_from_8b(self.get_hl());

        let new_value = self._rotate(register_value, false, true, false);

        bus.write_to_8b(self.get_hl(), new_value);
        4
    }

    fn rl_hl_ind(&mut self, bus: &mut Bus) -> u64 {
        let register_value = bus.read_from_8b(self.get_hl());

        let new_value = self._rotate(register_value, false, false, false);

        bus.write_to_8b(self.get_hl(), new_value);
        4
    }

    fn _sa(&mut self, value: u8, right: bool) -> u8 {
        let msb = value & 0x80;
        let lsb = value & 0x01;

        let carry = if right { lsb != 0 } else { msb != 0 };
        let new_value = if right {
            (value >> 1) | msb
        } else {
            value << 1
        };

        self.set_flags(carry, false, false, new_value == 0);

        new_value
    }

    fn srar(&mut self, register: Register) -> u64 {
        let register_value = self.get_register(register.clone());

        let new_value = self._sa(register_value, true);

        self.set_register(register, new_value);
        2
    }

    fn sra_hl_ind(&mut self, bus: &mut Bus) -> u64 {
        let register_value = bus.read_from_8b(self.get_hl());

        let new_value = self._sa(register_value, true);

        bus.write_to_8b(self.get_hl(), new_value);
        4
    }

    fn slar(&mut self, register: Register) -> u64 {
        let register_value = self.get_register(register.clone());

        let new_value = self._sa(register_value, false);

        self.set_register(register, new_value);
        2
    }

    fn sla_hl_ind(&mut self, bus: &mut Bus) -> u64 {
        let register_value = bus.read_from_8b(self.get_hl());

        let new_value = self._sa(register_value, false);

        bus.write_to_8b(self.get_hl(), new_value);
        4
    }

    fn _sl(&mut self, value: u8, right: bool) -> u8 {
        let msb = value & 0x80;
        let lsb = value & 0x01;

        let carry = if right { lsb != 0 } else { msb != 0 };
        let new_value = if right { value >> 1 } else { value << 1 };

        self.set_flags(carry, false, false, new_value == 0);

        new_value
    }

    fn srlr(&mut self, register: Register) -> u64 {
        let register_value = self.get_register(register.clone());

        let new_value = self._sl(register_value, true);

        self.set_register(register, new_value);
        2
    }

    fn srl_hl_ind(&mut self, bus: &mut Bus) -> u64 {
        let register_value = bus.read_from_8b(self.get_hl());

        let new_value = self._sl(register_value, true);

        bus.write_to_8b(self.get_hl(), new_value);
        4
    }

    fn _swap(&mut self, value: u8) -> u8 {
        let new_value = value.rotate_left(4);

        self.set_flags(false, false, false, new_value == 0);

        new_value
    }

    fn swapr(&mut self, register: Register) -> u64 {
        let register_value = self.get_register(register.clone());

        let new_value = self._swap(register_value);

        self.set_register(register, new_value);
        2
    }

    fn swap_hl_ind(&mut self, bus: &mut Bus) -> u64 {
        let register_value = bus.read_from_8b(self.get_hl());

        let new_value = self._swap(register_value);

        bus.write_to_8b(self.get_hl(), new_value);
        4
    }

    fn _bit(&mut self, bit_position: u8, value: u8) {
        let old_carry = self.get_flag(Flag::Carry);

        let bit_is_set = value & (1 << bit_position);

        self.set_flags(old_carry, true, false, bit_is_set == 0);
    }

    fn bitr(&mut self, bit_position: u8, register: Register) -> u64 {
        let register_value = self.get_register(register);
        self._bit(bit_position, register_value);
        2
    }

    fn bit_hl_ind(&mut self, bit_position: u8, bus: &mut Bus) -> u64 {
        let register_value = bus.read_from_8b(self.get_hl());
        self._bit(bit_position, register_value);
        3
    }

    fn setr(&mut self, bit_position: u8, register: Register) -> u64 {
        let register_value = self.get_register(register.clone());
        let new_value = bit_utils::modify_bit(register_value, bit_position, true);
        self.set_register(register, new_value);
        2
    }

    fn set_hl_ind(&mut self, bit_position: u8, bus: &mut Bus) -> u64 {
        let register_value = bus.read_from_8b(self.get_hl());
        let new_value = bit_utils::modify_bit(register_value, bit_position, true);
        bus.write_to_8b(self.get_hl(), new_value);
        4
    }

    fn resetr(&mut self, bit_position: u8, register: Register) -> u64 {
        let register_value = self.get_register(register.clone());
        let new_value = bit_utils::modify_bit(register_value, bit_position, false);
        self.set_register(register, new_value);
        2
    }

    fn reset_hl_ind(&mut self, bit_position: u8, bus: &mut Bus) -> u64 {
        let register_value = bus.read_from_8b(self.get_hl());
        let new_value = bit_utils::modify_bit(register_value, bit_position, false);
        bus.write_to_8b(self.get_hl(), new_value);
        4
    }

    fn nop(&self) -> u64 {
        1
    }

    fn jp(&mut self, addr: u16) -> u64 {
        self.pc = addr;
        4
    }

    fn jp_hl(&mut self) -> u64 {
        self.pc = self.get_hl();
        1
    }

    fn jp_cc(&mut self, cond: FlowCondition, addr: u16) -> u64 {
        if self.evaluate_flow_condition(cond) {
            self.jp(addr);
            4
        } else {
            3
        }
    }

    fn jr(&mut self, imm: i8) -> u64 {
        self.pc = self.pc.wrapping_add_signed(imm as i16);
        3
    }

    fn jr_cc(&mut self, cond: FlowCondition, imm: i8) -> u64 {
        if self.evaluate_flow_condition(cond) {
            self.jr(imm);
            3
        } else {
            2
        }
    }

    fn call(&mut self, addr: u16, bus: &mut Bus) -> u64 {
        self.push_to_stack_16b(self.pc, bus);
        self.pc = addr;
        6
    }

    fn call_cc(&mut self, cond: FlowCondition, addr: u16, bus: &mut Bus) -> u64 {
        if self.evaluate_flow_condition(cond) {
            self.call(addr, bus);
            6
        } else {
            3
        }
    }

    fn ret(&mut self, bus: &mut Bus) -> u64 {
        self.pc = self.pop_from_stack_16b(bus);
        4
    }

    fn ret_cc(&mut self, cond: FlowCondition, bus: &mut Bus) -> u64 {
        if self.evaluate_flow_condition(cond) {
            self.ret(bus);
            5
        } else {
            2
        }
    }

    fn ret_i(&mut self, bus: &mut Bus) -> u64 {
        self.ret(bus);
        self.ei();
        4
    }

    fn rst(&mut self, addr: u8, bus: &mut Bus) -> u64 {
        self.push_to_stack_16b(self.pc, bus);
        self.pc = (addr as u16) * 8;
        4
    }

    fn halt(&mut self) -> u64 {
        self.halted = true;
        1
    }

    fn stop(&mut self) -> u64 {
        self.halted = true;
        1
    }

    fn di(&mut self) -> u64 {
        self.interrupts_enabled = false;
        1
    }

    fn ei(&mut self) -> u64 {
        self.interrupts_enabled = true;
        1
    }

    fn decode_instruction_at_pc(&self, bus: &Bus) -> (Instruction, u16) {
        let first_byte = bus.read_from_8b(self.pc);
        let second_byte = bus.read_from_8b(self.pc.wrapping_add(1));
        let third_byte = bus.read_from_8b(self.pc.wrapping_add(2));
        Instruction::decode(first_byte, second_byte, third_byte)
    }

    pub fn clock(&mut self, bus: &mut Bus) {
        if self.halted {
            return;
        }

        let (instruction, size) = self.decode_instruction_at_pc(bus);

        self.step(size);

        let cycles = self.execute(instruction, bus);

        self.clock = self.clock.wrapping_add(cycles);
    }

    fn step(&mut self, instruction_size: u16) {
        self.pc = self.pc.wrapping_add(instruction_size);
    }

    fn execute(&mut self, instruction: Instruction, bus: &mut Bus) -> u64 {
        match instruction {
            Instruction::HALT() => self.halt(),
            Instruction::LD(r8_operand_dest, r8_operand_source) => {
                match (r8_operand_dest.clone(), r8_operand_source.clone()) {
                    (R8Operand::HLInd, R8Operand::HLInd) => self.halt(),
                    (R8Operand::HLInd, _) => {
                        let register_source: Register =
                            Register::from_r8_operand(r8_operand_source);
                        self.ld_to_hl_ind(register_source, bus)
                    }
                    (_, R8Operand::HLInd) => {
                        let register_dest: Register = Register::from_r8_operand(r8_operand_dest);
                        self.ld_from_hl_ind(register_dest, bus)
                    }
                    (_, _) => {
                        let register_source: Register =
                            Register::from_r8_operand(r8_operand_source);
                        let register_dest: Register = Register::from_r8_operand(r8_operand_dest);
                        self.ld(register_dest, register_source)
                    }
                }
            }
            Instruction::LDImm8(r8_operand, imm) => match r8_operand {
                R8Operand::HLInd => self.ld_to_hl_ind_imm(imm, bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.ld_imm(register, imm)
                }
            },
            Instruction::LDToMemFromA(r16mem_operand) => {
                match r16mem_operand {
                    R16MemOperand::BC => self.ld_to_bc_ind_from_a(bus),
                    R16MemOperand::DE => self.ld_to_de_ind_from_a(bus),
                    R16MemOperand::HLD => self.ld_to_hl_ind_dec_from_a(bus),
                    R16MemOperand::HLI => self.ld_to_hl_ind_inc_from_a(bus),
                }
            }
            Instruction::LDFromImmIndToA16(imm) => self.ld_from_imm_ind_to_a(imm, bus),
            Instruction::LDToImmIndFromA16(imm) => self.ld_to_imm_ind_from_a(imm, bus),
            Instruction::LDToAFromCInd() => self.ld_to_a_from_c_ind(bus),
            Instruction::LDFromAToCInd() => self.ld_from_a_to_c_ind(bus),
            Instruction::LDFromImmIndToA8(imm) => self.ld_from_imm_ind_to_a_8(imm, bus),
            Instruction::LDToImmIndFromA8(imm) => self.ld_to_imm_ind_from_a_8(imm, bus),
            Instruction::LDFromMemToA(r16mem_operand) => {
                match r16mem_operand {
                    R16MemOperand::BC => self.ld_from_bc_ind_to_a(bus),
                    R16MemOperand::DE => self.ld_from_de_ind_to_a(bus),
                    R16MemOperand::HLD => self.ld_from_hl_ind_dec_to_a(bus),
                    R16MemOperand::HLI => self.ld_from_hl_ind_inc_to_a(bus),
                }
            }
            Instruction::LDImm16(r16_operand, imm) => {
                let register_16 = Register16::from_r16_operand(r16_operand);
                self.ld_imm_16(register_16, imm)
            }
            Instruction::LDToImmIndFromSP(imm) => self.ld_to_imm_ind_from_sp(imm, bus),
            Instruction::LDSPFromHL() => self.ld_sp_from_hl(),
            Instruction::PUSH(r16stk_operand) => {
                let register_16 = Register16::from_r16stk_operand(r16stk_operand);
                self.push(register_16, bus)
            }
            Instruction::POP(r16stk_operand) => {
                let register_16 = Register16::from_r16stk_operand(r16stk_operand);
                self.pop(register_16, bus)
            }
            Instruction::LDHLFromAdjustedSP(imm) => self.ld_hl_from_adjusted_sp(imm),
            Instruction::ADD(r8_operand) => match r8_operand {
                R8Operand::HLInd => self.add_hl_ind(bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.add(register)
                }
            },
            Instruction::ADDImm(imm) => self.add_imm(imm),
            Instruction::ADC(r8_operand) => match r8_operand {
                R8Operand::HLInd => self.add_c_hl_ind(bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.add_c(register)
                }
            },
            Instruction::ADCImm(imm) => self.add_c_imm(imm),
            Instruction::SUB(r8_operand) => match r8_operand {
                R8Operand::HLInd => self.sub_hl_ind(bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.sub(register)
                }
            },
            Instruction::SUBImm(imm) => self.sub_imm(imm),
            Instruction::SBC(r8_operand) => match r8_operand {
                R8Operand::HLInd => self.sub_c_hl_ind(bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.sub_c(register)
                }
            },
            Instruction::SBCImm(imm) => self.sub_c_imm(imm),
            Instruction::CP(r8_operand) => match r8_operand {
                R8Operand::HLInd => self.cp_hl_ind(bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.cp(register)
                }
            },
            Instruction::CPImm(imm) => self.cp_imm(imm),
            Instruction::INC8(r8_operand) => match r8_operand {
                R8Operand::HLInd => self.inc_hl_ind(bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.inc(register)
                }
            },
            Instruction::DEC8(r8_operand) => match r8_operand {
                R8Operand::HLInd => self.dec_hl_ind(bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.dec(register)
                }
            },
            Instruction::AND(r8_operand) => match r8_operand {
                R8Operand::HLInd => self.and_hl_ind(bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.and(register)
                }
            },
            Instruction::ANDImm(imm) => self.and_imm(imm),
            Instruction::OR(r8_operand) => match r8_operand {
                R8Operand::HLInd => self.or_hl_ind(bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.or(register)
                }
            },
            Instruction::ORImm(imm) => self.or_imm(imm),
            Instruction::XOR(r8_operand) => match r8_operand {
                R8Operand::HLInd => self.xor_hl_ind(bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.xor(register)
                }
            },
            Instruction::XORImm(imm) => self.xor_imm(imm),
            Instruction::CCF() => self.ccf(),
            Instruction::SCF() => self.scf(),
            Instruction::DAA() => self.daa(),
            Instruction::CPL() => self.cpl(),
            Instruction::INC16(r16_operand) => {
                let register_16 = Register16::from_r16_operand(r16_operand);
                self.inc_16(register_16)
            }
            Instruction::DEC16(r16_operand) => {
                let register_16 = Register16::from_r16_operand(r16_operand);
                self.dec_16(register_16)
            }
            Instruction::ADDHL(r16_operand) => {
                let register_16 = Register16::from_r16_operand(r16_operand);
                self.add_hl(register_16)
            }
            Instruction::ADDSPImm(imm) => self.add_sp_imm(imm),
            Instruction::RLCA() => self.rlca(),
            Instruction::RRCA() => self.rrca(),
            Instruction::RRA() => self.rra(),
            Instruction::RLA() => self.rla(),
            Instruction::RLC(r8_operand) => match r8_operand {
                R8Operand::HLInd => self.rlc_hl_ind(bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.rlcr(register)
                }
            },
            Instruction::RRC(r8_operand) => match r8_operand {
                R8Operand::HLInd => self.rrc_hl_ind(bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.rrcr(register)
                }
            },
            Instruction::RL(r8_operand) => match r8_operand {
                R8Operand::HLInd => self.rl_hl_ind(bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.rlr(register)
                }
            },
            Instruction::RR(r8_operand) => match r8_operand {
                R8Operand::HLInd => self.rr_hl_ind(bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.rrr(register)
                }
            },
            Instruction::SRA(r8_operand) => match r8_operand {
                R8Operand::HLInd => self.sra_hl_ind(bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.srar(register)
                }
            },
            Instruction::SLA(r8_operand) => match r8_operand {
                R8Operand::HLInd => self.sla_hl_ind(bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.slar(register)
                }
            },
            Instruction::SRL(r8_operand) => match r8_operand {
                R8Operand::HLInd => self.srl_hl_ind(bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.srlr(register)
                }
            },
            Instruction::SWAP(r8_operand) => match r8_operand {
                R8Operand::HLInd => self.swap_hl_ind(bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.swapr(register)
                }
            },
            Instruction::BIT(bit, r8_operand) => match r8_operand {
                R8Operand::HLInd => self.bit_hl_ind(bit, bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.bitr(bit, register)
                }
            },
            Instruction::SET(bit, r8_operand) => match r8_operand {
                R8Operand::HLInd => self.set_hl_ind(bit, bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.setr(bit, register)
                }
            },
            Instruction::RESET(bit, r8_operand) => match r8_operand {
                R8Operand::HLInd => self.reset_hl_ind(bit, bus),
                _ => {
                    let register = Register::from_r8_operand(r8_operand);
                    self.resetr(bit, register)
                }
            },
            Instruction::NOP() => self.nop(),
            Instruction::JP(addr) => self.jp(addr),
            Instruction::JPHL() => self.jp_hl(),
            Instruction::JPCC(cond, addr) => self.jp_cc(cond, addr),
            Instruction::JR(imm) => self.jr(imm),
            Instruction::JRCC(cond, imm) => self.jr_cc(cond, imm),
            Instruction::CALL(addr) => self.call(addr, bus),
            Instruction::CALLCC(cond, addr) => self.call_cc(cond, addr, bus),
            Instruction::RET() => self.ret(bus),
            Instruction::RETCC(cond) => self.ret_cc(cond, bus),
            Instruction::RETI() => self.ret_i(bus),
            Instruction::RST(addr) => self.rst(addr, bus),
            Instruction::STOP() => self.stop(),
            Instruction::DI() => self.di(),
            Instruction::EI() => self.ei(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::console::cpu::cpu::*;
    use crate::console::cpu::instruction::R8Operand;

    fn execute_cb_instruction(cpu: &mut Cpu, bus: &mut Bus, offset: &mut u16, subopcode: u8) {
        bus.write_to_8b(*offset, 0xcb);
        bus.write_to_8b(*offset + 1, subopcode);
        *offset += 2;
        cpu.clock(bus);
    }

    fn preload_hl(bus: &mut Bus, cpu: &Cpu, value: u8) {
        let hl_addr = cpu.get_register_16(Register16::HL);
        bus.write_to_8b(hl_addr, value);
    }

    macro_rules! assert_reg {
        ($cpu:expr, $reg:expr, $expected:expr) => {
            assert_eq!(
                $cpu.get_register($reg),
                $expected,
                "{} expected to be {:#04X}",
                stringify!($reg),
                $expected
            )
        };
    }

    macro_rules! assert_mem {
        ($bus:expr, $addr:expr, $expected:expr) => {
            assert_eq!(
                $bus.read_from_8b($addr),
                $expected,
                "memory at {:#04X} expected to be {:#04X}",
                $addr,
                $expected
            )
        };
    }

    #[test]
    fn test_cb_set_operations() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();
        let mut offset: u16 = 0;

        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b11_001_000 | R8Operand::E.to_byte(),
        );
        assert_reg!(cpu, Register::E, 0b0000_0010);

        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b11_000_000 | R8Operand::D.to_byte(),
        );
        assert_reg!(cpu, Register::D, 0b0000_0001);

        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b11_011_000 | R8Operand::A.to_byte(),
        );
        assert_reg!(cpu, Register::A, 0b0000_1000);

        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut bus, &cpu, 0xcb);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b11_010_000 | R8Operand::HLInd.to_byte(),
        );
        assert_mem!(bus, hl_addr, 0xcb | 0b0000_0100);

        // Ensure other registers remain unchanged.
        assert_reg!(cpu, Register::A, 0b0000_1000);
        assert_reg!(cpu, Register::E, 0b0000_0010);
        assert_reg!(cpu, Register::D, 0b0000_0001);
    }

    #[test]
    fn test_cb_res_operations() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();
        let mut offset: u16 = 0;

        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b11_001_000 | R8Operand::E.to_byte(),
        );
        assert_reg!(cpu, Register::E, 0b0000_0010);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b10_001_000 | R8Operand::E.to_byte(),
        );
        assert_reg!(cpu, Register::E, 0);

        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut bus, &cpu, 0xcb);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b10_011_000 | R8Operand::HLInd.to_byte(),
        );
        assert_mem!(bus, hl_addr, 0xc3);
    }

    #[test]
    fn test_cb_bit_operations() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();
        let mut offset: u16 = 0;

        cpu.set_register(Register::C, 0b0010_0000);

        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b01_001_000 | R8Operand::C.to_byte(),
        );
        assert!(
            cpu.get_flag(Flag::Zero),
            "BIT 1 in C should set the zero flag"
        );

        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b01_101_000 | R8Operand::C.to_byte(),
        );
        assert!(
            !cpu.get_flag(Flag::Zero),
            "BIT 5 in C should reset the zero flag"
        );

        preload_hl(&mut bus, &cpu, 0b0000_1000);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b01_011_000 | R8Operand::HLInd.to_byte(),
        );
        assert!(
            !cpu.get_flag(Flag::Zero),
            "BIT 3 in [HL] should reset the zero flag"
        );
    }

    #[test]
    fn test_cb_swap_operations() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();
        let mut offset: u16 = 0;

        cpu.set_register(Register::E, 0b1111_0000);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b00_110_000 | R8Operand::E.to_byte(),
        );
        assert_reg!(cpu, Register::E, 0b0000_1111);

        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut bus, &cpu, 0xcb);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b00_110_000 | R8Operand::HLInd.to_byte(),
        );
        assert_mem!(bus, hl_addr, 0xbc);
    }

    #[test]
    fn test_cb_srl_operations() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();
        let mut offset: u16 = 0;

        cpu.set_register(Register::B, 0b1111_0000);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b00_111_000 | R8Operand::B.to_byte(),
        );
        let original_lsb = (0b1111_0000 & 1) != 0;
        assert_reg!(cpu, Register::B, 0b0111_1000);
        assert_eq!(
            cpu.get_flag(Flag::Carry),
            original_lsb,
            "Carry flag should match original LSB"
        );

        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut bus, &cpu, 0b1100_1011);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b00_111_000 | R8Operand::HLInd.to_byte(),
        );
        assert_mem!(bus, hl_addr, 0b0110_0101);
        assert!(
            cpu.get_flag(Flag::Carry),
            "SRL [HL] should set the carry flag"
        );
    }

    #[test]
    fn test_cb_rrc_operations() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();
        let mut offset: u16 = 0;

        cpu.set_register(Register::E, 0b1111_0000);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b00_001_000 | R8Operand::E.to_byte(),
        );
        assert_reg!(cpu, Register::E, 0b0111_1000);
        assert!(
            !cpu.get_flag(Flag::Carry),
            "RRC on E should not set the carry flag"
        );

        // Test RRC on bus pointed by HL.
        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut bus, &cpu, 0b1110_0101);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b00_001_000 | R8Operand::HLInd.to_byte(),
        );
        assert_mem!(bus, hl_addr, 0b1111_0010);
        assert!(
            cpu.get_flag(Flag::Carry),
            "RRC [HL] should set the carry flag"
        );
    }

    #[test]
    fn test_cb_rlc_operations() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();
        let mut offset: u16 = 0;

        cpu.set_register(Register::A, 0b0000_1111);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b00_000_000 | R8Operand::A.to_byte(),
        );
        assert_reg!(cpu, Register::A, 0b0001_1110);
        assert!(
            !cpu.get_flag(Flag::Carry),
            "RLC on A should not set the carry flag"
        );

        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut bus, &cpu, 0b0010_1111);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b00_000_000 | R8Operand::HLInd.to_byte(),
        );
        assert_mem!(bus, hl_addr, 0b0101_1110);
        assert!(
            !cpu.get_flag(Flag::Carry),
            "RLC [HL] should not set the carry flag"
        );
    }

    #[test]
    fn test_cb_sra_operations() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();
        let mut offset: u16 = 0;

        cpu.set_register(Register::D, 0b1111_0000);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b00_101_000 | R8Operand::D.to_byte(),
        );
        let original_lsb = (0b1111_0000 & 1) != 0;
        assert_reg!(cpu, Register::D, 0b1111_1000);
        assert_eq!(
            cpu.get_flag(Flag::Carry),
            original_lsb,
            "SRA D carry flag mismatch"
        );

        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut bus, &cpu, 0b1110_0101);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b00_101_000 | R8Operand::HLInd.to_byte(),
        );
        assert_mem!(bus, hl_addr, 0b1111_0010);
        assert!(
            cpu.get_flag(Flag::Carry),
            "SRA [HL] should set the carry flag"
        );
    }

    #[test]
    fn test_cb_sla_operations() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();
        let mut offset: u16 = 0;

        cpu.set_register(Register::E, 0b1111_0000);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b00_100_000 | R8Operand::E.to_byte(),
        );
        let original_msb = (0b1111_0000 & 0x80) != 0;
        assert_reg!(cpu, Register::E, 0b1110_0000);
        assert_eq!(
            cpu.get_flag(Flag::Carry),
            original_msb,
            "SLA E carry flag mismatch"
        );

        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut bus, &cpu, 0b0010_1011);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b00_100_000 | R8Operand::HLInd.to_byte(),
        );
        assert_mem!(bus, hl_addr, 0b0101_0110);
        assert!(
            !cpu.get_flag(Flag::Carry),
            "SLA [HL] should not set the carry flag"
        );
    }

    #[test]
    fn test_cb_rr_operations() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();
        let mut offset: u16 = 0;

        cpu.set_register(Register::E, 0b1111_0000);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b00_011_000 | R8Operand::E.to_byte(),
        );
        assert_reg!(cpu, Register::E, 0b0111_1000);
        assert_eq!(
            cpu.get_flag(Flag::Carry),
            false,
            "RR on E should not set the carry flag"
        );

        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut bus, &cpu, 0b1110_0101);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b00_011_000 | R8Operand::HLInd.to_byte(),
        );
        assert_mem!(bus, hl_addr, 0b0111_0010);
        assert!(
            cpu.get_flag(Flag::Carry),
            "RR [HL] should set the carry flag"
        );
    }

    #[test]
    fn test_cb_rl_operations() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();
        let mut offset: u16 = 0;

        cpu.set_register(Register::A, 0b0000_1111);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b00_010_000 | R8Operand::A.to_byte(),
        );
        assert_reg!(cpu, Register::A, 0b0001_1110);
        assert_eq!(
            cpu.get_flag(Flag::Carry),
            false,
            "RL on A should not set the carry flag"
        );

        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut bus, &cpu, 0b0010_1111);
        execute_cb_instruction(
            &mut cpu,
            &mut bus,
            &mut offset,
            0b00_010_000 | R8Operand::HLInd.to_byte(),
        );
        assert_mem!(bus, hl_addr, 0b0101_1110);
        assert_eq!(
            cpu.get_flag(Flag::Carry),
            false,
            "RL [HL] should not set the carry flag"
        );
    }
}
