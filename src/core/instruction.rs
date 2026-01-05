use crate::core::StarFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StarInstruction {
    // ==== OOOO_XXXX_YYYY_ZZZZ ====

    Add, // addition -- add $rd, $r1, $r2
    Sub, // subtraction -- sub $rd, $r1, $r2
    And, // and -- and $rd, $r1, $r2
    Or, // or -- or $rd, $r1, $r2
    Xor, // xor -- xor $rd, $r1, $r2
    Shl, // shift left -- shl $rd, $r1, $r2
    Shr, // shift right -- shr $rd, $r1, $r2

    Lai, // load alt immediate -- lai $rd, $imm<8>
    Lli, // load low immediate -- lli $rd, $imm<8>

    Beqr, // branch equal -- beqr $r1, $r2, $rt
    Bneqr, // branch not equal -- bneqr $r1, $r2, $rt
    Bgtr, // branch greater than -- bgtr $r1, $r2, $rt
    Bltr, // branch less than -- bltr $r1, $r2, $rt

    Bgtur, // branch greater than unsigned relative -- bgtur $r1, $r2, $rt
    Bltur, // branch less than unsigned relative -- bltur $r1, $r2, $rt

    // ==== 1111_OOOO_XXXX_YYYY ====

    Mulhl, // multiply high low -- mulhl $r1, $r2
    Divhl, // divide high low -- divhl $r1, $r2

    Muluhl, // multiply unsigned high low -- muluhl $r1, $r2
    Divuhl, // divide unsigned high low -- divuhl $r1, $r2

    Not, // not -- not $rd, $rs

    Xlb, // extend low byte -- xlb $r1, $r2

    Lab, // load alt byte -- lab $r, $raddress
    Llb, // load low byte -- llb $r, $raddress

    Sab, // store alt byte -- sab $r, $raddress
    Slb, // store low byte -- slb $r, $raddress
    
    // ==== 1111_1111_OOOO_XXXX ====
        // DELETED: Br, // branch relative -- br $r
        J, // jump -- j $rs
    // ==== 1111_1111_1111_OOOO ====
        // DELETED: Ret, // return -- ret
    Mcall, // machine call (syscall) -- mcall
        // DELETED: Nope, // (1111_1111_1111_1111) -- nope
}

impl StarInstruction {
    pub fn format(&self) -> StarFormat {
        match self {
            // ==== TRINITY ====
            StarInstruction::Add
            | StarInstruction::Sub
            | StarInstruction::And
            | StarInstruction::Or
            | StarInstruction::Xor
            | StarInstruction::Shl
            | StarInstruction::Shr
            
            | StarInstruction::Beqr
            | StarInstruction::Bneqr
            | StarInstruction::Bgtr
            | StarInstruction::Bltr
            | StarInstruction::Bgtur
            | StarInstruction::Bltur
            => StarFormat::Trinity,

            // ==== HIME ====
            StarInstruction::Lai
            | StarInstruction::Lli
            => StarFormat::Hime,
            // ==== PAIR ====
            StarInstruction::Mulhl
            | StarInstruction::Divhl
            | StarInstruction::Muluhl
            | StarInstruction::Divuhl
            | StarInstruction::Not
            | StarInstruction::Xlb
            | StarInstruction::Lab
            | StarInstruction::Llb
            | StarInstruction::Sab
            | StarInstruction::Slb
            => StarFormat::Pair,

            // ==== CLOVER ====
            StarInstruction::J 
            => StarFormat::Clover,
            // ==== ARK ====
            StarInstruction::Mcall 
            => StarFormat::Ark,
        }
    }
    pub fn opcode(&self) -> u16 {
        match self {
            // ==== TRINITY ====
            StarInstruction::Add => 0b0000_0000_0000_0000,
            StarInstruction::Sub => 0b0000_0000_0000_0001,
            StarInstruction::And => 0b0000_0000_0000_0010,
            StarInstruction::Or => 0b0000_0000_0000_0011,
            StarInstruction::Xor => 0b0000_0000_0000_0100,
            StarInstruction::Shl => 0b0000_0000_0000_0101,
            StarInstruction::Shr => 0b0000_0000_0000_0110,

            StarInstruction::Lai => 0b0000_0000_0000_0111,
            StarInstruction::Lli => 0b0000_0000_0000_1000,

            StarInstruction::Beqr => 0b0000_0000_0000_1001,
            StarInstruction::Bneqr => 0b0000_0000_0000_1010,
            StarInstruction::Bgtr => 0b0000_0000_0000_1011,
            StarInstruction::Bltr => 0b0000_0000_0000_1100,
            StarInstruction::Bgtur => 0b0000_0000_0000_1101,
            StarInstruction::Bltur => 0b0000_0000_0000_1110,

            // ==== PAIR ====
            StarInstruction::Mulhl => 0b0000_0000_0000_1111,
            StarInstruction::Divhl => 0b0000_0000_0001_1111,
            StarInstruction::Muluhl => 0b0000_0000_0010_1111,
            StarInstruction::Divuhl => 0b0000_0000_0011_1111,

            StarInstruction::Not => 0b0000_0000_0100_1111,

            StarInstruction::Xlb => 0b0000_0000_0101_1111,
            StarInstruction::Lab => 0b0000_0000_0110_1111,
            StarInstruction::Llb => 0b0000_0000_0111_1111,
            StarInstruction::Sab => 0b0000_0000_1000_1111,
            StarInstruction::Slb => 0b0000_0000_1001_1111,

            // ==== CLOVER ====
            StarInstruction::J => 0b0000_0000_1111_1111,

            // ==== ARK ====
            StarInstruction::Mcall => 0b0000_1111_1111_1111,

        }
    }

    pub fn from_opcode(opcode: u16) -> Option<Self> {
        match opcode {
            0b_0000_0000_0000_0000 => Some(StarInstruction::Add),
            0b_0000_0000_0000_0001 => Some(StarInstruction::Sub),
            0b_0000_0000_0000_0010 => Some(StarInstruction::And),
            0b_0000_0000_0000_0011 => Some(StarInstruction::Or),
            0b_0000_0000_0000_0100 => Some(StarInstruction::Xor),
            0b_0000_0000_0000_0101 => Some(StarInstruction::Shl),
            0b_0000_0000_0000_0110 => Some(StarInstruction::Shr),

            0b_0000_0000_0000_0111 => Some(StarInstruction::Lai),
            0b_0000_0000_0000_1000 => Some(StarInstruction::Lli),

            0b_0000_0000_0000_1001 => Some(StarInstruction::Beqr),
            0b_0000_0000_0000_1010 => Some(StarInstruction::Bneqr),
            0b_0000_0000_0000_1011 => Some(StarInstruction::Bgtr),
            0b_0000_0000_0000_1100 => Some(StarInstruction::Bltr),
            0b_0000_0000_0000_1101 => Some(StarInstruction::Bgtur),
            0b_0000_0000_0000_1110 => Some(StarInstruction::Bltur),
            
            // ==== PAIR ====Some(
            0b_0000_0000_0000_1111 => Some(StarInstruction::Mulhl),
            0b_0000_0000_0001_1111 => Some(StarInstruction::Divhl),
            0b_0000_0000_0010_1111 => Some(StarInstruction::Muluhl),
            0b_0000_0000_0011_1111 => Some(StarInstruction::Divuhl),

            0b_0000_0000_0100_1111 => Some(StarInstruction::Not),

            0b_0000_0000_0101_1111 => Some(StarInstruction::Xlb),
            0b_0000_0000_0110_1111 => Some(StarInstruction::Lab),
            0b_0000_0000_0111_1111 => Some(StarInstruction::Llb),
            0b_0000_0000_1000_1111 => Some(StarInstruction::Sab),
            0b_0000_0000_1001_1111 => Some(StarInstruction::Slb),

            // ==== CLOVER ====
            0b_0000_0000_1111_1111 => Some(StarInstruction::J),
            
            // ==== ARK ====
            0b_0000_1111_1111_1111 => Some(StarInstruction::Mcall),

            _ => None,
        }
    }
}