use std::{collections::HashMap, ops::Range};

use nasl_syntax::{
    Keyword, Lexer, NumberBase, Statement, Statement::*, StringCategory, Token, TokenCategory,
    Tokenizer, ACT,
};
use sink::Sink;

use crate::{
    assign::AssignExtension,
    call::CallExtension,
    context::{Definition, Register},
    declare::DeclareFunctionExtension,
    error::InterpretError,
    operator::OperatorExtension, loop_extension::LoopExtension,
};

/// Represents a valid Value of NASL
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NaslValue {
    /// String value
    String(String),
    /// Number value
    Number(i32),
    /// Array value
    Array(Vec<NaslValue>),
    /// Array value
    Dict(HashMap<String, NaslValue>),
    /// Boolean value
    Boolean(bool),
    /// Attack category keyword
    AttackCategory(ACT),
    /// Null value
    Null,
    /// Returns value of the context
    Return(Box<NaslValue>),
    /// Signals continuing a loop
    Continue,
    /// Signals a break of a control structure
    Break,
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
            NaslValue::Continue => "".to_string(),
            NaslValue::Break => "".to_string(),
        }
    }
}

/// Used to interpret a Statement
pub struct Interpreter<'a> {
    // while running the interpreter it needs either an OID when runned based on that
    // or a filename. This mainly needed for storage.
    pub(crate) key: &'a str,
    pub(crate) code: &'a str,
    pub(crate) registrat: Register,
    pub(crate) storage: &'a dyn Sink,
    lexer: Lexer<'a>,
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
            NaslValue::Continue => false,
            NaslValue::Break => false,
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

impl From<NaslValue> for Vec<NaslValue> {
    fn from(value: NaslValue) -> Self {
        match value {
            NaslValue::Array(ret) => ret,
            NaslValue::Dict(ret) => ret.values().cloned().collect(),
            NaslValue::Boolean(_) => vec![value],
            NaslValue::Number(_) => vec![value],
            NaslValue::String(ret) => ret.chars().map(|x| NaslValue::String(x.to_string())).collect(),
            _ => vec![]
        }
    }
    
}

/// Interpreter always returns a NaslValue or an InterpretError
///
/// When a result does not contain a value than NaslValue::Null must be returned.
pub type InterpretResult = Result<NaslValue, InterpretError>;

impl<'a> Interpreter<'a> {
    /// Creates a new Interpreter.
    pub fn new(
        key: &'a str,
        storage: &'a dyn Sink,
        initial: Vec<(String, Definition)>,
        code: &'a str,
    ) -> Self {
        let mut registrat = Register::default();
        let tokenizer = Tokenizer::new(code);
        let lexer = Lexer::new(tokenizer);
        registrat.create_root(initial);

        Interpreter {
            key,
            code,
            registrat,
            storage,
            lexer,
        }
    }

    /// Interprets a Statement
    pub fn resolve(&mut self, statement: Statement) -> InterpretResult {
        match statement {
            Array(name, position) => {
                let name = &self.code[Range::from(name)];
                let val = self.registrat.named(name).ok_or_else(|| InterpretError {
                    reason: format!("{} not found.", name),
                })?;
                let val = val.clone();

                match (position, val) {
                    (None, Definition::Value(v)) => Ok(v),
                    (Some(p), Definition::Value(NaslValue::Array(x))) => {
                        let position = self.resolve(*p)?;
                        let position = i32::from(&position) as usize;
                        let result = x.get(position).ok_or_else(|| InterpretError {
                            reason: format!("positiong {} not found", position),
                        })?;
                        Ok(result.clone())
                    }
                    (Some(p), Definition::Value(NaslValue::Dict(x))) => {
                        let position = self.resolve(*p)?.to_string();
                        let result = x.get(&position).ok_or_else(|| InterpretError {
                            reason: format!("{} not found.", position),
                        })?;
                        Ok(result.clone())
                    }
                    (p, x) => Err(InterpretError {
                        reason: format!("Internal error statement: {:?} -> {:?}.", p, x),
                    }),
                }
            }
            Exit(stmt) => {
                let rc = self.resolve(*stmt)?;
                match rc {
                    NaslValue::Number(rc) => Ok(NaslValue::Exit(rc)),
                    _ => Err(InterpretError::new("expected numeric value".to_string())),
                }
            }
            Return(stmt) => {
                let rc = self.resolve(*stmt)?;
                Ok(NaslValue::Return(Box::new(rc)))
            }
            Include(_) => todo!(),
            NamedParameter(_, _) => todo!(),
            For(assignment, condition, update, body) => self.for_loop(assignment, condition, update, body),
            While(condition, body) => self.while_loop(condition, body),
            Repeat(body, condition) => self.repeat_loop(body, condition),
            ForEach(variable, iterable, body) => self.for_each_loop(variable, iterable, body),
            FunctionDeclaration(name, args, exec) => self.declare_function(name, args, exec),
            Primitive(token) => TryFrom::try_from((self.code, token)),
            Variable(token) => {
                let name: NaslValue = TryFrom::try_from((self.code, token))?;
                match self.registrat.named(&name.to_string()).ok_or_else(|| {
                    InterpretError::new(format!("variable {} not found", name.to_string()))
                })? {
                    Definition::Function(_, _) => todo!(),
                    Definition::Value(result) => Ok(result.clone()),
                }
            }
            Call(name, arguments) => self.call(name, arguments),
            Declare(_, _) => todo!(),
            // array creation
            Parameter(x) => {
                let mut result = vec![];
                for stmt in x {
                    let val = self.resolve(stmt)?;
                    result.push(val);
                }
                Ok(NaslValue::Array(result))
            }
            Assign(cat, order, left, right) => self.assign(cat, order, *left, *right),
            Operator(sign, stmts) => self.operator(sign, stmts),
            If(condition, if_block, else_block) => match self.resolve(*condition) {
                Ok(value) => {
                    if bool::from(value) {
                        return self.resolve(*if_block);
                    } else if else_block.is_some() {
                        return self.resolve(*else_block.unwrap());
                    }
                    Ok(NaslValue::Null)
                }
                Err(err) => Err(err),
            },
            Block(blocks) => {
                for stmt in blocks {
                    match self.resolve(stmt)? {
                        NaslValue::Exit(rc) => return Ok(NaslValue::Exit(rc)),
                        NaslValue::Return(rc) => return Ok(NaslValue::Return(rc)),
                        NaslValue::Break => return Ok(NaslValue::Break),
                        NaslValue::Continue => return Ok(NaslValue::Continue),
                        _ => {}
                    }
                }
                // currently blocks don't return something
                Ok(NaslValue::Null)
            }
            NoOp(_) => Ok(NaslValue::Null),
            EoF => todo!(),
            AttackCategory(cat) => Ok(NaslValue::AttackCategory(cat)),
            Continue => Ok(NaslValue::Continue),
            Break => Ok(NaslValue::Break),
        }
    }

    pub fn registrat(&self) -> &Register {
        &self.registrat
    }
}

impl<'a> Iterator for Interpreter<'a> {
    type Item = InterpretResult;

    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next().map(|lr| match lr {
            Ok(stmt) => self.resolve(stmt),
            Err(err) => Err(InterpretError::from(err)),
        })
    }
}
