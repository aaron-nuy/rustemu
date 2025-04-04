use crate::console::utils::bit_utils;
use crate::console::cpu::instruction::*;
use crate::console::memory::*;

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
    clock: u64
}

impl Cpu {
    const F_ZERO_FLAG_POS: u8 = 7;
    const F_SUB_FLAG_POS: u8 = 6;
    const F_HALF_CARRY_FLAG_POS: u8 = 5;
    const F_CARRY_FLAG_POS: u8 = 4;

    pub fn new() -> Self {
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
            Register::L => self.l
        }
    }

    fn get_register_16(&self, register: Register16) -> u16 {
        match register {
            Register16::AF => self.get_af(),
            Register16::BC => self.get_bc(),
            Register16::DE => self.get_de(),
            Register16::HL => self.get_hl(),
            Register16::SP => self.sp,
            Register16::PC => self.pc
        }
    }

    fn set_register_bit(&mut self, register: Register, bit_position: u8, on: bool) {
        let register_value = self.get_register(register.clone());
        let new_register_value = bit_utils::modify_bit(register_value, bit_position, on);
        self.set_register(register, new_register_value);
    }

    fn get_register_bit(&self, register: Register, bit_position: u8) -> bool {
        let register_value = self.get_register(register.clone());
        bit_utils::get_bit(register_value, bit_position)
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

    fn evaluate_flow_condition(&self, condition: FlowCondition) -> bool {
        match condition {
            FlowCondition::NotZero => !bit_utils::get_bit(self.f, Cpu::F_ZERO_FLAG_POS),
            FlowCondition::Zero => bit_utils::get_bit(self.f, Cpu::F_ZERO_FLAG_POS),
            FlowCondition::NotCarry => !bit_utils::get_bit(self.f, Cpu::F_CARRY_FLAG_POS),
            FlowCondition::Carry => bit_utils::get_bit(self.f, Cpu::F_CARRY_FLAG_POS),
            FlowCondition::Always => true
        }
    }

    fn push_to_stack_16b(&mut self, value: u16, memory: &mut Memory) {
        let addr = self.sp.wrapping_sub(2);
        memory.write_to_16b(addr, value);
        self.sp = addr;
    }

    fn pop_from_stack_16b(&mut self, memory: &Memory) -> u16 {
        let stack_value = memory.read_from_16b(self.sp);
        self.sp = self.sp.wrapping_add(2);
        stack_value
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

    fn extract_8b_operand(&self, memory: &Memory) -> u8 {
        memory.read_from_8b(self.pc.wrapping_add(1))
    }

    fn extract_16b_operand(&self, memory: &Memory) -> u16 {
        memory.read_from_16b(self.pc.wrapping_add(1))
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

    fn ld_from_hl_ind(&mut self, register_to: Register, memory: &mut Memory) -> u64 {
        let memory_value = memory.read_from_8b(self.get_hl());
        self.set_register(register_to, memory_value);
        2
    }

    fn ld_to_hl_ind(&mut self, register_from: Register, memory: &mut Memory) -> u64 {
        let register_from_value = self.get_register(register_from);
        memory.write_to_8b(self.get_hl(), register_from_value);
        2
    }

    fn ld_to_hl_ind_imm(&mut self, value: u8, memory: &mut Memory) -> u64 {
        memory.write_to_8b(self.get_hl(), value);
        3
    }

    fn ld_from_bc_ind_to_a(&mut self, memory: &mut Memory) -> u64 {
        let memory_value = memory.read_from_8b(self.get_bc());
        self.a = memory_value;
        2
    }

    fn ld_from_de_ind_to_a(&mut self, memory: &mut Memory) -> u64 {
        let memory_value = memory.read_from_8b(self.get_de());
        self.a = memory_value;
        2
    }

    fn ld_to_bc_ind_from_a(&mut self, memory: &mut Memory) -> u64 {
        memory.write_to_8b(self.get_bc(), self.a);
        2
    }

    fn ld_to_de_ind_from_a(&mut self, memory: &mut Memory) -> u64 {
        memory.write_to_8b(self.get_de(), self.a);
        2
    }

    fn ld_from_imm_ind_to_a(&mut self, imm: u16, memory: &mut Memory) -> u64 {
        self.a = memory.read_from_8b(imm);
        4
    }

    fn ld_to_imm_ind_from_a(&mut self, imm: u16, memory: &mut Memory) -> u64 {
        memory.write_to_8b(imm, self.a);
        4
    }

    fn ld_to_a_from_c_ind(&mut self, memory: &mut Memory) -> u64 {
        let addr: u16 = 0xff00 | (self.c as u16);
        self.a = memory.read_from_8b(addr);
        2
    }

    fn ld_from_a_to_c_ind(&mut self, memory: &mut Memory) -> u64 {
        let addr: u16 = 0xff00 | (self.c as u16);
        memory.write_to_8b(addr, self.a);
        2
    }

    fn ld_from_imm_ind_to_a_8(&mut self, imm: u8, memory: &mut Memory) -> u64 {
        let addr: u16 = 0xff00 | (imm as u16);
        self.a = memory.read_from_8b(addr);
        3
    }

    fn ld_to_imm_ind_from_a_8(&mut self, imm: u8, memory: &mut Memory) -> u64 {
        let addr: u16 = 0xff00 | (imm as u16);
        memory.write_to_8b(addr, self.a);
        3
    }

    fn ld_from_hl_ind_dec_to_a(&mut self, memory: &mut Memory) -> u64 {
        let mut hl: u16 = self.get_hl();
        self.a = memory.read_from_8b(hl);
        hl = hl.wrapping_sub(1);
        self.set_hl(hl);
        2
    }

    fn ld_to_hl_ind_dec_from_a(&mut self, memory: &mut Memory) -> u64 {
        let mut hl: u16 = self.get_hl();
        memory.write_to_8b(hl, self.a);
        hl = hl.wrapping_sub(1);
        self.set_hl(hl);
        2
    }

    fn ld_from_hl_ind_inc_to_a(&mut self, memory: &mut Memory) -> u64 {
        let mut hl: u16 = self.get_hl();
        self.a = memory.read_from_8b(hl);
        hl = hl.wrapping_add(1);
        self.set_hl(hl);
        2
    }

    fn ld_to_hl_ind_inc_from_a(&mut self, memory: &mut Memory) -> u64 {
        let mut hl: u16 = self.get_hl();
        memory.write_to_8b(hl, self.a);
        hl = hl.wrapping_add(1);
        self.set_hl(hl);
        2
    }

    fn ld_imm_16(&mut self, register_to: Register16, imm_value: u16) -> u64 {
        self.set_register_16(register_to, imm_value);
        3
    }

    fn ld_to_imm_ind_from_sp(&mut self, imm_value: u16, memory: &mut Memory) -> u64 {
        memory.write_to_16b(imm_value, self.sp);
        5
    }

    fn ld_sp_from_hl(&mut self) -> u64 {
        self.sp = self.get_hl();
        2
    }

    fn push(&mut self, register_from: Register16, memory: &mut Memory) -> u64 {
        let register_value = self.get_register_16(register_from);
        self.push_to_stack_16b(register_value, memory);
        4
    }

    fn pop(&mut self, register: Register16, memory: &mut Memory) -> u64 {
        let stack_value = self.pop_from_stack_16b(memory);
        self.set_register_16(register, stack_value);
        3
    }

    fn ld_hl_from_adjusted_sp(&mut self, imm: i8) -> u64 {
        let adjusted_sp = self.sp.wrapping_add_signed(imm as i8 as i16);
        self.set_hl(adjusted_sp);

        let sp = self.sp;

        let half_carry = (sp & 0x000f).wrapping_add(((imm as u8) & 0x0f) as u16) > 0x000f;
        let did_overflow = (sp & 0x00ff).wrapping_add(imm as u8 as u16) > 0x00ff;

        self.set_f_flags(did_overflow, half_carry, false, false);
        3
    }

    fn _add(&mut self, value: u8) {
        let (new_value, did_overflow) = self.a.overflowing_add(value);
        let half_carry = (self.a & 0x0f) + (value & 0x0f) > 0x0f;

        self.set_f_flags(did_overflow, half_carry, false, new_value == 0);

        self.a = new_value;
    }

    fn _adc(&mut self, value: u8) {
        let carry_flag = if bit_utils::get_bit(self.f, Cpu::F_CARRY_FLAG_POS) {
            1
        } else {
            0
        };

        let (first_add, did_overflow1) = self.a.overflowing_add(value);
        let (second_add, did_overflow2) = first_add.overflowing_add(carry_flag);

        let half_carry = (self.a & 0x0f) + (value & 0x0f) + carry_flag > 0x0f;
        let did_overflow = did_overflow1 || did_overflow2;

        self.set_f_flags(did_overflow, half_carry, false, second_add == 0);

        self.a = second_add;
    }

    fn add(&mut self, register: Register) -> u64 {
        let value = self.get_register(register);
        self._add(value);
        1
    }

    fn add_hl_ind(&mut self, memory: &mut Memory) -> u64 {
        let value = memory.read_from_8b(self.get_hl());
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

    fn add_c_hl_ind(&mut self, memory: &mut Memory) -> u64 {
        let value = memory.read_from_8b(self.get_hl());
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

        self.set_f_flags(did_borrow, half_carry, true, new_register_value == 0);

        self.a = new_register_value;
    }

    fn _sbc(&mut self, value: u8) {
        let carry_flag = if bit_utils::get_bit(self.f, Cpu::F_CARRY_FLAG_POS) {
            1
        } else {
            0
        };

        let (fist_sub, did_borrow1) = self.a.overflowing_sub(value);
        let (second_sub, did_borrow2) = fist_sub.overflowing_sub(carry_flag);

        let half_carry = self.a & 0x0f < (value & 0x0f) + carry_flag;
        let did_borrow = did_borrow1 || did_borrow2;

        self.set_f_flags(did_borrow, half_carry, true, second_sub == 0);

        self.a = second_sub;
    }

    fn sub(&mut self, register: Register) -> u64 {
        let value = self.get_register(register);
        self._sub(value);
        1
    }

    fn sub_hl_ind(&mut self, memory: &mut Memory) -> u64 {
        let value = memory.read_from_8b(self.get_hl());
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

    fn sub_c_hl_ind(&mut self, memory: &mut Memory) -> u64 {
        let value = memory.read_from_8b(self.get_hl());
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

        self.set_f_flags(did_borrow, half_carry, true, new_register_value == 0);
    }

    fn cp(&mut self, register: Register) -> u64 {
        let value = self.get_register(register);
        self._cp(value);
        1
    }

    fn cp_hl_ind(&mut self, memory: &mut Memory) -> u64 {
        let value = memory.read_from_8b(self.get_hl());
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
        let current_carry = self.get_register_bit(Register::F, Cpu::F_CARRY_FLAG_POS);

        self.set_f_flags(current_carry, half_carry, false, new_value == 0);

        self.set_register(register, new_value);
        1
    }

    fn inc_hl_ind(&mut self, memory: &mut Memory) -> u64 {
        let value = memory.read_from_8b(self.get_hl());

        let (new_value, _) = value.overflowing_add(1);
        let half_carry = (value & 0x0f) + 0b1 > 0x0f;
        let current_carry = self.get_register_bit(Register::F, Cpu::F_CARRY_FLAG_POS);

        self.set_f_flags(current_carry, half_carry, false, new_value == 0);

        memory.write_to_8b(self.get_hl(), new_value);
        3
    }

    fn dec(&mut self, register: Register) -> u64 {
        let value = self.get_register(register.clone());

        let (new_value, _) = value.overflowing_sub(1);
        let half_carry = (value & 0x0f) == 0;
        let current_carry = self.get_register_bit(Register::F, Cpu::F_CARRY_FLAG_POS);

        self.set_f_flags(current_carry, half_carry, true, new_value == 0);

        self.set_register(register, new_value);
        1
    }

    fn dec_hl_ind(&mut self, memory: &mut Memory) -> u64 {
        let value = memory.read_from_8b(self.get_hl());

        let (new_value, _) = value.overflowing_sub(1);
        let half_carry = (value & 0x0f) == 0;
        let current_carry = self.get_register_bit(Register::F, Cpu::F_CARRY_FLAG_POS);

        self.set_f_flags(current_carry, half_carry, true, new_value == 0);

        memory.write_to_8b(self.get_hl(), new_value);
        3
    }

    fn _and(&mut self, value: u8) {
        let new_value = self.a & value;

        self.set_f_flags(false, true, false, new_value == 0);

        self.a = new_value;
    }

    fn and(&mut self, register: Register) -> u64 {
        let value = self.get_register(register);
        self._and(value);
        1
    }

    fn and_hl_ind(&mut self, memory: &mut Memory) -> u64 {
        let value = memory.read_from_8b(self.get_hl());
        self._and(value);
        2
    }

    fn and_imm(&mut self, imm: u8) -> u64 {
        self._and(imm);
        2
    }

    fn _or(&mut self, value: u8) {
        let new_value = self.a | value;

        self.set_f_flags(false, false, false, new_value == 0);

        self.a = new_value;
    }

    fn or(&mut self, register: Register) -> u64 {
        let value = self.get_register(register);
        self._or(value);
        1
    }

    fn or_hl_ind(&mut self, memory: &mut Memory) -> u64 {
        let value = memory.read_from_8b(self.get_hl());
        self._or(value);
        2
    }

    fn or_imm(&mut self, imm: u8) -> u64 {
        self._or(imm);
        2
    }

    fn _xor(&mut self, value: u8) {
        let new_value = self.a ^ value;

        self.set_f_flags(false, false, false, new_value == 0);

        self.a = new_value;
    }

    fn xor(&mut self, register: Register) -> u64 {
        let value = self.get_register(register);
        self._xor(value);
        1
    }

    fn xor_hl_ind(&mut self, memory: &mut Memory) -> u64 {
        let value = memory.read_from_8b(self.get_hl());
        self._xor(value);
        2
    }

    fn xor_imm(&mut self, imm: u8) -> u64 {
        self._xor(imm);
        2
    }

    fn ccf(&mut self) -> u64 {
        let current_carry = self.get_register_bit(Register::F, Cpu::F_CARRY_FLAG_POS);
        let current_zero = self.get_register_bit(Register::F, Cpu::F_ZERO_FLAG_POS);

        self.set_f_flags(!current_carry, false, false, current_zero);
        1
    }

    fn scf(&mut self) -> u64 {
        let current_zero = self.get_register_bit(Register::F, Cpu::F_ZERO_FLAG_POS);
        self.set_f_flags(true, false, false, current_zero);
        1
    }

    fn daa(&mut self) -> u64 {
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
        1
    }

    fn cpl(&mut self) -> u64 {
        let current_carry = self.get_register_bit(Register::F, Cpu::F_CARRY_FLAG_POS);
        let current_zero = self.get_register_bit(Register::F, Cpu::F_ZERO_FLAG_POS);

        self.set_f_flags(current_carry, true, true, current_zero);

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
        let current_zero = self.get_register_bit(Register::F, Cpu::F_ZERO_FLAG_POS);
        let half_carry = (hl & 0x0fff) + (register_value & 0x0fff) > 0x0fff;

        self.set_f_flags(did_overflow, half_carry, false, current_zero);

        self.set_hl(new_value);
        2
    }

    fn add_sp_imm(&mut self, imm: u8) -> u64 {
        let new_value = self.sp.wrapping_add_signed(imm as i8 as i16);

        let sp = self.sp;

        let half_carry = (sp & 0x000f).wrapping_add(((imm as u8) & 0x0f) as u16) > 0x000f;
        let did_overflow = (sp & 0x00ff).wrapping_add(imm as u8 as u16) > 0x00ff;

        self.set_f_flags(did_overflow, half_carry, false, false);

        self.sp = new_value;
        4
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

    fn rlc_hl_ind(&mut self, memory: &mut Memory) -> u64 {
        let register_value = memory.read_from_8b(self.get_hl());

        let new_value = self._rotate(register_value, true, false, false);

        memory.write_to_8b(self.get_hl(), new_value);
        4
    }

    fn rrcr(&mut self, register: Register) -> u64 {
        let register_value = self.get_register(register.clone());

        let new_value = self._rotate(register_value, true, true, false);

        self.set_register(register, new_value);
        2
    }

    fn rrc_hl_ind(&mut self, memory: &mut Memory) -> u64 {
        let register_value = memory.read_from_8b(self.get_hl());

        let new_value = self._rotate(register_value, true, true, false);

        memory.write_to_8b(self.get_hl(), new_value);
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

    fn rr_hl_ind(&mut self, memory: &mut Memory) -> u64 {
        let register_value = memory.read_from_8b(self.get_hl());

        let new_value = self._rotate(register_value, false, true, false);

        memory.write_to_8b(self.get_hl(), new_value);
        4
    }

    fn rl_hl_ind(&mut self, memory: &mut Memory) -> u64 {
        let register_value = memory.read_from_8b(self.get_hl());

        let new_value = self._rotate(register_value, false, false, false);

        memory.write_to_8b(self.get_hl(), new_value);
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

        self.set_f_flags(carry, false, false, new_value == 0);

        new_value
    }

    fn srar(&mut self, register: Register) -> u64 {
        let register_value = self.get_register(register.clone());

        let new_value = self._sa(register_value, true);

        self.set_register(register, new_value);
        2
    }

    fn sra_hl_ind(&mut self, memory: &mut Memory) -> u64 {
        let register_value = memory.read_from_8b(self.get_hl());

        let new_value = self._sa(register_value, true);

        memory.write_to_8b(self.get_hl(), new_value);
        4
    }

    fn slar(&mut self, register: Register) -> u64 {
        let register_value = self.get_register(register.clone());

        let new_value = self._sa(register_value, false);

        self.set_register(register, new_value);
        2
    }

    fn sla_hl_ind(&mut self, memory: &mut Memory) -> u64 {
        let register_value = memory.read_from_8b(self.get_hl());

        let new_value = self._sa(register_value, false);

        memory.write_to_8b(self.get_hl(), new_value);
        4
    }

    fn _sl(&mut self, value: u8, right: bool) -> u8 {
        let msb = value & 0x80;
        let lsb = value & 0x01;

        let carry = if right { lsb != 0 } else { msb != 0 };
        let new_value = if right { value >> 1 } else { value << 1 };

        self.set_f_flags(carry, false, false, new_value == 0);

        new_value
    }

    fn srlr(&mut self, register: Register) -> u64 {
        let register_value = self.get_register(register.clone());

        let new_value = self._sl(register_value, true);

        self.set_register(register, new_value);
        2
    }

    fn srl_hl_ind(&mut self, memory: &mut Memory) -> u64 {
        let register_value = memory.read_from_8b(self.get_hl());

        let new_value = self._sl(register_value, true);

        memory.write_to_8b(self.get_hl(), new_value);
        4
    }

    fn _swap(&mut self, value: u8) -> u8 {
        let new_value = value.rotate_left(4);

        self.set_f_flags(false, false, false, new_value == 0);

        new_value
    }

    fn swapr(&mut self, register: Register) -> u64 {
        let register_value = self.get_register(register.clone());

        let new_value = self._swap(register_value);

        self.set_register(register, new_value);
        2
    }

    fn swap_hl_ind(&mut self, memory: &mut Memory) -> u64 {
        let register_value = memory.read_from_8b(self.get_hl());

        let new_value = self._swap(register_value);

        memory.write_to_8b(self.get_hl(), new_value);
        4
    }

    fn _bit(&mut self, bit_position: u8, value: u8) {
        let old_carry = self.get_register_bit(Register::F, Cpu::F_CARRY_FLAG_POS);

        let bit_is_set = value & (1 << bit_position);

        self.set_f_flags(old_carry, true, false, bit_is_set == 0);
    }

    fn bitr(&mut self, bit_position: u8, register: Register) -> u64 {
        let register_value = self.get_register(register);
        self._bit(bit_position, register_value);
        2
    }

    fn bit_hl_ind(&mut self, bit_position: u8, memory: &mut Memory) -> u64 {
        let register_value = memory.read_from_8b(self.get_hl());
        self._bit(bit_position, register_value);
        3
    }

    fn setr(&mut self, bit_position: u8, register: Register) -> u64 {
        let register_value = self.get_register(register.clone());
        let new_value = bit_utils::modify_bit(register_value, bit_position, true);
        self.set_register(register, new_value);
        2
    }

    fn set_hl_ind(&mut self, bit_position: u8, memory: &mut Memory) -> u64 {
        let register_value = memory.read_from_8b(self.get_hl());
        let new_value = bit_utils::modify_bit(register_value, bit_position, true);
        memory.write_to_8b(self.get_hl(), new_value);
        4
    }

    fn resetr(&mut self, bit_position: u8, register: Register) -> u64 {
        let register_value = self.get_register(register.clone());
        let new_value = bit_utils::modify_bit(register_value, bit_position, false);
        self.set_register(register, new_value);
        2
    }

    fn reset_hl_ind(&mut self, bit_position: u8, memory: &mut Memory) -> u64 {
        let register_value = memory.read_from_8b(self.get_hl());
        let new_value = bit_utils::modify_bit(register_value, bit_position, false);
        memory.write_to_8b(self.get_hl(), new_value);
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

    fn jr(&mut self, imm: u8) -> u64 {
        self.pc = self.pc.wrapping_add_signed(imm as i8 as i16);
        3
    }

    fn jr_cc(&mut self, cond: FlowCondition, imm: u8) -> u64 {
        if self.evaluate_flow_condition(cond) {
            self.jr(imm);
            3
        } else {
            2
        }
    }

    fn call(&mut self, addr: u16, memory: &mut Memory) -> u64 {
        self.push_to_stack_16b(self.pc, memory);
        self.pc = addr;
        6
    }

    fn call_cc(&mut self, cond: FlowCondition, addr: u16, memory: &mut Memory) -> u64 {
        if self.evaluate_flow_condition(cond) {
            self.call(addr, memory);
            6
        } else {
            3
        }
    }

    fn ret(&mut self, memory: &mut Memory) -> u64 {
        self.pc = self.pop_from_stack_16b(memory);
        4
    }

    fn ret_cc(&mut self, cond: FlowCondition, memory: &mut Memory) -> u64 {
        if self.evaluate_flow_condition(cond) {
            self.ret(memory);
            5
        } else {
            2
        }
    }

    fn ret_i(&mut self, memory: &mut Memory) -> u64 {
        self.ret(memory);
        self.ei();
        4
    }

    fn rst(&mut self, addr: u8, memory: &mut Memory) -> u64 {
        self.push_to_stack_16b(self.pc, memory);
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

    pub fn clock(&mut self, memory: &mut Memory) {

        if self.halted {
            return;
         }

        let (instruction, size) = self.decode(memory);

        self.step(size);

        let cycles = self.execute(instruction, memory);

        self.clock = self.clock.wrapping_add(cycles);
    }

    fn decode(&self, memory: &mut Memory) -> (Instruction, u16) {
        let opcode = memory.read_from_8b(self.pc);

        match opcode {
            0xcb => self.decode_cb(memory),
            _ => self.decode_generic(memory),
        }
    }

    fn decode_cb(&self, memory: &mut Memory) -> (Instruction, u16) {
        let mut helper_pc = self.pc + 1; // First byte is always 0xCB

        let byte = memory.read_from_8b(helper_pc);

        match (byte & 0b11000000) >> 6 {
            0b00 => match (byte & 0b00111000) >> 3 {
                0b000 => {
                    let operand =
                        Self::get_operand_from_opcode(byte, OperandType::R8Operand, false, true);
                    let r8_operand = R8Operand::from_byte(operand);

                    match r8_operand {
                        R8Operand::B => (Instruction::RLCR(Register::B), 2),
                        R8Operand::C => (Instruction::RLCR(Register::C), 2),
                        R8Operand::D => (Instruction::RLCR(Register::D), 2),
                        R8Operand::E => (Instruction::RLCR(Register::E), 2),
                        R8Operand::H => (Instruction::RLCR(Register::H), 2),
                        R8Operand::A => (Instruction::RLCR(Register::A), 2),
                        R8Operand::L => (Instruction::RLCR(Register::L), 2),
                        R8Operand::HLInd => (Instruction::RLCHLInd(), 2),
                    }
                }
                0b001 => {
                    let operand =
                        Self::get_operand_from_opcode(byte, OperandType::R8Operand, false, true);
                    let r8_operand = R8Operand::from_byte(operand);

                    match r8_operand {
                        R8Operand::B => (Instruction::RRCR(Register::B), 2),
                        R8Operand::C => (Instruction::RRCR(Register::C), 2),
                        R8Operand::D => (Instruction::RRCR(Register::D), 2),
                        R8Operand::E => (Instruction::RRCR(Register::E), 2),
                        R8Operand::H => (Instruction::RRCR(Register::H), 2),
                        R8Operand::A => (Instruction::RRCR(Register::A), 2),
                        R8Operand::L => (Instruction::RRCR(Register::L), 2),
                        R8Operand::HLInd => (Instruction::RRCHLInd(), 2),
                    }
                }
                0b010 => {
                    let operand =
                        Self::get_operand_from_opcode(byte, OperandType::R8Operand, false, true);
                    let r8_operand = R8Operand::from_byte(operand);

                    match r8_operand {
                        R8Operand::B => (Instruction::RLR(Register::B), 2),
                        R8Operand::C => (Instruction::RLR(Register::C), 2),
                        R8Operand::D => (Instruction::RLR(Register::D), 2),
                        R8Operand::E => (Instruction::RLR(Register::E), 2),
                        R8Operand::H => (Instruction::RLR(Register::H), 2),
                        R8Operand::A => (Instruction::RLR(Register::A), 2),
                        R8Operand::L => (Instruction::RLR(Register::L), 2),
                        R8Operand::HLInd => (Instruction::RLHLInd(), 2),
                    }
                }
                0b011 => {
                    let operand =
                        Self::get_operand_from_opcode(byte, OperandType::R8Operand, false, true);
                    let r8_operand = R8Operand::from_byte(operand);

                    match r8_operand {
                        R8Operand::B => (Instruction::RRR(Register::B), 2),
                        R8Operand::C => (Instruction::RRR(Register::C), 2),
                        R8Operand::D => (Instruction::RRR(Register::D), 2),
                        R8Operand::E => (Instruction::RRR(Register::E), 2),
                        R8Operand::H => (Instruction::RRR(Register::H), 2),
                        R8Operand::A => (Instruction::RRR(Register::A), 2),
                        R8Operand::L => (Instruction::RRR(Register::L), 2),
                        R8Operand::HLInd => (Instruction::RRHLInd(), 2),
                    }
                }
                0b100 => {
                    let operand =
                        Self::get_operand_from_opcode(byte, OperandType::R8Operand, false, true);
                    let r8_operand = R8Operand::from_byte(operand);

                    match r8_operand {
                        R8Operand::B => (Instruction::SLAR(Register::B), 2),
                        R8Operand::C => (Instruction::SLAR(Register::C), 2),
                        R8Operand::D => (Instruction::SLAR(Register::D), 2),
                        R8Operand::E => (Instruction::SLAR(Register::E), 2),
                        R8Operand::H => (Instruction::SLAR(Register::H), 2),
                        R8Operand::A => (Instruction::SLAR(Register::A), 2),
                        R8Operand::L => (Instruction::SLAR(Register::L), 2),
                        R8Operand::HLInd => (Instruction::SLAHLInd(), 2),
                    }
                }
                0b101 => {
                    let operand =
                        Self::get_operand_from_opcode(byte, OperandType::R8Operand, false, true);
                    let r8_operand = R8Operand::from_byte(operand);

                    match r8_operand {
                        R8Operand::B => (Instruction::SRAR(Register::B), 2),
                        R8Operand::C => (Instruction::SRAR(Register::C), 2),
                        R8Operand::D => (Instruction::SRAR(Register::D), 2),
                        R8Operand::E => (Instruction::SRAR(Register::E), 2),
                        R8Operand::H => (Instruction::SRAR(Register::H), 2),
                        R8Operand::A => (Instruction::SRAR(Register::A), 2),
                        R8Operand::L => (Instruction::SRAR(Register::L), 2),
                        R8Operand::HLInd => (Instruction::SRAHLInd(), 2),
                    }
                }
                0b110 => {
                    let operand =
                        Self::get_operand_from_opcode(byte, OperandType::R8Operand, false, true);
                    let r8_operand = R8Operand::from_byte(operand);

                    match r8_operand {
                        R8Operand::B => (Instruction::SWAPR(Register::B), 2),
                        R8Operand::C => (Instruction::SWAPR(Register::C), 2),
                        R8Operand::D => (Instruction::SWAPR(Register::D), 2),
                        R8Operand::E => (Instruction::SWAPR(Register::E), 2),
                        R8Operand::H => (Instruction::SWAPR(Register::H), 2),
                        R8Operand::A => (Instruction::SWAPR(Register::A), 2),
                        R8Operand::L => (Instruction::SWAPR(Register::L), 2),
                        R8Operand::HLInd => (Instruction::SWAPHLInd(), 2),
                    }
                }
                0b111 => {
                    let operand =
                        Self::get_operand_from_opcode(byte, OperandType::R8Operand, false, true);
                    let r8_operand = R8Operand::from_byte(operand);

                    match r8_operand {
                        R8Operand::B => (Instruction::SRLR(Register::B), 2),
                        R8Operand::C => (Instruction::SRLR(Register::C), 2),
                        R8Operand::D => (Instruction::SRLR(Register::D), 2),
                        R8Operand::E => (Instruction::SRLR(Register::E), 2),
                        R8Operand::H => (Instruction::SRLR(Register::H), 2),
                        R8Operand::A => (Instruction::SRLR(Register::A), 2),
                        R8Operand::L => (Instruction::SRLR(Register::L), 2),
                        R8Operand::HLInd => (Instruction::SRLHLInd(), 2),
                    }
                }
                _ => panic!("Unknown CB instruction {}", byte),
            },
            0b01 => {
                let bit_index =
                    Self::get_operand_from_opcode(byte, OperandType::BitIndex, false, true);
                let operand =
                    Self::get_operand_from_opcode(byte, OperandType::R8Operand, false, true);

                let r8_operand = R8Operand::from_byte(operand);

                match r8_operand {
                    R8Operand::B => (Instruction::BITR(bit_index, Register::B), 2),
                    R8Operand::C => (Instruction::BITR(bit_index, Register::C), 2),
                    R8Operand::D => (Instruction::BITR(bit_index, Register::D), 2),
                    R8Operand::E => (Instruction::BITR(bit_index, Register::E), 2),
                    R8Operand::H => (Instruction::BITR(bit_index, Register::H), 2),
                    R8Operand::A => (Instruction::BITR(bit_index, Register::A), 2),
                    R8Operand::L => (Instruction::BITR(bit_index, Register::L), 2),
                    R8Operand::HLInd => (Instruction::BITHLInd(bit_index), 2),
                }
            }
            0b10 => {
                let bit_index =
                    Self::get_operand_from_opcode(byte, OperandType::BitIndex, false, true);
                let operand =
                    Self::get_operand_from_opcode(byte, OperandType::R8Operand, false, true);

                let r8_operand = R8Operand::from_byte(operand);

                match r8_operand {
                    R8Operand::B => (Instruction::RESETR(bit_index, Register::B), 2),
                    R8Operand::C => (Instruction::RESETR(bit_index, Register::C), 2),
                    R8Operand::D => (Instruction::RESETR(bit_index, Register::D), 2),
                    R8Operand::E => (Instruction::RESETR(bit_index, Register::E), 2),
                    R8Operand::H => (Instruction::RESETR(bit_index, Register::H), 2),
                    R8Operand::A => (Instruction::RESETR(bit_index, Register::A), 2),
                    R8Operand::L => (Instruction::RESETR(bit_index, Register::L), 2),
                    R8Operand::HLInd => (Instruction::RESETHLInd(bit_index), 2),
                }
            }
            0b11 => {
                let bit_index =
                    Self::get_operand_from_opcode(byte, OperandType::BitIndex, false, true);
                let operand =
                    Self::get_operand_from_opcode(byte, OperandType::R8Operand, false, true);

                let r8_operand = R8Operand::from_byte(operand);

                match r8_operand {
                    R8Operand::B => (Instruction::SETR(bit_index, Register::B), 2),
                    R8Operand::C => (Instruction::SETR(bit_index, Register::C), 2),
                    R8Operand::D => (Instruction::SETR(bit_index, Register::D), 2),
                    R8Operand::E => (Instruction::SETR(bit_index, Register::E), 2),
                    R8Operand::H => (Instruction::SETR(bit_index, Register::H), 2),
                    R8Operand::A => (Instruction::SETR(bit_index, Register::A), 2),
                    R8Operand::L => (Instruction::SETR(bit_index, Register::L), 2),
                    R8Operand::HLInd => (Instruction::SETHLInd(bit_index), 2),
                }
            }
            _ => panic!("Unknown instruction: 0xCB {}", byte),
        }
    }

    fn decode_generic(&self, memory: &mut Memory) -> (Instruction, u16) {
        let mut helper_pc = self.pc;

        let opcode = memory.read_from_8b(helper_pc);

        match (opcode & 0b11000000) >> 6 {
            0b00 => self.decode_generic_block_0(opcode, memory),
            0b01 => self.decode_generic_block_1(opcode, memory),
            0b10 => self.decode_generic_block_2(opcode, memory),
            0b11 => self.decode_generic_block_3(opcode, memory),
            _ => panic!("Uknown block")
        }
    }

    fn decode_generic_block_0(&self, opcode: u8, memory: &mut Memory) -> (Instruction, u16) {
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
            0b0001_1000 => {
                let imm = self.extract_8b_operand(memory);
                (Instruction::JR(imm), 2)
            },
            0b0000_1000 => {
                let imm = self.extract_16b_operand(memory);
                (Instruction::LDToImmIndFromSP(imm), 3)
            },
            opcode if (opcode & 0b1110_0111) == 0b0010_0000 => {
                let imm = self.extract_8b_operand(memory);
                let operand = Self::get_operand_from_opcode(opcode, OperandType::FlowCondition, false, false);
                let cond_operand = FlowCondition::from_byte(operand);
                (Instruction::JRCC(cond_operand, imm), 2)
            },
            opcode if (opcode & 0b1100_0111) == 0b0000_0110 => {
                let imm = self.extract_8b_operand(memory);
                let operand = Self::get_operand_from_opcode(opcode, OperandType::R8Operand, false, false);
                let r8_operand = R8Operand::from_byte(operand);
                
                match r8_operand {
                    R8Operand::B => (Instruction::LDImm(Register::B, imm), 2),
                    R8Operand::C => (Instruction::LDImm(Register::C, imm), 2),
                    R8Operand::D => (Instruction::LDImm(Register::D, imm), 2),
                    R8Operand::E => (Instruction::LDImm(Register::E, imm), 2),
                    R8Operand::H => (Instruction::LDImm(Register::H, imm), 2),
                    R8Operand::A => (Instruction::LDImm(Register::A, imm), 2),
                    R8Operand::L => (Instruction::LDImm(Register::L, imm), 2),
                    R8Operand::HLInd => (Instruction::LDToHlIndImm(imm), 2),
                }
            },
            opcode if (opcode & 0b1100_0111) == 0b0000_0100 => {
                let operand = Self::get_operand_from_opcode(opcode, OperandType::R8Operand, false, false);
                let r8_operand = R8Operand::from_byte(operand);

                match r8_operand {
                    R8Operand::B => (Instruction::INC(Register::B), 1),
                    R8Operand::C => (Instruction::INC(Register::C), 1),
                    R8Operand::D => (Instruction::INC(Register::D), 1),
                    R8Operand::E => (Instruction::INC(Register::E), 1),
                    R8Operand::H => (Instruction::INC(Register::H), 1),
                    R8Operand::A => (Instruction::INC(Register::A), 1),
                    R8Operand::L => (Instruction::INC(Register::L), 1),
                    R8Operand::HLInd => (Instruction::INCHLInd(), 1),
                }
            },
            opcode if (opcode & 0b1100_0111) == 0b0000_0101 => {
                let operand = Self::get_operand_from_opcode(opcode, OperandType::R8Operand, false, false);
                let r8_operand = R8Operand::from_byte(operand);

                match r8_operand {
                    R8Operand::B => (Instruction::DEC(Register::B), 1),
                    R8Operand::C => (Instruction::DEC(Register::C), 1),
                    R8Operand::D => (Instruction::DEC(Register::D), 1),
                    R8Operand::E => (Instruction::DEC(Register::E), 1),
                    R8Operand::H => (Instruction::DEC(Register::H), 1),
                    R8Operand::A => (Instruction::DEC(Register::A), 1),
                    R8Operand::L => (Instruction::DEC(Register::L), 1),
                    R8Operand::HLInd => (Instruction::DECHLInd(), 1),
                }
            },
            opcode if (opcode & 0b1100_1111) == 0b0000_0011 => {
                let operand = Self::get_operand_from_opcode(opcode, OperandType::R16Operand, false, false);
                let r16_operand = R16Operand::from_byte(operand);

                match r16_operand {
                    R16Operand::BC => (Instruction::INC16(Register16::BC), 1),
                    R16Operand::DE => (Instruction::INC16(Register16::DE), 1),
                    R16Operand::HL => (Instruction::INC16(Register16::HL), 1),
                    R16Operand::SP => (Instruction::INC16(Register16::SP), 1),
                }
            },
            opcode if (opcode & 0b1100_1111) == 0b0000_1011 => {
                let operand = Self::get_operand_from_opcode(opcode, OperandType::R16Operand, false, false);
                let r16_operand = R16Operand::from_byte(operand);

                match r16_operand {
                    R16Operand::BC => (Instruction::DEC16(Register16::BC), 1),
                    R16Operand::DE => (Instruction::DEC16(Register16::DE), 1),
                    R16Operand::HL => (Instruction::DEC16(Register16::HL), 1),
                    R16Operand::SP => (Instruction::DEC16(Register16::SP), 1),
                }
            },
            opcode if (opcode & 0b1100_1111) == 0b0000_1001 => {
                let operand = Self::get_operand_from_opcode(opcode, OperandType::R16Operand, false, false);
                let r16_operand = R16Operand::from_byte(operand);

                match r16_operand {
                    R16Operand::BC => (Instruction::ADDHL(Register16::BC), 1),
                    R16Operand::DE => (Instruction::ADDHL(Register16::DE), 1),
                    R16Operand::HL => (Instruction::ADDHL(Register16::HL), 1),
                    R16Operand::SP => (Instruction::ADDHL(Register16::SP), 1),
                }
            },
            opcode if (opcode & 0b1100_1111) == 0b0000_0001 => {
                let imm = self.extract_16b_operand(memory);
                let operand = Self::get_operand_from_opcode(opcode, OperandType::R16Operand, false, false);
                let r16_operand = R16Operand::from_byte(operand);

                match r16_operand {
                    R16Operand::BC => (Instruction::LDImm16(Register16::BC, imm), 3),
                    R16Operand::DE => (Instruction::LDImm16(Register16::DE, imm), 3),
                    R16Operand::HL => (Instruction::LDImm16(Register16::HL, imm), 3),
                    R16Operand::SP => (Instruction::LDImm16(Register16::SP, imm), 3),
                }
            },
            opcode if (opcode & 0b1100_1111) == 0b0000_0010 => {
                let operand = Self::get_operand_from_opcode(opcode, OperandType::R16MemOperand, false, false);
                let r16mem_operand = R16MemOperand::from_byte(operand);

                match r16mem_operand {
                    R16MemOperand::BC  => (Instruction::LDToBCIndFromA(), 1),
                    R16MemOperand::DE  => (Instruction::LDToDEIndFromA(), 1),
                    R16MemOperand::HLI => (Instruction::LDToHLIndIncFromA(), 1),
                    R16MemOperand::HLD => (Instruction::LDToHLIndDecFromA(), 1),
                }
            },
            opcode if (opcode & 0b1100_1111) == 0b0000_1010 => {
                let operand = Self::get_operand_from_opcode(opcode, OperandType::R16MemOperand, false, false);
                let r16mem_operand = R16MemOperand::from_byte(operand);

                match r16mem_operand {
                    R16MemOperand::BC  => (Instruction::LDFromBCIndToA(), 1),
                    R16MemOperand::DE  => (Instruction::LDFromDEIndToA(), 1),
                    R16MemOperand::HLI => (Instruction::LDFromHLIndIncToA(), 1),
                    R16MemOperand::HLD => (Instruction::LDFromHLIndDecToA(), 1),
                }
            },
            _ => panic!("Unknown instruction Block Zero Hex: {:#x} | Binary: {:#b}", opcode, opcode)
        }
    }

    fn decode_generic_block_1(&self, opcode: u8, memory: &mut Memory) -> (Instruction, u16) {
        match opcode {
            _ => (Instruction::NOP(), 1)
        }
    }

    fn decode_generic_block_2(&self, opcode: u8, memory: &mut Memory) -> (Instruction, u16) {
        match opcode {
            _ => (Instruction::NOP(), 1)
        }
    }

    fn decode_generic_block_3(&self, opcode: u8, memory: &mut Memory) -> (Instruction, u16) {
        match opcode {
            _ => (Instruction::NOP(), 1)
        }
    }

    fn step(&mut self, instruction_size: u16) {
        self.pc = self.pc.wrapping_add(instruction_size);
    }

    fn execute(&mut self, instruction: Instruction, memory: &mut Memory) -> u64 {
        match instruction {
            Instruction::LD(register_to, register_from) => self.ld(register_to, register_from),
            Instruction::LDImm(register_to, imm) => self.ld_imm(register_to, imm),
            Instruction::LDFromHLInd(register_to) => self.ld_from_hl_ind(register_to, memory),
            Instruction::LDToHLInd(register_from) => self.ld_to_hl_ind(register_from, memory),
            Instruction::LDToHlIndImm(imm) => self.ld_to_hl_ind_imm(imm, memory),
            Instruction::LDFromBCIndToA() => self.ld_from_bc_ind_to_a(memory),
            Instruction::LDFromDEIndToA() => self.ld_from_de_ind_to_a(memory),
            Instruction::LDToBCIndFromA() => self.ld_to_bc_ind_from_a(memory),
            Instruction::LDToDEIndFromA() => self.ld_to_de_ind_from_a(memory),
            Instruction::LDFromImmIndToA(imm) => self.ld_from_imm_ind_to_a(imm, memory),
            Instruction::LDToImmIndFromA(imm) => self.ld_to_imm_ind_from_a(imm, memory),
            Instruction::LDToAFromCInd() => self.ld_to_a_from_c_ind(memory),
            Instruction::LDFromAToCInd() => self.ld_from_a_to_c_ind(memory),
            Instruction::LDFromImmIndToA8(imm) => self.ld_from_imm_ind_to_a_8(imm, memory),
            Instruction::LDToImmIndFromA8(imm) => self.ld_to_imm_ind_from_a_8(imm, memory),
            Instruction::LDFromHLIndDecToA() => self.ld_from_hl_ind_dec_to_a(memory),
            Instruction::LDToHLIndDecFromA() => self.ld_to_hl_ind_dec_from_a(memory),
            Instruction::LDFromHLIndIncToA() => self.ld_from_hl_ind_inc_to_a(memory),
            Instruction::LDToHLIndIncFromA() => self.ld_to_hl_ind_inc_from_a(memory),
            Instruction::LDImm16(register_to, imm) => self.ld_imm_16(register_to, imm),
            Instruction::LDToImmIndFromSP(imm) => self.ld_to_imm_ind_from_sp(imm, memory),
            Instruction::LDSPFromHL() => self.ld_sp_from_hl(),
            Instruction::PUSH(register_from) => self.push(register_from, memory),
            Instruction::POP(register_from) => self.pop(register_from, memory),
            Instruction::LDHLFromAdjustedSP(imm) => self.ld_hl_from_adjusted_sp(imm),
            Instruction::ADD(register) => self.add(register),
            Instruction::ADDHLInd() => self.add_hl_ind(memory),
            Instruction::ADDImm(imm) => self.add_imm(imm),
            Instruction::ADDC(register) => self.add_c(register),
            Instruction::ADDCHLInd() => self.add_c_hl_ind(memory),
            Instruction::ADDCImm(imm) => self.add_c_imm(imm),
            Instruction::SUB(register) => self.sub(register),
            Instruction::SUBHLInd() => self.sub_hl_ind(memory),
            Instruction::SUBImm(imm) => self.sub_imm(imm),
            Instruction::SUBC(register) => self.sub_c(register),
            Instruction::SUBCHLInd() => self.sub_c_hl_ind(memory),
            Instruction::SUBCImm(imm) => self.sub_c_imm(imm),
            Instruction::CP(register) => self.cp(register),
            Instruction::CPHLInd() => self.cp_hl_ind(memory),
            Instruction::CPImm(imm) => self.cp_imm(imm),
            Instruction::INC(register) => self.inc(register),
            Instruction::INCHLInd() => self.inc_hl_ind(memory),
            Instruction::DEC(register) => self.dec(register),
            Instruction::DECHLInd() => self.dec_hl_ind(memory),
            Instruction::AND(register) => self.and(register),
            Instruction::ANDHLInd() => self.and_hl_ind(memory),
            Instruction::ANDImm(imm) => self.and_imm(imm),
            Instruction::OR(register) => self.or(register),
            Instruction::ORHLInd() => self.or_hl_ind(memory),
            Instruction::ORImm(imm) => self.or_imm(imm),
            Instruction::XOR(register) => self.xor(register),
            Instruction::XORHLInd() => self.xor_hl_ind(memory),
            Instruction::XORImm(imm) => self.xor_imm(imm),
            Instruction::CCF() => self.ccf(),
            Instruction::SCF() => self.scf(),
            Instruction::DAA() => self.daa(),
            Instruction::CPL() => self.cpl(),
            Instruction::INC16(register) => self.inc_16(register),
            Instruction::DEC16(register) => self.dec_16(register),
            Instruction::ADDHL(register) => self.add_hl(register),
            Instruction::ADDSPImm(imm) => self.add_sp_imm(imm),
            Instruction::RLCA() => self.rlca(),
            Instruction::RRCA() => self.rrca(),
            Instruction::RRA() => self.rra(),
            Instruction::RLA() => self.rla(),
            Instruction::RLCR(register) => self.rlcr(register),
            Instruction::RRCR(register) => self.rrcr(register),
            Instruction::RLCHLInd() => self.rlc_hl_ind(memory),
            Instruction::RRCHLInd() => self.rrc_hl_ind(memory),
            Instruction::RLR(register) => self.rlr(register),
            Instruction::RRR(register) => self.rrr(register),
            Instruction::RLHLInd() => self.rl_hl_ind(memory),
            Instruction::RRHLInd() => self.rr_hl_ind(memory),
            Instruction::SRAR(register) => self.srar(register),
            Instruction::SRAHLInd() => self.sra_hl_ind(memory),
            Instruction::SLAR(register) => self.slar(register),
            Instruction::SLAHLInd() => self.sla_hl_ind(memory),
            Instruction::SRLR(register) => self.srlr(register),
            Instruction::SRLHLInd() => self.srl_hl_ind(memory),
            Instruction::SWAPR(register) => self.swapr(register),
            Instruction::SWAPHLInd() => self.swap_hl_ind(memory),
            Instruction::BITR(bit, register) => self.bitr(bit, register),
            Instruction::BITHLInd(bit) => self.bit_hl_ind(bit, memory),
            Instruction::SETR(bit, register) => self.setr(bit, register),
            Instruction::SETHLInd(bit) => self.set_hl_ind(bit, memory),
            Instruction::RESETR(bit, register) => self.resetr(bit, register),
            Instruction::RESETHLInd(bit) => self.reset_hl_ind(bit,memory),
            Instruction::NOP() => self.nop(),
            Instruction::JP(addr) => self.jp(addr),
            Instruction::JPHL() => self.jp_hl(),
            Instruction::JPCC(cond, addr) => self.jp_cc(cond, addr),
            Instruction::JR(imm) => self.jr(imm),
            Instruction::JRCC(cond, imm) => self.jr_cc(cond, imm),
            Instruction::CALL(addr) => self.call(addr, memory),
            Instruction::CALLCC(cond, addr) => self.call_cc(cond, addr, memory),
            Instruction::RET() => self.ret(memory),
            Instruction::RETCC(cond) => self.ret_cc(cond, memory),
            Instruction::RETI() => self.ret_i(memory),
            Instruction::RST(addr) => self.rst(addr, memory),
            Instruction::HALT() => self.halt(),
            Instruction::STOP() => self.stop(),
            Instruction::DI() => self.di(),
            Instruction::EI() => self.ei()
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::console::cpu::cpu::*;
    use crate::console::cpu::instruction::R8Operand;
    use crate::console::utils::bit_utils;

    
    fn execute_cb_instruction(cpu: &mut Cpu, memory: &mut Memory, offset: &mut u16, subopcode: u8) {
        memory.write_to_8b(*offset, 0xCB);
        memory.write_to_8b(*offset + 1, subopcode);
        *offset += 2;
        cpu.clock(memory);
    }
    
    fn preload_hl(memory: &mut Memory, cpu: &Cpu, value: u8) {
        let hl_addr = cpu.get_register_16(Register16::HL);
        memory.write_to_8b(hl_addr, value);
    }
    
    fn carry_flag(cpu: &Cpu) -> bool {
        bit_utils::get_bit(cpu.get_register(Register::F), Cpu::F_CARRY_FLAG_POS)
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
        ($memory:expr, $addr:expr, $expected:expr) => {
            assert_eq!(
                $memory.read_from_8b($addr),
                $expected,
                "Memory at {:#04X} expected to be {:#04X}",
                $addr,
                $expected
            )
        };
    }
    
    #[test]
    fn test_cb_set_operations() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();
        let mut offset: u16 = 0;
    
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b11_001_000 | R8Operand::to_byte(R8Operand::E));
        assert_reg!(cpu, Register::E, 0b0000_0010);
    
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b11_000_000 | R8Operand::to_byte(R8Operand::D));
        assert_reg!(cpu, Register::D, 0b0000_0001);
    
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b11_011_000 | R8Operand::to_byte(R8Operand::A));
        assert_reg!(cpu, Register::A, 0b0000_1000);
    
        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut memory, &cpu, 0xCB);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b11_010_000 | R8Operand::to_byte(R8Operand::HLInd));
        assert_mem!(memory, hl_addr, 0xCB | 0b0000_0100);
    
        // Ensure other registers remain unchanged.
        assert_reg!(cpu, Register::A, 0b0000_1000);
        assert_reg!(cpu, Register::E, 0b0000_0010);
        assert_reg!(cpu, Register::D, 0b0000_0001);
    }
    
    #[test]
    fn test_cb_res_operations() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();
        let mut offset: u16 = 0;
    
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b11_001_000 | R8Operand::to_byte(R8Operand::E));
        assert_reg!(cpu, Register::E, 0b0000_0010);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b10_001_000 | R8Operand::to_byte(R8Operand::E));
        assert_reg!(cpu, Register::E, 0);
    
        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut memory, &cpu, 0xCB);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b10_011_000 | R8Operand::to_byte(R8Operand::HLInd));
        assert_mem!(memory, hl_addr, 0xC3);
    }
    
    #[test]
    fn test_cb_bit_operations() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();
        let mut offset: u16 = 0;
    
        cpu.set_register(Register::C, 0b0010_0000);
    
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b01_001_000 | R8Operand::to_byte(R8Operand::C));
        assert!(bit_utils::get_bit(cpu.get_register(Register::F), Cpu::F_ZERO_FLAG_POS),
                "BIT 1 in C should set the zero flag");
    
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b01_101_000 | R8Operand::to_byte(R8Operand::C));
        assert!(!bit_utils::get_bit(cpu.get_register(Register::F), Cpu::F_ZERO_FLAG_POS),
                "BIT 5 in C should reset the zero flag");
    
        preload_hl(&mut memory, &cpu, 0b0000_1000);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b01_011_000 | R8Operand::to_byte(R8Operand::HLInd));
        assert!(!bit_utils::get_bit(cpu.get_register(Register::F), Cpu::F_ZERO_FLAG_POS),
                "BIT 3 in [HL] should reset the zero flag");
    }
    
    #[test]
    fn test_cb_swap_operations() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();
        let mut offset: u16 = 0;
    
        cpu.set_register(Register::E, 0b1111_0000);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b00_110_000 | R8Operand::to_byte(R8Operand::E));
        assert_reg!(cpu, Register::E, 0b0000_1111);
    
        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut memory, &cpu, 0xCB);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b00_110_000 | R8Operand::to_byte(R8Operand::HLInd));
        assert_mem!(memory, hl_addr, 0xBC);
    }
    
    #[test]
    fn test_cb_srl_operations() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();
        let mut offset: u16 = 0;
    
        cpu.set_register(Register::B, 0b1111_0000);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b00_111_000 | R8Operand::to_byte(R8Operand::B));
        let original_lsb = (0b1111_0000 & 1) != 0;
        assert_reg!(cpu, Register::B, 0b0111_1000);
        assert_eq!(carry_flag(&cpu), original_lsb, "Carry flag should match original LSB");
    
        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut memory, &cpu, 0b1100_1011);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b00_111_000 | R8Operand::to_byte(R8Operand::HLInd));
        assert_mem!(memory, hl_addr, 0b0110_0101);
        assert!(carry_flag(&cpu), "SRL [HL] should set the carry flag");
    }
    
    #[test]
    fn test_cb_rrc_operations() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();
        let mut offset: u16 = 0;
        
        cpu.set_register(Register::E, 0b1111_0000);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b00_001_000 | R8Operand::to_byte(R8Operand::E));
        assert_reg!(cpu, Register::E, 0b0111_1000);
        assert!(!carry_flag(&cpu), "RRC on E should not set the carry flag");
    
        // Test RRC on memory pointed by HL.
        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut memory, &cpu, 0b1110_0101);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b00_001_000 | R8Operand::to_byte(R8Operand::HLInd));
        assert_mem!(memory, hl_addr, 0b1111_0010);
        assert!(carry_flag(&cpu), "RRC [HL] should set the carry flag");
    }
    
    #[test]
    fn test_cb_rlc_operations() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();
        let mut offset: u16 = 0;
    
        cpu.set_register(Register::A, 0b0000_1111);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b00_000_000 | R8Operand::to_byte(R8Operand::A));
        assert_reg!(cpu, Register::A, 0b0001_1110);
        assert!(!carry_flag(&cpu), "RLC on A should not set the carry flag");
    
        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut memory, &cpu, 0b0010_1111);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b00_000_000 | R8Operand::to_byte(R8Operand::HLInd));
        assert_mem!(memory, hl_addr, 0b0101_1110);
        assert!(!carry_flag(&cpu), "RLC [HL] should not set the carry flag");
    }
    
    #[test]
    fn test_cb_sra_operations() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();
        let mut offset: u16 = 0;
    
        cpu.set_register(Register::D, 0b1111_0000);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b00_101_000 | R8Operand::to_byte(R8Operand::D));
        let original_lsb = (0b1111_0000 & 1) != 0;
        assert_reg!(cpu, Register::D, 0b1111_1000);
        assert_eq!(carry_flag(&cpu), original_lsb, "SRA D carry flag mismatch");
    
        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut memory, &cpu, 0b1110_0101);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b00_101_000 | R8Operand::to_byte(R8Operand::HLInd));
        assert_mem!(memory, hl_addr, 0b1111_0010);
        assert!(carry_flag(&cpu), "SRA [HL] should set the carry flag");
    }
    
    #[test]
    fn test_cb_sla_operations() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();
        let mut offset: u16 = 0;
    
        cpu.set_register(Register::E, 0b1111_0000);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b00_100_000 | R8Operand::to_byte(R8Operand::E));
        let original_msb = (0b1111_0000 & 0x80) != 0;
        assert_reg!(cpu, Register::E, 0b1110_0000);
        assert_eq!(carry_flag(&cpu), original_msb, "SLA E carry flag mismatch");
    
        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut memory, &cpu, 0b0010_1011);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b00_100_000 | R8Operand::to_byte(R8Operand::HLInd));
        assert_mem!(memory, hl_addr, 0b0101_0110);
        assert!(!carry_flag(&cpu), "SLA [HL] should not set the carry flag");
    }

    #[test]
    fn test_cb_rr_operations() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();
        let mut offset: u16 = 0;
        
        cpu.set_register(Register::E, 0b1111_0000);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b00_011_000 | R8Operand::to_byte(R8Operand::E));
        assert_reg!(cpu, Register::E, 0b0111_1000);
        assert_eq!(carry_flag(&cpu), false, "RR on E should not set the carry flag");
    
        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut memory, &cpu, 0b1110_0101);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b00_011_000 | R8Operand::to_byte(R8Operand::HLInd));
        assert_mem!(memory, hl_addr, 0b0111_0010);
        assert!(carry_flag(&cpu), "RR [HL] should set the carry flag");
    }
    
    #[test]
    fn test_cb_rl_operations() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();
        let mut offset: u16 = 0;
    
        cpu.set_register(Register::A, 0b0000_1111);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b00_010_000 | R8Operand::to_byte(R8Operand::A));
        assert_reg!(cpu, Register::A, 0b0001_1110);
        assert_eq!(carry_flag(&cpu), false, "RL on A should not set the carry flag");
    
        let hl_addr = cpu.get_register_16(Register16::HL);
        preload_hl(&mut memory, &cpu, 0b0010_1111);
        execute_cb_instruction(&mut cpu, &mut memory, &mut offset, 0b00_010_000 | R8Operand::to_byte(R8Operand::HLInd));
        assert_mem!(memory, hl_addr, 0b0101_1110);
        assert_eq!(carry_flag(&cpu), false, "RL [HL] should not set the carry flag");
    }
    

}