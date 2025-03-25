pub struct Cpu {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
}

enum Instruction {
    ADD(Register),
}

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

impl Cpu {
    const F_ZERO_FLAG: u8 = 0x80;
    const F_SUB_FLAG: u8 = 0x40;
    const F_HALF_CARRY_FLAG: u8 = 0x20;
    const F_CARRY_FLAG: u8 = 0x10;
    const F_ZERO_FLAG_POS: u8 = 7;
    const F_SUB_FLAG_POS: u8 = 6;
    const F_HALF_CARRY_FLAG_POS: u8 = 5;
    const F_CARRY_FLAG_POS: u8 = 4;

    fn clear_bit(value: &mut u8, bit_position: u8, on: bool) {
        let mask: u8 = if on { 0xFF } else { 0x00 };
        *value &= !(0b1 << bit_position);
        *value |= (mask & (0b1 << bit_position));
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

    fn set_bit(&mut self, register: Register, bit_position: u8, on: bool) {
        Cpu::clear_bit(
            match register {
                Register::A => &mut self.a,
                Register::B => &mut self.b,
                Register::C => &mut self.c,
                Register::D => &mut self.d,
                Register::E => &mut self.e,
                Register::F => &mut self.f,
                Register::H => &mut self.h,
                Register::L => &mut self.l,
                _ => panic!("Uknown register"),
            }
            , bit_position
            , on
        );
    }

    fn add(&mut self, register: Register) {
        let register_value = self.get_register_value(register);

        let (new_register_value, did_overflow) = self.a.overflowing_add(register_value);
        let half_carry = (self.a & 0x0F) + (register_value & 0x0F) > 0x0F;


        self.set_bit(Register::F, Cpu::F_CARRY_FLAG_POS, did_overflow);
        self.set_bit(Register::F, Cpu::F_ZERO_FLAG_POS, new_register_value == 0);
        self.set_bit(Register::F, Cpu::F_SUB_FLAG_POS, false);
        self.set_bit(Register::F, Cpu::F_HALF_CARRY_FLAG_POS, half_carry);
                

        self.a = new_register_value;
    }

    fn get_register_value(&mut self, register: Register) -> u8 {
        let register_value = match register {
            Register::A => self.a,
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::F => self.f,
            Register::H => self.h,
            Register::L => self.l,
            _ => panic!("Uknown register"),
        };
        register_value
    }
    
    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::ADD(register) => self.add(register),
            _ => panic!("Uknown instruction"),
        }
    }
}
