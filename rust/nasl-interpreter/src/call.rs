use nasl_syntax::{Statement, Statement::*, Token};
use std::{collections::HashMap, ops::Range};

use crate::{
    error::InterpretError, interpreter::InterpretResult, lookup,
    Definition, Interpreter, NaslValue,
};

/// Is a trait to handle function calls within nasl.
pub(crate) trait CallExtension {
    fn call(&mut self, name: Token, arguments: Box<Statement>) -> InterpretResult;
}

impl<'a> CallExtension for Interpreter<'a> {
    #[inline(always)]
    fn call(&mut self, name: Token, arguments: Box<Statement>) -> InterpretResult {
        let name = &self.code[Range::from(name)];
        // get the context
        let mut named = HashMap::new();
        let mut position = vec![];
        match *arguments {
            Parameter(params) => {
                for p in params {
                    match p {
                        NamedParameter(token, val) => {
                            let val = self.resolve(*val)?;
                            let name = self.code[Range::from(token)].to_owned();
                            named.insert(name, Definition::Value(val));
                        }
                        val => {
                            let val = self.resolve(val)?;
                            position.push(val);
                        }
                    }
                }
            }
            _ => {
                return Err(InterpretError::new(
                    "invalid statement type for function parameters".to_string(),
                ))
            }
        };
        named.insert(
            "_FCT_ANON_ARGS".to_owned(),
            Definition::Value(NaslValue::Array(position.clone())),
        );

        self.registrat.create_root_child(named);
        let result = match lookup(name) {
            // Built-In Function
            Some(function) => match function(self.key, self.storage, &self.registrat) {
                Ok(value) => Ok(value),
                Err(x) => Err(InterpretError::new(format!(
                    "unable to call function {}: {:?}",
                    name, x
                ))),
            },
            // Check for user defined function
            None => {
                let found = self
                    .registrat
                    .named(name)
                    .ok_or_else(|| InterpretError {
                        reason: format!("function {} not found", name),
                    })?
                    .clone();
                match found {
                    Definition::Function(params, stmt) => {
                        // prepare default values
                        for p in params {
                            match self.registrat.named(&p) {
                                None => {
                                    // add default NaslValue::Null for each defined params
                                    self.registrat
                                        .last_mut()
                                        .add_named(&p, Definition::Value(NaslValue::Null));
                                }
                                Some(_) => {}
                            }
                        }
                        match self.resolve(stmt)? {
                            NaslValue::Return(x) => Ok(*x),
                            a => Ok(a),
                        }
                    }
                    _ => Err(InterpretError {
                        reason: format!("unexpected ContextType: {:?}", found),
                    }),
                }
            }
        };
        self.registrat.drop_last();
        result
    }
}

#[cfg(test)]
mod tests {
    use sink::DefaultSink;

    use crate::{Interpreter, NaslValue};

    #[test]
    fn default_null_on_user_defined_functions() {
        let code = r###"
        function test(a, b) {
            return a + b;
        }
        test(a: 1, b: 2);
        test(a: 1);
        test();
        "###;
        let storage = DefaultSink::new(false);
        let mut interpreter = Interpreter::new("1", &storage, vec![], code);
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Null)));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(3))));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(1))));
    }
    #[test]
    fn fct_anon_args() {
        let code = r###"
        function test() {
            return _FCT_ANON_ARGS;
        }
        test(1, 23);
        test();
        "###;
        let storage = DefaultSink::new(false);
        let mut interpreter = Interpreter::new("1", &storage, vec![], code);
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Null)));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Array(vec![NaslValue::Number(1), NaslValue::Number(23)]))));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Array(vec![]))));
    }
}
