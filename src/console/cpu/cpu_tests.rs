#[cfg(test)]
mod tests {
    use crate::console::cpu::cpu::*;
    use crate::console::cpu::instruction::R8Operand;
    use crate::console::cpu::register::*;
    use crate::console::memory::*;

    #[test]
    fn test_cpu_cb_set() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();

        memory.write_to_8b(0, 0xCB); // 0xCB
        memory.write_to_8b(1, 0b11_001_000 | R8Operand::to_byte(R8Operand::E)); // set 1, e
        
        cpu.clock(&mut memory);

        assert_eq!(cpu.get_register(Register::E), 2);

        memory.write_to_8b(2, 0xCB); // 0xCB
        memory.write_to_8b(3, 0b11_000_000 | R8Operand::to_byte(R8Operand::D)); // set 0, d
        
        cpu.clock(&mut memory);

        assert_eq!(cpu.get_register(Register::D), 1);

        memory.write_to_8b(4, 0xCB); // 0xCB
        memory.write_to_8b(5, 0b11_011_000 | R8Operand::to_byte(R8Operand::A)); // set 3, a
        
        cpu.clock(&mut memory);

        assert_eq!(cpu.get_register(Register::A), 8);

        memory.write_to_8b(6, 0xCB); // 0xCB
        memory.write_to_8b(7, 0b11_010_000 | R8Operand::to_byte(R8Operand::HLInd)); // set 2, [hl]
        
        cpu.clock(&mut memory);

        let value_hl_ind = memory.read_from_8b(cpu.get_register_16(Register16::HL));

        assert_eq!(value_hl_ind, 0xCB | 0b000_0100);

        // To make sure their values didn't change
        assert_eq!(cpu.get_register(Register::A), 8);
        assert_eq!(cpu.get_register(Register::E), 2);
        assert_eq!(cpu.get_register(Register::D), 1);
    }
}