use crate::renderer::dom::node::Node as DomNode;
use crate::renderer::js::ast::Node;
use crate::renderer::js::ast::Program;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    id: String,
    params: Vec<Option<Rc<Node>>>,
    body: Option<Rc<Node>>,
}

impl Function {
    pub fn new(id: String, params: Vec<Option<Rc<Node>>>, body: Option<Rc<Node>>) -> Self {
        Self { id, params, body }
    }
}

/// 変数の名前と値のペアを保持するマップ
type VariableMap = Vec<(String, Option<RuntimeValue>)>;

/// JSの変数のスコープ管理を行う
/// https://262.ecma-international.org/#sec-environment-records
#[derive(Debug, Clone)]
pub struct Environment {
    variables: VariableMap,
    outer: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new(outer: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            variables: VariableMap::new(),
            outer, // 外側のスコープを持つフィールド(内側→外側へのスコープへはアクセス可だが、逆は不可であることを表現)
        }
    }

    pub fn get_variable(&self, name: String) -> Option<RuntimeValue> {
        for variable in &self.variables {
            if variable.0 == name {
                return variable.1.clone();
            }
        }
        if let Some(env) = &self.outer {
            env.borrow_mut().get_variable(name)
        } else {
            None
        }
    }

    /// 現在のスコープに新しい変数を追加
    fn add_variable(&mut self, name: String, value: Option<RuntimeValue>) {
        self.variables.push((name, value));
    }

    /// 現在のスコープに既存の変数を更新
    fn update_variable(&mut self, name: String, value: Option<RuntimeValue>) {
        for i in 0..self.variables.len() {
            // もし変数を見つけた場合、今までの名前と値のタプルを削除し、新しい値とのタプルを追加
            if self.variables[i].0 == name {
                self.variables.remove(i);
                self.variables.push((name, value));
                return;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct JsRuntime {
    env: Rc<RefCell<Environment>>,
    functions: Vec<Function>,
}

impl JsRuntime {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            env: Rc::new(RefCell::new(Environment::new(None))),
        }
    }

    pub fn execute(&mut self, program: &Program) {
        for node in program.body() {
            self.eval(&Some(node.clone()), self.env.clone());
        }
    }

    fn eval(
        &mut self,
        node: &Option<Rc<Node>>,
        env: Rc<RefCell<Environment>>,
    ) -> Option<RuntimeValue> {
        let node = match node {
            Some(n) => n,
            None => return None,
        };

        match node.borrow() {
            Node::ExpressionStatement(expr) => self.eval(&expr, env.clone()),
            Node::AdditiveExpression {
                operator,
                left,
                right,
            } => {
                let left_value = self.eval(&left, env.clone())?;
                let right_value = self.eval(&right, env.clone())?;

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
                if operator != &'=' {
                    return None;
                }
                // 変数の割り当て
                if let Some(node) = left {
                    if let Node::Identifier(id) = node.borrow() {
                        let new_value = self.eval(right, env.clone());
                        env.borrow_mut().update_variable(id.clone(), new_value);
                        return None;
                    }
                }
                None
            }
            Node::MemberExpression { object, property } => {
                let object_value = match self.eval(object, env.clone()) {
                    Some(value) => value,
                    None => return None,
                };
                let property_value = match self.eval(property, env.clone()) {
                    Some(value) => value,
                    // プロパティが存在しないため、object_valueを返す
                    None => return Some(object_value),
                };

                // document.getElementById("id")のようなコードの場合、object_valueは"document"となり、property_valueは"getElementById"となる
                return Some(object_value + RuntimeValue::StringLiteral('.') + property_value);
            }
            Node::NumberLiteral(value) => Some(RuntimeValue::Number(*value)),
            Node::VariableDeclaration { declarations } => {
                for declaration in declarations {
                    self.eval(&declaration, env.clone());
                }
                None
            }
            Node::VariableDeclarator { id, init } => {
                if let Some(node) = id {
                    if let Node::Identifier(id) = node.borrow() {
                        let init = self.eval(&init, env.clone());
                        env.borrow_mut().add_variable(id.clone(), init);
                    }
                }
                None
            }
            Node::Identifier(name) => {
                match env.borrow_mut().get_variable(name.clone()) {
                    Some(v) => Some(v),
                    // 変数名が初めて使用される場合は、まだ値は保存されていないので文字列として扱う
                    // 例えば"var a = 42;"のようなコードなら、はStringLiteralとして扱われる
                    None => Some(RuntimeValue::StringLiteral(name.clone())),
                }
            }
            Node::StringLiteral(value) => Some(RuntimeValue::StringLiteral(value.clone())),
            Node::BlockStatement { body } => {
                let mut result: Option<RuntimeValue> = None;
                for stmt in body {
                    result = self.eval(&stmt, env.clone());
                }
                result
            }
            Node::ReturnStatement { argument } => {
                return self.eval(&argument, env.clone());
            }
            Node::FunctionDeclaration { id, params, body } => {
                if let Some(RuntimeValue::StringLiteral(id)) = self.eval(&id, env.clone()) {
                    let cloned_body = match body {
                        Some(b) => Some(b.clone()),
                        None => None,
                    };
                    self.functions
                        .push(Function::new(id, params.to_vec(), cloned_body));
                }
                None
            }
            Node::CallExpression { callee, arguments } => {
                // 新しいスコープを作成
                let new_env = Rc::new(RefCell::new(Environment::new(Some(env))));

                let callee_value = match self.eval(callee, new_env.clone()) {
                    Some(value) => value,
                    _ => return None,
                };

                // 既に定義されている関数を探す
                let function = {
                    let mut f: Option<Function> = None;

                    for func in &self.functions {
                        if callee_value == RuntimeValue::StringLiteral(func.id.to_string()) {
                            f = Some(func.clone());
                        }
                    }
                    match f {
                        Some(f) => f,
                        None => panic!("function {:?} not doesn't exist", callee),
                    }
                };
                // 関数呼び出し時に渡される引数を新しく作成したスコープのローカル変数として割り当て
                assert!(arguments.len() == function.params.len());
                for (i, item) in arguments.iter().enumerate() {
                    if let Some(RuntimeValue::StringLiteral(name)) =
                        self.eval(&function.params[i], new_env.clone())
                    {
                        new_env
                            .borrow_mut()
                            .add_variable(name, self.eval(item, new_env.clone()));
                    }
                }

                // 関数の中身を新しいスコープとともにevalメソッドで解釈する
                self.eval(&function.body.clone(), new_env.clone())
            }
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
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
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
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(result, expected[i]);
            i += 1;
        }
    }

    #[test]
    fn test_assign_variable() {
        /* 変数定義のテスト */
        let input = "var foo=42;";
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new();
        let expected = [None];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_add_variable_and_num() {
        /* 変数呼び出しのテスト */
        let input = "var foo=42; foo+1";
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new();
        let expected = [None, Some(RuntimeValue::Number(43))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_reassign_variable() {
        /* 変数の再代入のテスト */
        let input = "var foo=42; foo=1; foo";
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new();
        let expected = [None, None, Some(RuntimeValue::Number(1))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_add_function_and_num() {
        /* 関数を呼び出し、その戻り値と足し算を行うプログラム */
        let input = "function foo() { return 42; } foo()+1";
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new();
        let expected = [None, Some(RuntimeValue::Number(43))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_define_function_with_args() {
        /* 引数つきの関数を呼び出し、その戻り値と足し算を行うプログラム */
        let input = "function foo(a, b) { return a + b; } foo(1, 2) + 3;";
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new();
        let expected = [None, Some(RuntimeValue::Number(6))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_local_variable() {
        /* ローカル変数を定義し、そのローカル変数を関数内で参照するプログラム */
        let input = "var a=42; function foo() { var a=1; return a; } foo()+a";
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new();
        let expected = [None, None, Some(RuntimeValue::Number(43))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }
}
