#[cfg(test)]
mod tests {
    use crate::console::cpu::cpu::*;
    use crate::console::cpu::register::*;

    #[test]
    fn test_cpu_initialization() {
        let cpu = Cpu::new();
        assert_eq!(cpu.get_register(Register::A), 0);
    }
}