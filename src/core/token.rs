use crate::utils::*;
use crate::core::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StarToken {
    Processor(StarProcessor),
    GeneralRegister(StarGeneralRegister),
    Instruction(StarInstruction),
    PseudoInstruction(StarPseudoInstruction),
    LabelDeclaration(String),
    Identifier(String),
    MacroArgIdentifier(String),
    StarDirective(StarDirective),
    NumberLiteral(String),
    StringLiteral(String),
    Comma,
    LeftSquareBracket,
    RightSquareBracket,
    LeftParenthesis,
    RightParenthesis,
    Backslash,
}

impl StarToken {
    pub fn from_string(tkn_string: String) -> Result<Self, String> {
        match tkn_string.as_str() {
            "lai" => Ok(StarToken::Instruction(StarInstruction::Lai)),
            "lli" => Ok(StarToken::Instruction(StarInstruction::Lli)),
            "add" => Ok(StarToken::Instruction(StarInstruction::Add)),
            "sub" => Ok(StarToken::Instruction(StarInstruction::Sub)),
            "and" => Ok(StarToken::Instruction(StarInstruction::And)),
            "or" => Ok(StarToken::Instruction(StarInstruction::Or)),
            "xor" => Ok(StarToken::Instruction(StarInstruction::Xor)),
            "shl" => Ok(StarToken::Instruction(StarInstruction::Shl)),
            "shr" => Ok(StarToken::Instruction(StarInstruction::Shr)),
            
            "beqr" => Ok(StarToken::Instruction(StarInstruction::Beqr)),
            "bneqr" => Ok(StarToken::Instruction(StarInstruction::Bneqr)),
            "bgtr" => Ok(StarToken::Instruction(StarInstruction::Bgtr)),
            "bltr" => Ok(StarToken::Instruction(StarInstruction::Bltr)),
            "bgtur" => Ok(StarToken::Instruction(StarInstruction::Bgtur)),
            "bltur" => Ok(StarToken::Instruction(StarInstruction::Bltur)),
            
            "xlb" => Ok(StarToken::Instruction(StarInstruction::Xlb)),
            "lab" => Ok(StarToken::Instruction(StarInstruction::Lab)),
            "llb" => Ok(StarToken::Instruction(StarInstruction::Llb)),
            "sab" => Ok(StarToken::Instruction(StarInstruction::Sab)),
            "slb" => Ok(StarToken::Instruction(StarInstruction::Slb)),
            "mulhl" => Ok(StarToken::Instruction(StarInstruction::Mulhl)),
            "divhl" => Ok(StarToken::Instruction(StarInstruction::Divhl)),
            "muluhl" => Ok(StarToken::Instruction(StarInstruction::Muluhl)),
            "divuhl" => Ok(StarToken::Instruction(StarInstruction::Divuhl)),
            "not" => Ok(StarToken::Instruction(StarInstruction::Not)),

            "j" => Ok(StarToken::Instruction(StarInstruction::J)),

            "mcall" => Ok(StarToken::Instruction(StarInstruction::Mcall)),
            

            // ==== PSEUDO INSTRUCTIONS ====
            "nope" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Nope)),

            "move" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Move)),
            "swap" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Swap)),
            "la" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::La)),
            
            "lb" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Lb)),
            "lw" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Lw)),
            
            "li" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Li)),
            
            "sb" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Sb)),
            "sw" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Sw)),
            
            
            "addi" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Addi)),
            "subi" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Subi)),
            "andi" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Andi)),
            "ori" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Ori)),
            "xori" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Xori)),
            "shli" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Shli)),
            "shri" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Shri)),
            
            "inc" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Inc)),
            "dec" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Dec)),
            
            "mul" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Mul)),
            "div" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Div)),
            "mod" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Mod)),

            "muli" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Muli)),
            "divi" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Divi)),
            "modi" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Modi)),
            
            "beqa" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Beqa)),
            "bneqa" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Bneqa)),
            "bgta" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Bgta)),
            "blta" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Blta)),
            "bgtua" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Bgtua)),
            "bltua" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Bltua)),
            "ja" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Ja)),
            "jr" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Jr)),

            "ret" => Ok(StarToken::PseudoInstruction(StarPseudoInstruction::Ret)),
            // ==== MISC ====
            "," => Ok(StarToken::Comma),
            "[" => Ok(StarToken::LeftSquareBracket),
            "]" => Ok(StarToken::RightSquareBracket),
            "(" => Ok(StarToken::LeftParenthesis),
            ")" => Ok(StarToken::RightParenthesis),
            "\\" => Ok(StarToken::Backslash),

            // ==== PROCESSORS ====
            _ if tkn_string.starts_with("@") => {
                match tkn_string.as_str() {
                    "@include" => Ok(StarToken::Processor(StarProcessor::Include)),
                    "@define" => Ok(StarToken::Processor(StarProcessor::Define)),
                    "@once" => Ok(StarToken::Processor(StarProcessor::Once)),
                    _ => Err("Invalid processor".to_string()),
                }
            }

            // ==== REGISTERS ====
            _ if tkn_string.starts_with("$") => {
                match tkn_string.as_str() {
                    "$zero" | "$0" => Ok(StarToken::GeneralRegister(StarGeneralRegister::Zero)),
                    "$a" | "$1" => Ok(StarToken::GeneralRegister(StarGeneralRegister::A)),
                    "$b" | "$2" => Ok(StarToken::GeneralRegister(StarGeneralRegister::B)),
                    "$c" | "$3" => Ok(StarToken::GeneralRegister(StarGeneralRegister::C)),
                    "$d" | "$4" => Ok(StarToken::GeneralRegister(StarGeneralRegister::D)),
                    "$e" | "$5" => Ok(StarToken::GeneralRegister(StarGeneralRegister::E)),
                    "$f" | "$6" => Ok(StarToken::GeneralRegister(StarGeneralRegister::F)),
                    "$g" | "$7" => Ok(StarToken::GeneralRegister(StarGeneralRegister::G)),
                    "$aux1" | "$8" => Ok(StarToken::GeneralRegister(StarGeneralRegister::Aux1)),
                    "$aux2" | "$9" => Ok(StarToken::GeneralRegister(StarGeneralRegister::Aux2)),
                    "$aux3" | "$10" => Ok(StarToken::GeneralRegister(StarGeneralRegister::Aux3)),
                    "$carry" | "$11" => Ok(StarToken::GeneralRegister(StarGeneralRegister::Carry)),
                    "$low" | "$12" => Ok(StarToken::GeneralRegister(StarGeneralRegister::Low)),
                    "$high" | "$13" => Ok(StarToken::GeneralRegister(StarGeneralRegister::High)),
                    "$ra" | "$14" => Ok(StarToken::GeneralRegister(StarGeneralRegister::ReturnAddress)),
                    "$sp" | "$15" => Ok(StarToken::GeneralRegister(StarGeneralRegister::StackPointer)),
                    _ => Err("Invalid register".to_string()),
                }
            }

            // ==== DIRECTIVES ====
            _ if tkn_string.starts_with(".") => {
                match tkn_string.as_str() {
                    ".data" => Ok(StarToken::StarDirective(StarDirective::Data)),
                    ".instr" => Ok(StarToken::StarDirective(StarDirective::Instr)),
                    ".byte" => Ok(StarToken::StarDirective(StarDirective::Byte)),
                    ".word" => Ok(StarToken::StarDirective(StarDirective::Word)),
                    ".space" => Ok(StarToken::StarDirective(StarDirective::Space)),
                    ".string" => Ok(StarToken::StarDirective(StarDirective::String)),
                    ".stringz" => Ok(StarToken::StarDirective(StarDirective::Stringz)),
                    ".checkpoint" => Ok(StarToken::StarDirective(StarDirective::Checkpoint)),
                    _ => Ok(StarToken::StarDirective(StarDirective::Custom(tkn_string))),
                }
            }

            // ==== MACRO ARGUMENT ====
            _ if tkn_string.starts_with("%") => {
                let arg_name = tkn_string.trim_start_matches('%').to_string();
                Ok(StarToken::MacroArgIdentifier(arg_name))
            }

            // ==== LABEL DECLARATIONS ====
            _ if tkn_string.ends_with(":") => {
                let label = tkn_string.trim_end_matches(':').to_string();
                Ok(StarToken::LabelDeclaration(label))
            }

            // ==== STRING LITERALS ====
            _ if tkn_string.starts_with("\"") && tkn_string.ends_with("\"") => {
                let string_literal = tkn_string[1..tkn_string.len()-1].to_string();
                Ok(StarToken::StringLiteral(string_literal.processed_string()))
            }

            // ==== NUMBERS ====
            _ if tkn_string.to_lowercase().starts_with("0x") 
            || tkn_string.to_lowercase().starts_with("0b") 
            || (
                !tkn_string.starts_with("_") &&
                tkn_string.replace("_","").parse::<i64>().is_ok() 
            )
            => {
                Ok(StarToken::NumberLiteral(tkn_string))
            }

            // ==== IDENTIFIERS ====
            _ => Ok(StarToken::Identifier(tkn_string)),
        }
    }

}