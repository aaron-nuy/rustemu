#[derive(Clone)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L
}

#[derive(Clone)]
pub enum Register16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC
}
