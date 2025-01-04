use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

static RESERVED_WORDS: [&str; 3] = ["var", "function", "return"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// https://262.ecma-international.org/#sec-punctuators
    Punctuator(char),
    /// https://262.ecma-international.org/#sec-literals-numeric-literals
    Number(u64),
    /// https://262.ecma-international.org/#sec-identifier-names
    Identifier(String),
    /// https://262.ecma-international.org/#sec-keywords-and-reserved-words
    Keyword(String),
    /// https://262.ecma-international.org/#sec-literals-string-literals
    StringLiteral(String),
}

pub struct JsLexer {
    pos: usize,
    input: Vec<char>,
}

/// レキサー: トークナイザーの同義語だが、トークナイザーの機能を含んだ、より広範な解析を行うコンポーネント
/// - トークンの生成
/// - トークンの分類
/// - 行番号などの追加情報の付加など
impl JsLexer {
    pub fn new(input: &str) -> Self {
        Self {
            pos: 0,
            input: input.chars().collect(),
        }
    }

    /// 数字を解釈
    fn consume_number(&mut self) -> u64 {
        let mut num = 0;

        loop {
            if self.pos >= self.input.len() {
                return num;
            }

            let c = self.input[self.pos];

            match c {
                '0'..='9' => {
                    num = num * 10 + (c.to_digit(10).unwrap() as u64);
                    self.pos += 1;
                }
                _ => {
                    break;
                }
            }
        }

        num
    }

    fn contains(&self, keyword: &str) -> bool {
        for i in 0..keyword.len() {
            if keyword
                .chars()
                .nth(i)
                .expect("failed to access to i-th char")
                != self.input[self.pos + i]
            {
                return false;
            }
        }
        true
    }

    fn check_reserved_word(&self) -> Option<String> {
        for word in RESERVED_WORDS {
            if self.contains(word) {
                return Some(word.to_string());
            }
        }

        None
    }

    fn consume_string(&mut self) -> String {
        let mut result = String::new();
        self.pos += 1;

        loop {
            if self.pos >= self.input.len() {
                return result;
            }

            if self.input[self.pos] == '"' {
                self.pos += 1;
                return result;
            }

            result.push(self.input[self.pos]);
            self.pos += 1;
        }
    }

    fn consume_identifier(&mut self) -> String {
        let mut result = String::new();

        loop {
            if self.pos >= self.input.len() {
                return result;
            }

            if self.input[self.pos].is_ascii_alphabetic() || self.input[self.pos] == '$' {
                result.push(self.input[self.pos]);
                self.pos += 1;
            } else {
                return result;
            }
        }
    }
}

/// 次のトークンを返すイテレータ
impl Iterator for JsLexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.input.len() {
            return None;
        }

        // ホワイトスペースまたは改行文字が続く限り、次の位置に進める
        while self.input[self.pos] == ' ' || self.input[self.pos] == '\n' {
            self.pos += 1;

            if self.pos >= self.input.len() {
                return None;
            }
        }

        // 予約語が現れたら、Keywordトークンを返す
        if let Some(keyword) = self.check_reserved_word() {
            self.pos += keyword.len();
            let token = Some(Token::Keyword(keyword));
            return token;
        }

        let c = self.input[self.pos];

        let token = match c {
            '+' | '-' | ';' | '=' | '(' | ')' | '{' | '}' | ',' | '.' => {
                let t = Token::Punctuator(c);
                self.pos += 1;
                return Some(t);
            }
            '0'..='9' => Token::Number(self.consume_number()),
            'a'..='z' | 'A'..='Z' | '_' | '$' => Token::Identifier(self.consume_identifier()),
            '"' => Token::StringLiteral(self.consume_string()),
            _ => {
                unimplemented!("char {:?} is not supported yet", c);
            }
        };

        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        /*
         * 空文字列の場合、トークンは生成されない
         */
        let input: &str = "";
        let mut lexer = JsLexer::new(input).peekable();
        assert!(lexer.peek().is_none());
    }

    #[test]
    fn test_num() {
        /*
         * 数字が1つの場合、トークンが生成される
         */
        let input = "42";
        let mut lexer = JsLexer::new(input).peekable();
        let expected = [Token::Number(42)].to_vec();
        let mut i = 0;
        while lexer.peek().is_some() {
            assert_eq!(Some(expected[i].clone()), lexer.next());
            i += 1;
        }
        assert!(lexer.peek().is_none());
    }

    #[test]
    fn test_add_nums() {
        /*
         * 足し算の場合、トークンが生成される
         */
        let input = "1 + 2";
        let mut lexer = JsLexer::new(input).peekable();
        let expected = [Token::Number(1), Token::Punctuator('+'), Token::Number(2)].to_vec();

        let mut i = 0;
        while lexer.peek().is_some() {
            assert_eq!(Some(expected[i].clone()), lexer.next());
            i += 1;
        }
        assert!(lexer.peek().is_none());
    }

    #[test]
    fn test_assign_variable() {
        /*
         * 変数やキーワードを含んだJavaScriptコードの場合、トークンが生成される
         */
        let input = "var foo=\"bar\";";
        let mut lexer = JsLexer::new(input).peekable();
        let expected = [
            Token::Keyword("var".to_string()),
            Token::Identifier("foo".to_string()),
            Token::Punctuator('='),
            Token::StringLiteral("bar".to_string()),
            Token::Punctuator(';'),
        ]
        .to_vec();

        let mut i = 0;
        while lexer.peek().is_some() {
            assert_eq!(Some(expected[i].clone()), lexer.next());
            i += 1;
        }
        assert!(lexer.peek().is_none());
    }

    #[test]
    fn test_add_variable_and_num() {
        /*
         * 定義した変数を呼び出すJavaScriptコードの場合、トークンが生成される
         */
        let input = "var foo=42; var result=foo+1;";
        let mut lexer = JsLexer::new(input).peekable();
        let expected = [
            Token::Keyword("var".to_string()),
            Token::Identifier("foo".to_string()),
            Token::Punctuator('='),
            Token::Number(42),
            Token::Punctuator(';'),
            Token::Keyword("var".to_string()),
            Token::Identifier("result".to_string()),
            Token::Punctuator('='),
            Token::Identifier("foo".to_string()),
            Token::Punctuator('+'),
            Token::Number(1),
            Token::Punctuator(';'),
        ]
        .to_vec();

        let mut i = 0;
        while lexer.peek().is_some() {
            assert_eq!(Some(expected[i].clone()), lexer.next());
            i += 1;
        }
        assert!(lexer.peek().is_none());
    }

    #[test]
    fn test_add_local_variable_and_num() {
        /*
         * ローカル変数を呼び出すJavaScriptコードの場合、トークンが生成される
         */
        let input = "function foo() { var a=42; return a; } var result = foo() + 1;";
        let mut lexer = JsLexer::new(input).peekable();
        let expected = [
            Token::Keyword("function".to_string()),
            Token::Identifier("foo".to_string()),
            Token::Punctuator('('),
            Token::Punctuator(')'),
            Token::Punctuator('{'),
            Token::Keyword("var".to_string()),
            Token::Identifier("a".to_string()),
            Token::Punctuator('='),
            Token::Number(42),
            Token::Punctuator(';'),
            Token::Keyword("return".to_string()),
            Token::Identifier("a".to_string()),
            Token::Punctuator(';'),
            Token::Punctuator('}'),
            Token::Keyword("var".to_string()),
            Token::Identifier("result".to_string()),
            Token::Punctuator('='),
            Token::Identifier("foo".to_string()),
            Token::Punctuator('('),
            Token::Punctuator(')'),
            Token::Punctuator('+'),
            Token::Number(1),
            Token::Punctuator(';'),
        ]
        .to_vec();
        let mut i = 0;
        while lexer.peek().is_some() {
            assert_eq!(Some(expected[i].clone()), lexer.next());
            i += 1;
        }
        assert!(lexer.peek().is_none());
    }
}
