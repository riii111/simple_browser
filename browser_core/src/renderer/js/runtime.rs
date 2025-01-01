use crate::renderer::dom::node::Node as DomNode;
use crate::renderer::js::ast::Node;
use crate::renderer::js::ast::Program;
use alloc::rc::Rc;
use alloc::string::String;
use core::borrow::Borrow;
use core::cell::RefCell;
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
        }
    }
}
