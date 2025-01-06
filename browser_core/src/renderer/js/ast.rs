use crate::renderer::js::token::JsLexer;
use crate::renderer::js::token::Token;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::iter::Peekable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    ExpressionStatement(Option<Rc<Node>>),
    AdditiveExpression {
        operator: char,
        left: Option<Rc<Node>>,
        right: Option<Rc<Node>>,
    },
    AssignmentExpression {
        operator: char,
        left: Option<Rc<Node>>,
        right: Option<Rc<Node>>,
    },
    MemberExpression {
        object: Option<Rc<Node>>,
        property: Option<Rc<Node>>,
    },
    NumberLiteral(u64),
    VariableDeclaration {
        declarations: Vec<Option<Rc<Node>>>,
    },
    VariableDeclarator {
        id: Option<Rc<Node>>,
        init: Option<Rc<Node>>,
    },
    Identifier(String),
    StringLiteral(String),
    BlockStatement {
        body: Vec<Option<Rc<Node>>>,
    },
    ReturnStatement {
        argument: Option<Rc<Node>>,
    },
    FunctionDeclaration {
        id: Option<Rc<Node>>,
        params: Vec<Option<Rc<Node>>>,
        body: Option<Rc<Node>>,
    },
    CallExpression {
        callee: Option<Rc<Node>>,
        arguments: Vec<Option<Rc<Node>>>,
    },
}

impl Node {
    pub fn new_expression_statement(expression: Option<Rc<Self>>) -> Option<Rc<Self>> {
        Some(Rc::new(Node::ExpressionStatement(expression)))
    }

    pub fn new_additive_expression(
        operator: char,
        left: Option<Rc<Self>>,
        right: Option<Rc<Self>>,
    ) -> Option<Rc<Self>> {
        Some(Rc::new(Node::AdditiveExpression {
            operator,
            left,
            right,
        }))
    }

    pub fn new_assignment_expression(
        operator: char,
        left: Option<Rc<Self>>,
        right: Option<Rc<Self>>,
    ) -> Option<Rc<Self>> {
        Some(Rc::new(Node::AssignmentExpression {
            operator,
            left,
            right,
        }))
    }

    pub fn new_member_expression(
        object: Option<Rc<Self>>,
        property: Option<Rc<Self>>,
    ) -> Option<Rc<Self>> {
        Some(Rc::new(Node::MemberExpression { object, property }))
    }

    pub fn new_numeric_literal(value: u64) -> Option<Rc<Self>> {
        Some(Rc::new(Node::NumberLiteral(value)))
    }

    pub fn new_variable_declaration(declarations: Vec<Option<Rc<Self>>>) -> Option<Rc<Self>> {
        Some(Rc::new(Node::VariableDeclaration { declarations }))
    }

    pub fn new_identifier(name: String) -> Option<Rc<Self>> {
        Some(Rc::new(Node::Identifier(name)))
    }

    pub fn new_variable_identifier(name: String) -> Option<Rc<Self>> {
        Some(Rc::new(Node::Identifier(name)))
    }

    pub fn new_string_literal(value: String) -> Option<Rc<Self>> {
        Some(Rc::new(Node::StringLiteral(value)))
    }

    pub fn new_variable_declarator(
        id: Option<Rc<Self>>,
        init: Option<Rc<Self>>,
    ) -> Option<Rc<Self>> {
        Some(Rc::new(Node::VariableDeclarator { id, init }))
    }

    pub fn new_block_statement(body: Vec<Option<Rc<Self>>>) -> Option<Rc<Self>> {
        Some(Rc::new(Node::BlockStatement { body }))
    }

    pub fn new_return_statement(argument: Option<Rc<Self>>) -> Option<Rc<Self>> {
        Some(Rc::new(Node::ReturnStatement { argument }))
    }

    pub fn new_function_declaration(
        id: Option<Rc<Self>>,
        params: Vec<Option<Rc<Self>>>,
        body: Option<Rc<Self>>,
    ) -> Option<Rc<Self>> {
        Some(Rc::new(Node::FunctionDeclaration { id, params, body }))
    }

    pub fn new_call_expression(
        callee: Option<Rc<Self>>,
        arguments: Vec<Option<Rc<Self>>>,
    ) -> Option<Rc<Self>> {
        Some(Rc::new(Node::CallExpression { callee, arguments }))
    }
}

pub struct JsParser {
    t: Peekable<JsLexer>,
}

impl JsParser {
    pub fn new(t: JsLexer) -> Self {
        Self { t: t.peekable() }
    }

    /// トークンが"="の場合、assignment_expressionを呼び、それを変数の初期値として返す
    fn initialiser(&mut self) -> Option<Rc<Node>> {
        let t = match self.t.next() {
            Some(token) => token,
            None => return None,
        };

        match t {
            Token::Punctuator(c) => match c {
                '=' => self.assignment_expression(),
                _ => None,
            },
            _ => None,
        }
    }

    /// ASTを構築するためのメソッド
    pub fn parse_ast(&mut self) -> Program {
        let mut program = Program::new();

        let mut body = Vec::new();

        loop {
            // ノードが生成できなくなるまで繰り返す
            let node = self.source_element();

            match node {
                Some(n) => body.push(n),
                None => {
                    program.set_body(body);
                    return program;
                }
            }
        }
    }

    /// EBNFのSourceElementを解析する
    fn source_element(&mut self) -> Option<Rc<Node>> {
        let t = match self.t.peek() {
            Some(t) => t,
            None => return None,
        };

        match t {
            Token::Keyword(keyword) => {
                if keyword == "function" {
                    // "function"を消費する
                    assert!(self.t.next().is_some());
                    self.function_declaration()
                } else {
                    self.statement()
                }
            }
            _ => self.statement(),
        }
    }

    /// EBNFのFunctionDeclarationを解析する
    fn function_declaration(&mut self) -> Option<Rc<Node>> {
        let id = self.identifier();
        let params = self.parameter_list();
        Node::new_function_declaration(id, params, self.function_body())
    }

    /// EBNFのFormalParameterListを解析する
    fn parameter_list(&mut self) -> Vec<Option<Rc<Node>>> {
        let mut params = Vec::new();

        // '('を消費する。もし次のトークンが'('でない場合、エラーになる
        match self.t.next() {
            Some(t) => match t {
                Token::Punctuator(c) => assert!(c == '('),
                _ => unimplemented!("function should have `(` but got {:?}", t),
            },
            None => unimplemented!("function should have `(` but got None"),
        }

        loop {
            // ')'に到達するまで、paramsに仮引数となる変数を追加する
            match self.t.peek() {
                Some(t) => match t {
                    Token::Punctuator(')') => {
                        // ')'を消費する
                        assert!(self.t.next().is_some());
                        return params;
                    }
                    Token::Punctuator(',') => {
                        // ','を消費する
                        assert!(self.t.next().is_some());
                    }
                    _ => {
                        params.push(self.identifier());
                    }
                },
                None => return params,
            }
        }
    }

    /// EBNFのFunctionBodyを解析する
    fn function_body(&mut self) -> Option<Rc<Node>> {
        // '{'を消費する
        match self.t.next() {
            Some(t) => match t {
                Token::Punctuator(c) => assert!(c == '{'),
                _ => unimplemented!("function should have curly blacket but got {:?}", t),
            },
            None => unimplemented!("function should have curly blacket but got None"),
        }

        // 関数本体を解析する
        let mut body = Vec::new();
        loop {
            // '}'に到達するまで、関数内のコードとして解釈
            match self.t.peek() {
                Some(t) => match t {
                    Token::Punctuator('}') => {
                        // '}'を消費し、BlockStatementノードを返す
                        assert!(self.t.next().is_some());
                        return Node::new_block_statement(body);
                    }
                    _ => {}
                },
                None => {}
            }

            body.push(self.source_element());
        }
    }

    fn variable_declaration(&mut self) -> Option<Rc<Node>> {
        let ident = self.identifier();

        let declarator = Node::new_variable_declarator(ident, self.initialiser());

        let mut declarations = Vec::new();
        declarations.push(declarator);

        Node::new_variable_declaration(declarations)
    }

    fn identifier(&mut self) -> Option<Rc<Node>> {
        let t = match self.t.next() {
            Some(token) => token,
            None => return None,
        };

        match t {
            Token::Identifier(name) => Node::new_variable_identifier(name),
            _ => None,
        }
    }

    /// EBNFのStatement, ExpressionStatementを解析する
    fn statement(&mut self) -> Option<Rc<Node>> {
        let t = match self.t.peek() {
            Some(t) => t,
            None => return None,
        };

        let node = match t {
            Token::Keyword(keyword) => {
                if keyword == "var" {
                    // 予約語"var"を消費する
                    assert!(self.t.next().is_some());

                    self.variable_declaration()
                } else if keyword == "return" {
                    // 予約語"return"を消費する
                    assert!(self.t.next().is_some());

                    Node::new_return_statement(self.assignment_expression())
                } else {
                    None
                }
            }
            _ => Node::new_expression_statement(self.assignment_expression()),
        };

        if let Some(Token::Punctuator(c)) = self.t.peek() {
            // ';'を消費する
            if c == &';' {
                assert!(self.t.next().is_some());
            }
        }

        node
    }

    /// EBNFのAssignmentExpressionを解析する
    fn assignment_expression(&mut self) -> Option<Rc<Node>> {
        let expr = self.additive_expression();

        let t = match self.t.peek() {
            Some(token) => token,
            None => return expr,
        };

        match t {
            Token::Punctuator('=') => {
                // '='を消費する
                assert!(self.t.next().is_some());
                Node::new_assignment_expression('=', expr, self.assignment_expression())
                // '='の場合はAssignmentExpressionノードを生成
            }
            _ => expr, // それ以外ならAdditiveExpressionを返す
        }
    }

    /// EBNFのAdditiveExpressionを解析する
    fn additive_expression(&mut self) -> Option<Rc<Node>> {
        let left = self.left_hand_side_expression();

        let t = match self.t.peek() {
            Some(token) => token.clone(),
            None => return left,
        };

        match t {
            Token::Punctuator(c) => match c {
                '+' | '-' => {
                    // '+' or '-'を消費する
                    assert!(self.t.next().is_some());
                    Node::new_additive_expression(c, left, self.assignment_expression())
                }
                _ => return left,
            },
            _ => return left,
        }
    }

    /// EBNFのLeftHandSideExpressionを解析する
    fn left_hand_side_expression(&mut self) -> Option<Rc<Node>> {
        let expr = self.member_expression();

        let t = match self.t.peek() {
            Some(token) => token,
            None => return expr,
        };

        match t {
            Token::Punctuator(c) => {
                if c == &'(' {
                    // '('を消費する
                    assert!(self.t.next().is_some());
                    // 関数呼び出しのため、CallExpressionノードを返す
                    return Node::new_call_expression(expr, self.arguments());
                } else {
                    expr
                }
            }
            _ => expr,
        }
    }

    /// EBNFのArgumentsを解析する
    fn arguments(&mut self) -> Vec<Option<Rc<Node>>> {
        let mut args = Vec::new();

        loop {
            // )に到達するまで、解釈した値をargsに追加
            match self.t.peek() {
                Some(t) => match t {
                    Token::Punctuator(')') => {
                        // ')'を消費する
                        assert!(self.t.next().is_some());
                        return args;
                    }
                    Token::Punctuator(',') => {
                        // ','を消費する
                        assert!(self.t.next().is_some());
                    }
                    _ => {
                        args.push(self.assignment_expression());
                    }
                },
                None => return args,
            }
        }
    }

    /// EBNFのMemberExpressionを解析する
    fn member_expression(&mut self) -> Option<Rc<Node>> {
        let expr = self.primary_expression();

        let t = match self.t.peek() {
            Some(token) => token,
            None => return expr,
        };

        match t {
            Token::Punctuator(c) => {
                if c == &'.' {
                    // '.'を消費する
                    assert!(self.t.next().is_some());
                    Node::new_member_expression(expr, self.identifier())
                } else {
                    expr
                }
            }
            _ => expr,
        }
    }

    /// EBNFのPrimaryExpressionを解析する
    fn primary_expression(&mut self) -> Option<Rc<Node>> {
        let t = match self.t.next() {
            Some(token) => token,
            None => return None,
        };

        match t {
            Token::Identifier(value) => Node::new_identifier(value),
            Token::StringLiteral(value) => Node::new_string_literal(value),
            Token::Number(value) => Node::new_numeric_literal(value),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    body: Vec<Rc<Node>>,
}

impl Program {
    pub fn new() -> Self {
        Self { body: Vec::new() }
    }

    pub fn set_body(&mut self, body: Vec<Rc<Node>>) {
        self.body = body;
    }

    pub fn body(&self) -> &Vec<Rc<Node>> {
        &self.body
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_empty() {
        /* JavaScriptが空のプログラム */
        let input = "";
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let expected = Program::new();
        assert_eq!(expected, parser.parse_ast());
    }

    #[test]
    fn test_num() {
        /* 数字1つだけが存在するプログラム */
        let input: &str = "42";
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::ExpressionStatement(Some(Rc::new(
            Node::NumberLiteral(42),
        )))));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }

    #[test]
    fn test_add_nums() {
        /* 2つの数字を足し合わせるプログラム */
        let input: &str = "1 + 2";
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::ExpressionStatement(Some(Rc::new(
            Node::AdditiveExpression {
                operator: '+',
                left: Some(Rc::new(Node::NumberLiteral(1))),
                right: Some(Rc::new(Node::NumberLiteral(2))),
            },
        )))));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }

    #[test]
    fn test_assign_variable() {
        /* 変数に値を代入するプログラム */
        let input = "var foo=\"bar\";";
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::VariableDeclaration {
            declarations: [Some(Rc::new(Node::VariableDeclarator {
                id: Some(Rc::new(Node::Identifier("foo".to_string()))),
                init: Some(Rc::new(Node::StringLiteral("bar".to_string()))),
            }))]
            .to_vec(),
        }));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }

    #[test]
    fn test_define_function() {
        /* 関数を定義するプログラム */
        let input = "function foo() { return 42; }";
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::FunctionDeclaration {
            id: Some(Rc::new(Node::Identifier("foo".to_string()))),
            params: [].to_vec(),
            body: Some(Rc::new(Node::BlockStatement {
                body: [Some(Rc::new(Node::ReturnStatement {
                    argument: Some(Rc::new(Node::NumberLiteral(42))),
                }))]
                .to_vec(),
            })),
        }));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }

    #[test]
    fn test_define_function_with_args() {
        /* 引数を持つ関数を定義するプログラム */
        let input = "function foo(a, b) { return a + b; }";
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::FunctionDeclaration {
            id: Some(Rc::new(Node::Identifier("foo".to_string()))),
            params: [
                Some(Rc::new(Node::Identifier("a".to_string()))),
                Some(Rc::new(Node::Identifier("b".to_string()))),
            ]
            .to_vec(),
            body: Some(Rc::new(Node::BlockStatement {
                body: [Some(Rc::new(Node::ReturnStatement {
                    argument: Some(Rc::new(Node::AdditiveExpression {
                        operator: '+',
                        left: Some(Rc::new(Node::Identifier("a".to_string()))),
                        right: Some(Rc::new(Node::Identifier("b".to_string()))),
                    })),
                }))]
                .to_vec(),
            })),
        }));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }

    #[test]
    fn test_add_function_add_num() {
        /* 関数を呼び出すプログラム */
        let input = "function foo() { return 42; } var result = foo() + 1;";
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let mut expected = Program::new();
        let mut body = Vec::new();
        body.push(Rc::new(Node::FunctionDeclaration {
            id: Some(Rc::new(Node::Identifier("foo".to_string()))),
            params: [].to_vec(),
            body: Some(Rc::new(Node::BlockStatement {
                body: [Some(Rc::new(Node::ReturnStatement {
                    argument: Some(Rc::new(Node::NumberLiteral(42))),
                }))]
                .to_vec(),
            })),
        }));
        body.push(Rc::new(Node::VariableDeclaration {
            declarations: [Some(Rc::new(Node::VariableDeclarator {
                id: Some(Rc::new(Node::Identifier("result".to_string()))),
                init: Some(Rc::new(Node::AdditiveExpression {
                    operator: '+',
                    left: Some(Rc::new(Node::CallExpression {
                        callee: Some(Rc::new(Node::Identifier("foo".to_string()))),
                        arguments: [].to_vec(),
                    })),
                    right: Some(Rc::new(Node::NumberLiteral(1))),
                })),
            }))]
            .to_vec(),
        }));
        expected.set_body(body);
        assert_eq!(expected, parser.parse_ast());
    }
}
