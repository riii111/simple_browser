use crate::renderer::dom::node::ElementKind;
use crate::renderer::dom::node::Node;
use crate::renderer::dom::node::Window;
use crate::renderer::html::token::HtmlTokenizer;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;

use super::token::HtmlToken;

/// パーサーは常に特定の「挿入モード」が存在する. このモードによって、現在のトークンをどのように処理するかが決定される
/// https://html.spec.whatwg.org/multipage/parsing.html#the-insertion-mode
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
    Text,
    AfterBody,
    AfterAfterBody,
}

/// DOMツリーを構築するための情報をもつ
#[derive(Debug, Clone)]
pub struct HtmlParser {
    window: Rc<RefCell<Window>>,
    mode: InsertionMode, // HTML構文解析中に使用される挿入モード（13.2.6 Tree constructionより）
    /// https://html.spec.whatwg.org/multipage/parsing.html#original-insertion-mode
    original_insertion_mode: InsertionMode, // 次の状態に遷移する際、以前の挿入モードを保持するために使用される
    /// https://html.spec.whatwg.org/multipage/parsing.html#the-stack-of-open-elements
    stack_of_open_elements: Vec<Rc<RefCell<Node>>>, // HTMLの構文解析中にブラウザが使用するスタック
    t: HtmlTokenizer,
}

impl HtmlParser {
    pub fn new(t: HtmlTokenizer) -> Self {
        Self {
            window: Rc::new(RefCell::new(Window::new())),
            mode: InsertionMode::Initial,
            original_insertion_mode: InsertionMode::Initial,
            stack_of_open_elements: Vec::new(),
            t,
        }
    }

    /// ステートマシンの実装
    pub fn construct_tree(&mut self) -> Rc<RefCell<Window>> {
        let mut token = self.t.next();

        while token.is_some() {
            match self.mode {
                /*
                書籍「ブラウザのしくみ」ではDOCTYPEトークンをサポートしていないため、
                <!doctype html>のようなトークンは文字トークンとして表される.
                文字トークンは無視する
                 */
                InsertionMode::Initial => {
                    if let Some(HtmlToken::Char(_)) = token {
                        token = self.t.next();
                        continue;
                    }

                    self.mode = InsertionMOde::BeforeHtml;
                    continue;
                }

                InsertionMode::BeforeHtml => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            // 空白、改行文字なら無視して次のトークンへ移動
                            if c == ' ' || c == '\n' {
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::StartTag {
                            ref tag,
                            self_closing: _,
                            ref attributes,
                        }) => {
                            // タグ名がhtmlなら、DOMに新たなノードを追加
                            if tag == "html" {
                                self.insert_element(tag, attributes.to_vec());
                                self.mode = InsertionMode::BeforeHead;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::EndTag { ref tag }) => {
                            if tag != "head" || tag != "body" || tag != "html" || tag != "br" {
                                token = self.t.next();
                                continue;
                            }
                        }
                        // トークン終了時は構築したDOMツリーを返す
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                    }
                    // DOMツリーにHTML要素を追加
                    self.insert_element("html", Vec::new());
                    self.mode = InsertionMode::BeforeHead;
                    continue;
                }

                InsertionMode::BeforeHead => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            // 空白、改行文字なら無視して次のトークンへ移動
                            if c == ' ' || c == '\n' {
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::StartTag {
                            ref tag,
                            self_closing: _,
                            ref attributes,
                        }) => {
                            // headならDOMツリーに新たなノードを追加
                            if tag == "head" {
                                self.insert_element(tag, attributes.to_vec());
                                self.mode = InsertionMode::InHead;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        _ => {}
                    }
                    // DOMツリーにHEAD要素を追加
                    self.insert_element("head", Vec::new());
                    self.mode = InsertionMode::InHead;
                    continue;
                }

                InsertionMode::InHead => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            // 空白、改行文字なら無視して次のトークンへ移動
                            if c == ' ' || c == '\n' {
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::StartTag {
                            ref tag,
                            self_closing: _,
                            ref attributes,
                        }) => {
                            // 2
                            if tag == "style" || tag == "script" {
                                self.insert_element(tag, attributes.to_vec());
                                self.original_insertion_mode = self.mode;
                                self.mode = InsertionMode::Text;
                                token = self.t.next();
                                continue;
                            }
                            /*
                            <head>が省略されているHTMLで無限ループが発生しないように対策
                            仕様書には無いが、このブラウザでは仕様を全網羅しているわけではないので必要
                            */
                            if tag == "body" {
                                self.pop_until(ElementKind::Head);
                                self.mode = InsertionMode::AfterHead;
                                continue;
                            }
                            if let Ok(_element_kind) = ElementKind::from_str(tag) {
                                self.pop_until(ElementKind::Head);
                                self.mode = InsertionMode::AfterHead;
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                    }
                    // <meta>や<title>などはこのブラウザでは非サポートタグなので無視
                    token = self.t.next();
                    continue;
                }

                InsertionMode::AfterHead => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            // 空白、改行文字なら無視して次のトークンへ移動
                            if c == ' ' || c == '\n' {
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::StartTag {
                            ref tag,
                            self_closing: _,
                            ref attributes,
                        }) => {
                            if tag == "body" {
                                self.insert_element(tag, attributes.to_vec());
                                token = self.t.next();
                                self.mode = InsertionMode::InBody;
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        _ => {}
                    }
                    self.insert_element("body", Vec::new());
                    self.mode = InsertionMode::InBody;
                    continue;
                }

                InsertionMode::InBody => {
                    match token {
                        Some(HtmlToken::EndTag { ref tag }) => {
                            match tag.as_str() {
                                "body" => {
                                    self.mode = InsertionMode::AfterBody;
                                    token = self.t.next();
                                    if !self.contain_in_stack(ElementKind::Body) {
                                        // パースの失敗. トークンを無視する
                                        continue;
                                    }
                                    self.pop_until(ElementKind::Body);
                                    continue;
                                }
                                "html" => {
                                    if self.pop_current_node(ElementKind::Body) {
                                        self.mode = InsertionMode::AfterBody;
                                        assert!(self.pop_current_node(ElementKind::Html));
                                    } else {
                                        token = self.t.next();
                                    }
                                    continue;
                                }
                                _ => {
                                    token = self.t.next();
                                }
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        _ => {}
                    }
                }

                InsertionMode::Text => {
                    match token {
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        Some(HtmlToken::EndTag { ref tag }) => {
                            // style終了タグか、script終了タグが出現したら元の状態（original_insertion_mode）に戻る
                            if tag == "style" {
                                self.pop_until(ElementKind::Style);
                                self.mode = self.original_insertion_mode;
                                token = self.t.next();
                                continue;
                            }
                            if tag == "script" {
                                self.pop_until(ElementKind::Script);
                                self.mode = self.original_insertion_mode;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Char(c)) => {
                            self.insert_char(c);
                            token = self.t.next();
                            continue;
                        }
                        _ => {}
                    }

                    self.mode = self.original_insertion_mode;
                }

                InsertionMode::AfterBody => {
                    match token {
                        // 文字トークンなら無視
                        Some(HtmlToken::Char(_c)) => {
                            token = self.t.next();
                            continue;
                        }
                        Some(HtmlToken::EndTag { ref tag }) => {
                            if tag == "html" {
                                self.mode = InsertionMode::AfterAfterBody;
                                token = self.t.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        _ => {}
                    }

                    self.mode = InsertionMode::InBody;
                }

                InsertionMode::AfterAfterBody => {
                    match token {
                        // 文字トークンなら無視
                        Some(HtmlToken::Char(_c)) => {
                            token = self.t.next();
                            continue;
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        _ => {}
                    }
                    // パースの失敗
                    self.mode = InsertionMode::InBody;
                }
            }
        }

        self.window.clone()
    }
}
