use std::{collections::HashMap, ops::Range};

use nasl_syntax::{ACT, Keyword, Token, TokenCategory, StringCategory, NumberBase};

use crate::error::InterpretError;


/// Represents a valid Value of NASL
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NaslValue {
    /// String value
    String(String),
    /// Number value
    Number(i32),
    /// Array value
    Array(Vec<NaslValue>),
    /// Dict value
    Dict(HashMap<String, NaslValue>),
    /// Boolean value
    Boolean(bool),
    /// Attack category keyword
    AttackCategory(ACT),
    /// Null value
    Null,
    /// Returns value of the context
    Return(Box<NaslValue>),
    /// Exit value of the script
    Exit(i32),
}

impl ToString for NaslValue {
    fn to_string(&self) -> String {
        match self {
            NaslValue::String(x) => x.to_owned(),
            NaslValue::Number(x) => x.to_string(),
            NaslValue::Array(x) => x
                .iter()
                .enumerate()
                .map(|(i, v)| format!("{}: {}", i, v.to_string()))
                .collect::<Vec<String>>()
                .join(","),
            NaslValue::Dict(x) => x
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                .collect::<Vec<String>>()
                .join(","),
            NaslValue::Boolean(x) => x.to_string(),
            NaslValue::Null => "\0".to_owned(),
            NaslValue::Exit(rc) => format!("exit({})", rc),
            NaslValue::Return(rc) => format!("return({})", rc.to_string()),
            NaslValue::AttackCategory(category) => Keyword::ACT(*category).to_string(),
        }
    }
}
impl From<NaslValue> for bool {
    fn from(value: NaslValue) -> Self {
        match value {
            NaslValue::String(string) => !string.is_empty() && string != "0",
            NaslValue::Array(v) => !v.is_empty(),
            NaslValue::Boolean(boolean) => boolean,
            NaslValue::Null => false,
            NaslValue::Number(number) => number != 0,
            NaslValue::Exit(number) => number != 0,
            NaslValue::AttackCategory(_) => true,
            NaslValue::Dict(v) => !v.is_empty(),
            NaslValue::Return(number) => bool::from(*number),
        }
    }
}

impl From<&NaslValue> for i32 {
    fn from(value: &NaslValue) -> Self {
        match value {
            NaslValue::String(_) => 1,
            &NaslValue::Number(x) => x,
            NaslValue::Array(_) => 1,
            NaslValue::Dict(_) => 1,
            &NaslValue::Boolean(x) => x as i32,
            &NaslValue::AttackCategory(x) => x as i32,
            NaslValue::Null => 0,
            &NaslValue::Exit(x) => x,
            // to prevent unncessary clones for non boxed types it is resolved
            // via a general match and internal clone due to return having a boxed NaslValue
            boxed => match boxed.clone() {
                NaslValue::Return(x) => i32::from(&*x),
                _ => panic!("cannot handle {:?}", boxed),
            },
        }
    }
}

trait PrimitiveResolver<T> {
    fn resolve(&self, code: &str, range: Range<usize>) -> T;
}

impl PrimitiveResolver<String> for StringCategory {
    /// Resolves a range into a String based on code
    fn resolve(&self, code: &str, range: Range<usize>) -> String {
        match self {
            StringCategory::Quotable => code[range].to_owned(),
            StringCategory::Unquotable => {
                let mut string = code[range].to_string();
                string = string.replace(r#"\n"#, "\n");
                string = string.replace(r#"\\"#, "\\");
                string = string.replace(r#"\""#, "\"");
                string = string.replace(r#"\'"#, "'");
                string = string.replace(r#"\r"#, "\r");
                string = string.replace(r#"\t"#, "\t");
                string
            }
        }
    }
}

impl PrimitiveResolver<i32> for NumberBase {
    /// Resolves a range into number based on code
    fn resolve(&self, code: &str, range: Range<usize>) -> i32 {
        i32::from_str_radix(&code[range], self.radix()).unwrap()
    }
}

impl TryFrom<(&str, Token)> for NaslValue {
    type Error = InterpretError;

    fn try_from(value: (&str, Token)) -> Result<Self, Self::Error> {
        let (code, token) = value;
        match token.category {
            TokenCategory::String(category) => Ok(NaslValue::String(
                category.resolve(code, Range::from(token)),
            )),
            TokenCategory::Identifier(None) => Ok(NaslValue::String(
                StringCategory::Unquotable.resolve(code, Range::from(token)),
            )),
            TokenCategory::Number(base) => {
                Ok(NaslValue::Number(base.resolve(code, Range::from(token))))
            }
            _ => Err(InterpretError {
                reason: format!("invalid primitive {:?}", token.category()),
            }),
        }
    }
}

