use nasl_syntax::{Statement, Token};
use std::ops::Range;

use crate::{Interpreter, interpreter::InterpretResult, NaslValue, Definition};

pub(crate) trait LoopExtension {
    fn for_loop(&mut self, assignment: Box<Statement>, condition: Box<Statement>, update: Box<Statement>, body: Box<Statement>) -> InterpretResult;
    fn while_loop(&mut self, condition: Box<Statement>, body: Box<Statement>) -> InterpretResult;
    fn repeat_loop(&mut self, body: Box<Statement>, condition: Box<Statement>) -> InterpretResult;
    fn for_each_loop(&mut self, variable: Token, iterable: Box<Statement>, body: Box<Statement>) -> InterpretResult;
}

impl<'a> LoopExtension for Interpreter<'a> {
    fn for_loop(&mut self, assignment: Box<Statement>, condition: Box<Statement>, update: Box<Statement>, body: Box<Statement>) -> InterpretResult{
        // TODO: create child context

        // Resolve assignment
        self.resolve(*assignment)?;

        loop {
            // Check condition statement
            if !bool::from(self.resolve(*condition.clone())?) {
                break;
            }

            // Execute loop body
            let ret = self.resolve(*body.clone())?;
            // Catch special values
            match ret {
                NaslValue::Break => break,
                NaslValue::Exit(code) => return Ok(NaslValue::Exit(code)),
                NaslValue::Return(val) => return Ok(NaslValue::Return(val)),
                _ => (),
            };
            
            // Execute update Statement
            self.resolve(*update.clone())?;
            
        }

        return Ok(NaslValue::Null);
    }

    fn for_each_loop(&mut self, variable: Token, iterable: Box<Statement>, body: Box<Statement>) -> InterpretResult{
        // TODO: create child context
        
        // Get name of the iteration variable
        let iter_name = &self.code[Range::from(variable)];
        // Iterate through the iterable Statement
        for val in Vec::<NaslValue>::from(self.resolve(*iterable)?) {
            // Change the value of the iteration variable after each iteration
            self.registrat.last_mut().add_named(iter_name, Definition::Value(val));

            // Execute loop body
            let ret = self.resolve(*body.clone())?;
            // Catch special values
            match ret {
                NaslValue::Break => break,
                NaslValue::Exit(code) => return Ok(NaslValue::Exit(code)),
                NaslValue::Return(val) => return Ok(NaslValue::Return(val)),
                _ => (),
            };
        }

        return Ok(NaslValue::Null);
    }

    fn while_loop(&mut self, condition: Box<Statement>, body: Box<Statement>) -> InterpretResult{
        // TODO: create child context

        while bool::from(self.resolve(*condition.clone())?) {
            // Execute loop body
            let ret = self.resolve(*body.clone())?;
            // Catch special values
            match ret {
                NaslValue::Break => break,
                NaslValue::Exit(code) => return Ok(NaslValue::Exit(code)),
                NaslValue::Return(val) => return Ok(NaslValue::Return(val)),
                _ => (),
            };
            
        }

        return Ok(NaslValue::Null);
    }

    fn repeat_loop(&mut self, body: Box<Statement>, condition: Box<Statement>) -> InterpretResult{
        
        loop {
            // Execute loop body
            let ret = self.resolve(*body.clone())?;
            // Catch special values
            match ret {
                NaslValue::Break => break,
                NaslValue::Exit(code) => return Ok(NaslValue::Exit(code)),
                NaslValue::Return(val) => return Ok(NaslValue::Return(val)),
                _ => (),
            };

            // Check condition statement
            if !bool::from(self.resolve(*condition.clone())?) {
                break;
            }
        }

        return Ok(NaslValue::Null);
    }
}

#[cfg(test)]
mod tests {
    use sink::DefaultSink;

    use crate::{Interpreter, NaslValue};


    #[test]
    fn for_loop_test() {
        let code = r###"
        a = 0;
        for ( i = 1; i < 5; i++) {
            a += i;
        }
        a;
        "###;
        let storage = DefaultSink::new(false);
        let mut interpreter = Interpreter::new("1", &storage, vec![], code);
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(0))));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Null)));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(10))));
    }

    #[test]
    fn for_each_loop_test() {
        let code = r###"
        arr[0] = 3;
        arr[1] = 5;
        a = 0;
        foreach i (arr) {
            a += i;
        }
        a;
        "###;
        let storage = DefaultSink::new(false);
        let mut interpreter = Interpreter::new("1", &storage, vec![], code);
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(3))));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(5))));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(0))));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Null)));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(8))));
    }

    #[test]
    fn while_loop_test() {
        let code = r###"
        i = 4;
        a = 0;
        i > 0;
        while(i > 0) {
            a += i;
            i--;
        }
        a;
        i;
        "###;
        let storage = DefaultSink::new(false);
        let mut interpreter = Interpreter::new("1", &storage, vec![], code);
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(4))));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(0))));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Boolean(true))));
        
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Null)));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(10))));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(0))));
    }

    #[test]
    fn repeat_loop_test() {
        let code = r###"
        i = 4;
        a = 0;
        while(i > 0) {
            a += i;
            i--;
        }
        a;
        i;
        "###;
        let storage = DefaultSink::new(false);
        let mut interpreter = Interpreter::new("1", &storage, vec![], code);
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(4))));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(0))));        
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Null)));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(10))));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(0))));
    }

    #[test]
    fn control_flow() {
        let code = r###"
        a = 0;
        i = 5;
        while(i > 0) {
            if(i == 4) {
                i--;
                continue;
            }
            if (i == 1) {
                break;
            }
            a += i;
            i--;
        }
        a;
        i;
        "###;
        let storage = DefaultSink::new(false);
        let mut interpreter = Interpreter::new("1", &storage, vec![], code);
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(0))));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(5))));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Null)));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(10))));
        assert_eq!(interpreter.next(), Some(Ok(NaslValue::Number(1))));
    }
}