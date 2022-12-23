use std::ops::Range;

use nasl_syntax::{
    Lexer, Statement, Statement::*,
    Tokenizer,
};
use sink::Sink;

use crate::{
    assign::AssignExtension,
    call::CallExtension,
    context::{Definition, Register},
    declare::DeclareFunctionExtension,
    error::InterpretError,
    operator::OperatorExtension, naslvalue::NaslValue,
};


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
            For(_, _, _, _) => todo!(),
            While(_, _) => todo!(),
            Repeat(_, _) => todo!(),
            ForEach(_, _, _) => todo!(),
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
                        _ => {}
                    }
                }
                // currently blocks don't return something
                Ok(NaslValue::Null)
            }
            NoOp(_) => Ok(NaslValue::Null),
            EoF => todo!(),
            AttackCategory(cat) => Ok(NaslValue::AttackCategory(cat)),
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
