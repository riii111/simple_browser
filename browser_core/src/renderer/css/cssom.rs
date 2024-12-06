use crate::renderer::css::token::CssToken;
use crate::renderer::css::token::CssTokenizer;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::iter::Peekable;

/*
Peekable（イテレータのWrapper）を使って、トークンを先読みする
CssTokenizerはIteratorトレイトを実装したのでnextメソッドを持つが、これは呼び出すごとにトークンを消費する
（つまり同じトークンは2回以上取得できない）
そこでPeekableを使って、トークンを先読みする
*/

#[derive(Debug, Clone)]
pub struct CssParser {
    t: Peekable<CssTokenizer>,
}

impl CssParser {
    pub fn new(t: CssTokenizer) -> Self {
        Self { t: t.peekable() }
    }

    pub fn parse_stylesheet(&mut self) -> StyleSheet {
        let mut sheet = StyleSheet::new();

        // トークン列からルールのリストを作成し、StyleSheetのフィールドに設定する
        sheet.set_rules(self.consume_list_of_rules());
        sheet
    }

    fn consume_list_of_rules(&mut self) -> Vec<QualifiedRule> {
        let mut rules = Vec::new();

        loop {
            let token = match self.t.peek() {
                Some(t) => t,
                None => return rules,
            };

            match token {
                // AtKeyword トークンが出てきた場合、ほかのCSSをインポートする
                // @import、メディアクエリをあらわす@media などのルールがはじまることをあらわす
                CssToken::AtKeyword(_keyword) => {
                    let _rule = self.consume_qualified_rule();
                    // しかし、本書のブラウザでは@からはじまるルールはサポートしない予定なので、無視する
                }
                _ => {
                    // それ以外の場合は、通常のルールをパースする
                    let rule = self.consume_qualified_rule();
                    match rule {
                        Some(r) => rules.push(r),
                        None => return rules,
                    }
                }
            }
        }
    }

    fn consume_qualified_rule(&mut self) -> Option<QualifiedRule> {
        let mut rule = QualifiedRule::new();

        loop {
            let token = match self.t.peek() {
                Some(t) => t,
                None => return None,
            };

            match token {
                CssToken::OpenCurly => {
                    // 宣言ブロックとして解釈
                    assert_eq!(self.t.next(), Some(CssToken::OpenCurly));
                    rule.set_declarations(self.consume_list_of_declarations());
                    return Some(rule);
                }
                _ => {
                    // セレクタとして解釈
                    rule.set_selector(self.consume_selector());
                }
            }
        }
    }

    fn consume_selector(&mut self) -> Selector {
        let token = match self.t.next() {
            Some(t) => t,
            None => panic!("should have a token but got None"),
        };

        match token {
            CssToken::HashToken(value) => Selector::IdSelector(value[1..].to_string()),
            CssToken::Delim(delim) => {
                if delim == '.' {
                    return Selector::ClassSelector(self.consume_ident());
                }
                panic!("Parse error: {:?} is an unexpected token.", token);
            }
            CssToken::Ident(ident) => {
                // a:hover のようなセレクタは”タイプセレクタ”として扱うため、もしコロン（:）が出てきた場合
                // 宣言ブロックの開始直前までトークンを進める
                if self.t.peek() == Some(&CssToken::Colon) {
                    while self.t.peek() != Some(&CssToken::OpenCurly) {
                        self.t.next();
                    }
                }
                Selector::TypeSelector(ident.to_string())
            }
            CssToken::AtKeyword(_keyword) => {
                // @からはじまるルールを無視するために、宣言ブロックの開始直前までトークンを進める
                while self.t.peek() != Some(&CssToken::OpenCurly) {
                    self.t.next();
                }
                Selector::UnknownSelector
            }
            _ => {
                self.t.next();
                Selector::UnknownSelector
            }
        }
    }

    fn consume_list_of_declarations(&mut self) -> Vec<Declaration> {
        let mut declarations = Vec::new();

        loop {
            let token = match self.t.peek() {
                Some(t) => t,
                None => return declarations,
            };

            match token {
                CssToken::CloseCurly => {
                    assert_eq!(self.t.next(), Some(CssToken::CloseCurly));
                    // 今まで作成した宣言のベクタを返す
                    return declarations;
                }
                CssToken::SemiColon => {
                    assert_eq!(self.t.next(), Some(CssToken::SemiColon));
                    // 宣言が終了しただけなので、何もしない
                }
                CssToken::Ident(ref _ident) => {
                    if let Some(declaration) = self.consume_declaration() {
                        declarations.push(declaration);
                    }
                }
                _ => {
                    self.t.next();
                }
            }
        }
    }

    fn consume_declaration(&mut self) -> Option<Declaration> {
        self.t.peek()?;

        let mut declaration = Declaration::new();
        declaration.set_property(self.consume_ident());

        // 次のトークンがコロン以外ならパースエラーなのでNoneを返す
        match self.t.next() {
            Some(CssToken::Colon) => {}
            _ => return None,
        }

        // Declaration構造体の値にコンポーネント値を設定する
        declaration.set_value(self.consume_component_value());

        Some(declaration)
    }

    fn consume_ident(&mut self) -> String {
        let token = match self.t.next() {
            Some(t) => t,
            None => panic!("should have a token but got None"),
        };

        match token {
            CssToken::Ident(ref ident) => ident.to_string(),
            _ => {
                panic!("Parse error: {:?} is an unexpected token.", token);
            }
        }
    }

    /// https://www.w3.org/TR/css-syntax-3/#consume-component-value
    fn consume_component_value(&mut self) -> ComponentValue {
        self.t
            .next()
            .expect("should have a token in consume_component_value")
    }
}

/// https://www.w3.org/TR/cssom-1/#cssstylesheet
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StyleSheet {
    /// https://drafts.csswg.org/cssom/#dom-cssstylesheet-cssrules
    pub rules: Vec<QualifiedRule>,
}

impl StyleSheet {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn set_rules(&mut self, rules: Vec<QualifiedRule>) {
        self.rules = rules;
    }
}

/// https://www.w3.org/TR/css-syntax-3/#qualified-rule
/// QualifiedRule = ルールノード
/// ルールノードは通常、複数のセレクタが存在できるが、今回は1ルールにつき一つだけとする
#[derive(Debug, Clone, PartialEq, Default)]
pub struct QualifiedRule {
    /// https://www.w3.org/TR/selectors-4/#typedef-selector-list
    /// The prelude of the qualified rule is parsed as a <selector-list>.
    pub selector: Selector,
    /// https://www.w3.org/TR/css-syntax-3/#parse-a-list-of-declarations
    /// The content of the qualified rule’s block is parsed as a list of declarations.
    pub declarations: Vec<Declaration>,
}

impl QualifiedRule {
    pub fn new() -> Self {
        Self {
            selector: Selector::TypeSelector("".to_string()),
            declarations: Vec::new(),
        }
    }

    pub fn set_selector(&mut self, selector: Selector) {
        self.selector = selector;
    }

    pub fn set_declarations(&mut self, declarations: Vec<Declaration>) {
        self.declarations = declarations;
    }
}

/// https://www.w3.org/TR/selectors-4/
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    /// https://www.w3.org/TR/selectors-4/#type-selectors
    TypeSelector(String), // タグ名で指定
    /// https://www.w3.org/TR/selectors-4/#class-html
    ClassSelector(String), // クラス名で指定
    /// https://www.w3.org/TR/selectors-4/#id-selectors
    IdSelector(String), // idで指定
    /// パース中にエラーが起こったときに使用されるセレクタ
    UnknownSelector,
}

impl Default for Selector {
    fn default() -> Self {
        Selector::TypeSelector(String::new())
    }
}

/// https://www.w3.org/TR/css-syntax-3/#declaration
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Declaration {
    pub property: String,
    pub value: ComponentValue,
}

impl Declaration {
    pub fn new() -> Self {
        Self {
            property: String::new(),
            value: ComponentValue::Ident(String::new()),
        }
    }

    pub fn set_property(&mut self, property: String) {
        self.property = property;
    }

    pub fn set_value(&mut self, value: ComponentValue) {
        self.value = value;
    }
}

/// プロパティの値に対するノード
pub type ComponentValue = CssToken;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_empty() {
        /* 空のスタイルシートだった場合、ルールは存在しないことを確認する */
        let style = "".to_string();
        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(t).parse_stylesheet();

        assert_eq!(cssom.rules.len(), 0);
    }

    #[test]
    fn test_one_rule() {
        /* 一つのルールを持つスタイルシートをパースする場合、そのルールが正しくパースされることを確認する */
        let style = "p { color: red; }".to_string();
        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(t).parse_stylesheet();

        let mut rule = QualifiedRule::new();
        rule.set_selector(Selector::TypeSelector("p".to_string()));
        let mut declaration = Declaration::new();
        declaration.set_property("color".to_string());
        declaration.set_value(ComponentValue::Ident("red".to_string()));
        rule.set_declarations(vec![declaration]);

        let expected = [rule];
        assert_eq!(cssom.rules.len(), expected.len());

        for (i, rule) in cssom.rules.iter().enumerate() {
            assert_eq!(&expected[i], rule);
        }
    }

    #[test]
    fn test_id_selector() {
        /* idセレクタを持つスタイルシートをパースする場合、そのルールが正しくパースされることを確認する */
        let style = "#id { color: red; }".to_string();
        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(t).parse_stylesheet();

        let mut rule = QualifiedRule::new();
        rule.set_selector(Selector::IdSelector("id".to_string()));
        let mut declaration = Declaration::new();
        declaration.set_property("color".to_string());
        declaration.set_value(ComponentValue::Ident("red".to_string()));
        rule.set_declarations(vec![declaration]);

        let expected = [rule];
        assert_eq!(cssom.rules.len(), expected.len());

        for (i, rule) in cssom.rules.iter().enumerate() {
            assert_eq!(&expected[i], rule);
        }
    }

    #[test]
    fn test_class_selector() {
        /* クラスセレクタを持つスタイルシートをパースする場合、そのルールが正しくパースされることを確認する */
        let style = ".class { color: red; }".to_string();
        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(t).parse_stylesheet();

        let mut rule = QualifiedRule::new();
        rule.set_selector(Selector::ClassSelector("class".to_string()));
        let mut declaration = Declaration::new();
        declaration.set_property("color".to_string());
        declaration.set_value(ComponentValue::Ident("red".to_string()));
        rule.set_declarations(vec![declaration]);

        let expected = [rule];
        assert_eq!(cssom.rules.len(), expected.len());

        for (i, rule) in cssom.rules.iter().enumerate() {
            assert_eq!(&expected[i], rule);
        }
    }

    #[test]
    fn test_multiple_rules() {
        /* 複数のルールを持つスタイルシートをパースする場合、そのルールが正しくパースされることを確認する */
        let style = "p { content: \"Hey\"; } h1 { font-size: 40; color: blue; }".to_string();
        let t = CssTokenizer::new(style);
        let cssom = CssParser::new(t).parse_stylesheet();

        let mut rule1 = QualifiedRule::new();
        rule1.set_selector(Selector::TypeSelector("p".to_string()));
        let mut declaration1 = Declaration::new();
        declaration1.set_property("content".to_string());
        declaration1.set_value(ComponentValue::StringToken("Hey".to_string()));
        rule1.set_declarations(vec![declaration1]);

        let mut rule2 = QualifiedRule::new();
        rule2.set_selector(Selector::TypeSelector("h1".to_string()));
        let mut declaration2 = Declaration::new();
        declaration2.set_property("font-size".to_string());
        declaration2.set_value(ComponentValue::Number(40.0));
        let mut declaration3 = Declaration::new();
        declaration3.set_property("color".to_string());
        declaration3.set_value(ComponentValue::Ident("blue".to_string()));
        rule2.set_declarations(vec![declaration2, declaration3]);

        let expected = [rule1, rule2];
        assert_eq!(cssom.rules.len(), expected.len());

        for (i, rule) in cssom.rules.iter().enumerate() {
            assert_eq!(&expected[i], rule);
        }
    }
}
