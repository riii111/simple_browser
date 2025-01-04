use crate::renderer::dom::node::Node as DomNode;
use crate::renderer::js::ast::Node;
use crate::renderer::js::ast::Program;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use core::borrow::Borrow;
use core::cell::RefCell;
use core::fmt::Display;
use core::fmt::Formatter;
use core::ops::Add;
use core::ops::Sub;

/// https://262.ecma-international.org/#sec-ecmascript-language-types
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    /// https://262.ecma-international.org/#sec-numeric-types
    Number(u64),
    /// https://262.ecma-international.org/#sec-ecmascript-language-types-string-type
    StringLiteral(String),
    HtmlElement {
        object: Rc<RefCell<DomNode>>,
        property: Option<String>,
    },
}

impl Add<RuntimeValue> for RuntimeValue {
    type Output = RuntimeValue;

    fn add(self, rhs: RuntimeValue) -> RuntimeValue {
        if let (RuntimeValue::Number(left_num), RuntimeValue::Number(right_num)) = (&self, &rhs) {
            return RuntimeValue::Number(left_num + right_num);
        }

        RuntimeValue::StringLiteral(self.to_string() + &rhs.to_string())
    }
}

impl Sub<RuntimeValue> for RuntimeValue {
    type Output = RuntimeValue;

    fn sub(self, rhs: RuntimeValue) -> RuntimeValue {
        if let (RuntimeValue::Number(left_num), RuntimeValue::Number(right_num)) = (&self, &rhs) {
            return RuntimeValue::Number(left_num - right_num);
        }

        // NaN: Not a Number
        RuntimeValue::Number(u64::MIN)
    }
}

impl Display for RuntimeValue {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        let s = match self {
            RuntimeValue::Number(value) => format!("{}", value),
            RuntimeValue::StringLiteral(value) => value.to_string(),
            RuntimeValue::HtmlElement {
                object,
                property: _,
            } => {
                format!("HtmlElement: {:#?}", object)
            }
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
pub struct JsRuntime {}

impl JsRuntime {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute(&self, program: &Program) {
        for node in program.body() {
            self.eval(&Some(node.clone()));
        }
    }

    fn eval(&self, node: &Option<Rc<Node>>) -> Option<RuntimeValue> {
        let node = match node {
            Some(n) => n,
            None => return None,
        };

        match node.borrow() {
            Node::ExpressionStatement(expr) => self.eval(&expr),
            Node::AdditiveExpression {
                operator,
                left,
                right,
            } => {
                let left_value = self.eval(&left)?;
                let right_value = self.eval(&right)?;

                if operator == &'+' {
                    Some(left_value + right_value)
                } else if operator == &'-' {
                    Some(left_value - right_value)
                } else {
                    None
                }
            }
            Node::AssignmentExpression {
                operator,
                left,
                right,
            } => {
                // TODO: Implement assignment expression
                None
            }
            Node::MemberExpression { object, property } => {
                // TODO: Implement member expression
                None
            }
            Node::NumberLiteral(value) => Some(RuntimeValue::Number(*value)),
            // TODO: あとで削除
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::renderer::js::{ast::JsParser, token::JsLexer};

    use super::*;

    #[test]
    fn test_add_nums() {
        /* 足し算のテスト */
        let input = "1 + 2";
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new();
        let expected = [Some(RuntimeValue::Number(3))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()));
            assert_eq!(result, expected[i]);
            i += 1;
        }
    }

    #[test]
    fn test_sub_nums() {
        /* 引き算のテスト */
        let input = "2 - 1";
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new();
        let expected = [Some(RuntimeValue::Number(1))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()));
            assert_eq!(result, expected[i]);
            i += 1;
        }
    }
}
