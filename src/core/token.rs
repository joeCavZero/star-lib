use crate::utils::*;
use crate::core::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StarToken {
    StarProcessor(StarProcessor),
    StarGeneralRegister(StarGeneralRegister),
    StarInstruction(StarInstruction),
    StarPseudoInstruction(StarPseudoInstruction),
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
            "lai" => Ok(StarToken::StarInstruction(StarInstruction::Lai)),
            "lli" => Ok(StarToken::StarInstruction(StarInstruction::Lli)),
            "add" => Ok(StarToken::StarInstruction(StarInstruction::Add)),
            "sub" => Ok(StarToken::StarInstruction(StarInstruction::Sub)),
            "and" => Ok(StarToken::StarInstruction(StarInstruction::And)),
            "or" => Ok(StarToken::StarInstruction(StarInstruction::Or)),
            "xor" => Ok(StarToken::StarInstruction(StarInstruction::Xor)),
            "shl" => Ok(StarToken::StarInstruction(StarInstruction::Shl)),
            "shr" => Ok(StarToken::StarInstruction(StarInstruction::Shr)),
            
            "beqr" => Ok(StarToken::StarInstruction(StarInstruction::Beqr)),
            "bneqr" => Ok(StarToken::StarInstruction(StarInstruction::Bneqr)),
            "bgtr" => Ok(StarToken::StarInstruction(StarInstruction::Bgtr)),
            "bltr" => Ok(StarToken::StarInstruction(StarInstruction::Bltr)),
            "bgtur" => Ok(StarToken::StarInstruction(StarInstruction::Bgtur)),
            "bltur" => Ok(StarToken::StarInstruction(StarInstruction::Bltur)),
            
            "xlb" => Ok(StarToken::StarInstruction(StarInstruction::Xlb)),
            "lab" => Ok(StarToken::StarInstruction(StarInstruction::Lab)),
            "llb" => Ok(StarToken::StarInstruction(StarInstruction::Llb)),
            "sab" => Ok(StarToken::StarInstruction(StarInstruction::Sab)),
            "slb" => Ok(StarToken::StarInstruction(StarInstruction::Slb)),
            "mulhl" => Ok(StarToken::StarInstruction(StarInstruction::Mulhl)),
            "divhl" => Ok(StarToken::StarInstruction(StarInstruction::Divhl)),
            "muluhl" => Ok(StarToken::StarInstruction(StarInstruction::Muluhl)),
            "divuhl" => Ok(StarToken::StarInstruction(StarInstruction::Divuhl)),
            "not" => Ok(StarToken::StarInstruction(StarInstruction::Not)),

            "j" => Ok(StarToken::StarInstruction(StarInstruction::J)),

            "mcall" => Ok(StarToken::StarInstruction(StarInstruction::Mcall)),
            

            // ==== PSEUDO INSTRUCTIONS ====
            "nope" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Nope)),

            "move" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Move)),
            "swap" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Swap)),
            "la" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::La)),
            
            "lb" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Lb)),
            "lw" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Lw)),
            
            "li" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Li)),
            
            "sb" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Sb)),
            "sw" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Sw)),
            
            
            "addi" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Addi)),
            "subi" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Subi)),
            "andi" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Andi)),
            "ori" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Ori)),
            "xori" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Xori)),
            "shli" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Shli)),
            "shri" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Shri)),
            
            "inc" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Inc)),
            "dec" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Dec)),
            
            "mul" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Mul)),
            "div" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Div)),
            "mod" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Mod)),

            "muli" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Muli)),
            "divi" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Divi)),
            "modi" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Modi)),
            
            "beqa" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Beqa)),
            "bneqa" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Bneqa)),
            "bgta" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Bgta)),
            "blta" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Blta)),
            "bgtua" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Bgtua)),
            "bltua" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Bltua)),
            "ja" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Ja)),
            "jr" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Jr)),

            "ret" => Ok(StarToken::StarPseudoInstruction(StarPseudoInstruction::Ret)),
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
                    "@include" => Ok(StarToken::StarProcessor(StarProcessor::Include)),
                    "@define" => Ok(StarToken::StarProcessor(StarProcessor::Define)),
                    "@once" => Ok(StarToken::StarProcessor(StarProcessor::Once)),
                    _ => Err("Invalid processor".to_string()),
                }
            }

            // ==== REGISTERS ====
            _ if tkn_string.starts_with("$") => {
                match tkn_string.as_str() {
                    "$zero" | "$0" => Ok(StarToken::StarGeneralRegister(StarGeneralRegister::Zero)),
                    "$a" | "$1" => Ok(StarToken::StarGeneralRegister(StarGeneralRegister::A)),
                    "$b" | "$2" => Ok(StarToken::StarGeneralRegister(StarGeneralRegister::B)),
                    "$c" | "$3" => Ok(StarToken::StarGeneralRegister(StarGeneralRegister::C)),
                    "$d" | "$4" => Ok(StarToken::StarGeneralRegister(StarGeneralRegister::D)),
                    "$e" | "$5" => Ok(StarToken::StarGeneralRegister(StarGeneralRegister::E)),
                    "$f" | "$6" => Ok(StarToken::StarGeneralRegister(StarGeneralRegister::F)),
                    "$g" | "$7" => Ok(StarToken::StarGeneralRegister(StarGeneralRegister::G)),
                    "$aux1" | "$8" => Ok(StarToken::StarGeneralRegister(StarGeneralRegister::Aux1)),
                    "$aux2" | "$9" => Ok(StarToken::StarGeneralRegister(StarGeneralRegister::Aux2)),
                    "$aux3" | "$10" => Ok(StarToken::StarGeneralRegister(StarGeneralRegister::Aux3)),
                    "$carry" | "$11" => Ok(StarToken::StarGeneralRegister(StarGeneralRegister::Carry)),
                    "$low" | "$12" => Ok(StarToken::StarGeneralRegister(StarGeneralRegister::Low)),
                    "$high" | "$13" => Ok(StarToken::StarGeneralRegister(StarGeneralRegister::High)),
                    "$ra" | "$14" => Ok(StarToken::StarGeneralRegister(StarGeneralRegister::ReturnAddress)),
                    "$sp" | "$15" => Ok(StarToken::StarGeneralRegister(StarGeneralRegister::StackPointer)),
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