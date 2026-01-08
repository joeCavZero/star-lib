#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StarGeneralRegister {
    Zero,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    Aux1,
    Aux2,
    Aux3,
    Carry,
    High,
    Low,
    ReturnAddress,
    StackPointer,
}

impl StarGeneralRegister {
    pub fn code(&self) -> u16 {
        match self {
            StarGeneralRegister::Zero =>           0b0000_0000_0000_0000,
            StarGeneralRegister::A =>              0b0000_0000_0000_0001,
            StarGeneralRegister::B =>              0b0000_0000_0000_0010,
            StarGeneralRegister::C =>              0b0000_0000_0000_0011,
            StarGeneralRegister::D =>              0b0000_0000_0000_0100,
            StarGeneralRegister::E =>              0b0000_0000_0000_0101,
            StarGeneralRegister::F =>              0b0000_0000_0000_0110,
            StarGeneralRegister::G =>              0b0000_0000_0000_0111,
            StarGeneralRegister::Aux1 =>           0b0000_0000_0000_1000,
            StarGeneralRegister::Aux2 =>           0b0000_0000_0000_1001,
            StarGeneralRegister::Aux3 =>           0b0000_0000_0000_1010,
            StarGeneralRegister::Carry =>          0b0000_0000_0000_1011,
            StarGeneralRegister::Low =>            0b0000_0000_0000_1100,
            StarGeneralRegister::High =>           0b0000_0000_0000_1101,
            StarGeneralRegister::ReturnAddress =>  0b0000_0000_0000_1110,
            StarGeneralRegister::StackPointer =>   0b0000_0000_0000_1111,
        }
    } 

    pub fn from_code(code: u16) -> Self {
        let c = code & 0b0000_0000_0000_1111; // Ensure only the last 4 bits are considered
        match c {
            0b0000_0000_0000_0000 => StarGeneralRegister::Zero,
            0b0000_0000_0000_0001 => StarGeneralRegister::A,
            0b0000_0000_0000_0010 => StarGeneralRegister::B,
            0b0000_0000_0000_0011 => StarGeneralRegister::C,
            0b0000_0000_0000_0100 => StarGeneralRegister::D,
            0b0000_0000_0000_0101 => StarGeneralRegister::E,
            0b0000_0000_0000_0110 => StarGeneralRegister::F,
            0b0000_0000_0000_0111 => StarGeneralRegister::G,
            0b0000_0000_0000_1000 => StarGeneralRegister::Aux1,
            0b0000_0000_0000_1001 => StarGeneralRegister::Aux2,
            0b0000_0000_0000_1010 => StarGeneralRegister::Aux3,
            0b0000_0000_0000_1011 => StarGeneralRegister::Carry,
            0b0000_0000_0000_1100 => StarGeneralRegister::Low,
            0b0000_0000_0000_1101 => StarGeneralRegister::High,
            0b0000_0000_0000_1110 => StarGeneralRegister::ReturnAddress,
            0b0000_0000_0000_1111 => StarGeneralRegister::StackPointer,
            _ => StarGeneralRegister::Zero,
        }
    }
}