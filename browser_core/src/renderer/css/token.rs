use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub enum CssToken {
    /// https://www.w3.org/TR/css-syntax-3/#typedef-hash-token
    HashToken(String),
    /// https://www.w3.org/TR/css-syntax-3/#typedef-delim-token
    Delim(char),
    /// https://www.w3.org/TR/css-syntax-3/#typedef-number-token
    Number(f64),
    /// https://www.w3.org/TR/css-syntax-3/#typedef-colon-token
    Colon,
    /// https://www.w3.org/TR/css-syntax-3/#typedef-semicolon-token
    SemiColon,
    /// https://www.w3.org/TR/css-syntax-3/#tokendef-open-paren
    OpenParenthesis,
    /// https://www.w3.org/TR/css-syntax-3/#tokendef-close-paren
    CloseParenthesis,
    /// https://www.w3.org/TR/css-syntax-3/#tokendef-open-curly
    OpenCurly,
    /// https://www.w3.org/TR/css-syntax-3/#tokendef-close-curly
    CloseCurly,
    /// https://www.w3.org/TR/css-syntax-3/#typedef-ident-token
    Ident(String),
    /// https://www.w3.org/TR/css-syntax-3/#typedef-string-token
    StringToken(String),
    /// https://www.w3.org/TR/css-syntax-3/#typedef-at-keyword-token
    AtKeyword(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CssTokenizer {
    pos: usize,
    input: Vec<char>,
}

impl CssTokenizer {
    pub fn new(css: String) -> Self {
        Self {
            pos: 0,
            input: css.chars().collect(),
        }
    }

    /// https://www.w3.org/TR/css-syntax-3/#consume-a-string-token
    /// 入力を文字として解釈
    fn consume_string_token(&mut self) -> String {
        let mut s = String::new();

        loop {
            if self.pos >= self.input.len() {
                return s;
            }

            self.pos += 1;
            let c = self.input[self.pos];
            match c {
                '"' | '\'' => break,
                _ => s.push(c),
            }
        }

        s
    }

    /// https://www.w3.org/TR/css-syntax-3/#consume-number
    /// https://www.w3.org/TR/css-syntax-3/#consume-a-numeric-token
    /// 入力を数字として解釈
    fn consume_numeric_token(&mut self) -> f64 {
        let mut num = 0f64;
        let mut floating = false;
        let mut floating_digit = 1f64;

        loop {
            if self.pos >= self.input.len() {
                return num;
            }

            let c = self.input[self.pos];

            match c {
                '0'..='9' => {
                    if floating {
                        floating_digit *= 1f64 / 10f64;
                        num += (c.to_digit(10).unwrap() as f64) * floating_digit
                    } else {
                        num = num * 10.0 + (c.to_digit(10).unwrap() as f64);
                    }
                    self.pos += 1;
                }
                '.' => {
                    floating = true;
                    self.pos += 1;
                }
                _ => break,
            }
        }
        num
    }

    /// https://www.w3.org/TR/css-syntax-3/#consume-ident-like-token
    /// https://www.w3.org/TR/css-syntax-3/#consume-name
    /// 入力を識別子として解釈
    fn consume_ident_token(&mut self) -> String {
        let mut s = String::new();
        s.push(self.input[self.pos]);

        loop {
            self.pos += 1;
            let c = self.input[self.pos];
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => {
                    s.push(c);
                }
                _ => break,
            }
        }

        s
    }
}

impl Iterator for CssTokenizer {
    type Item = CssToken;

    /// https://www.w3.org/TR/css-syntax-3/#consume-a-string-token
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pos >= self.input.len() {
                return None;
            }

            let c = self.input[self.pos];

            let token = match c {
                '(' => CssToken::OpenParenthesis,
                ')' => CssToken::CloseParenthesis,
                ',' => CssToken::Delim(','),
                '.' => CssToken::Delim('.'),
                ':' => CssToken::Colon,
                ';' => CssToken::SemiColon,
                '{' => CssToken::OpenCurly,
                '}' => CssToken::CloseCurly,
                ' ' | '\n' => {
                    self.pos += 1;
                    continue;
                }
                '"' | '\'' => {
                    let value = self.consume_string_token();
                    CssToken::StringToken(value)
                }
                '0'..='9' => {
                    let t = CssToken::Number(self.consume_numeric_token());
                    self.pos -= 1;
                    t
                }
                '#' => {
                    // ハッシュタグ（#） = IDセレクタとして扱う
                    // 本来はハッシュタグの後に続く文字によってハッシュトークンor記号トークンを返すが、今回は常にハッシュトークンで固定する
                    let value = self.consume_ident_token();
                    self.pos -= 1;
                    CssToken::HashToken(value)
                }
                '-' => {
                    // 負の値は取り扱わないので、ハイフン = 識別子としてのみ扱う
                    // 本来はハイフンの後に続く文字によって数字トークン、コメントアウト終了トークン、識別子トークン、記号トークンなどを返すが、今回は常にIdentで固定する
                    let t = CssToken::Ident(self.consume_ident_token());
                    self.pos -= 1;
                    t
                }
                '@' => {
                    if self.input[self.pos + 1].is_ascii_alphabetic()
                        && self.input[self.pos + 2].is_alphabetic()
                        && self.input[self.pos + 3].is_alphabetic()
                    {
                        // skip '@'
                        self.pos += 1;
                        let t = CssToken::AtKeyword(self.consume_ident_token());
                        self.pos -= 1;
                        t
                    } else {
                        CssToken::Delim('@')
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    // 小文字、大文字、_ならIdentを返す
                    let t = CssToken::Ident(self.consume_ident_token());
                    self.pos -= 1;
                    t
                }
                _ => {
                    unimplemented!("char {} is not supported yet", c);
                }
            };

            self.pos += 1;
            return Some(token);
        }
    }
}
