use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// https://262.ecma-international.org/#sec-punctuators
    Punctuator(char),
    /// https://262.ecma-international.org/#sec-literals-numeric-literals
    Number(u64),
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

        let c = self.input[self.pos];

        let token = match c {
            '+' | '-' | ';' | '=' | '(' | ')' | '{' | '}' | ',' | '.' => {
                let t = Token::Punctuator(c);
                self.pos += 1;
                return Some(t);
            }
            '0'..='9' => Token::Number(self.consume_number()),
            _ => {
                unimplemented!("char {:?} is not supported yet", c);
            }
        };

        Some(token)
    }
}
