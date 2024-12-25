use crate::error::Error;
use crate::renderer::dom::node::ElementKind;
use crate::renderer::dom::node::Node;
use crate::renderer::dom::node::NodeKind;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use core::cell::RefCell;

#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    background_color: Option<Color>,
    color: Option<Color>,
    display: Option<DisplayType>,
    font_size: Option<FontSize>,
    text_decoration: Option<TextDecoration>,
    height: Option<f64>,
    width: Option<f64>,
}

impl ComputedStyle {
    pub fn new() -> Self {
        Self {
            background_color: None,
            color: None,
            display: None,
            font_size: None,
            text_decoration: None,
            height: None,
            width: None,
        }
    }

    pub fn defaulting(&mut self, node: &Rc<RefCell<Node>>, parent_style: Option<ComputedStyle>) {
        // もし親ノードが存在し、親のCSS値が初期値と異なる場合は値を継承する
        if let Some(parent_style) = parent_style {
            if self.background_color.is_none() && parent_style.background_color() != Color::white()
            {
                self.background_color = Some(parent_style.background_color());
            }
            // 親要素のcolorプロパティがブラウザで定められた初期値（黒）ではない場合、ユーザの設定する値なので、親要素の値を継承する
            if self.color.is_none() && parent_style.color() != Color::black() {
                self.color = Some(parent_style.color());
            }
            if self.font_size.is_none() && parent_style.font_size() != FontSize::Medium {
                self.font_size = Some(parent_style.font_size());
            }
            if self.text_decoration.is_none()
                && parent_style.text_decoration() != TextDecoration::None
            {
                self.text_decoration = Some(parent_style.text_decoration());
            }
        }

        // 各プロパティに対して、初期値を設定する
        if self.background_color.is_none() {
            self.background_color = Some(Color::white());
        }
        if self.color.is_none() {
            self.color = Some(Color::black());
        }
        if self.display.is_none() {
            self.display = Some(DisplayType::default(node));
        }
        if self.font_size.is_none() {
            self.font_size = Some(FontSize::default(node));
        }
        if self.text_decoration.is_none() {
            self.text_decoration = Some(TextDecoration::default(node));
        }
        if self.height.is_none() {
            self.height = Some(0.0);
        }
        if self.width.is_none() {
            self.width = Some(0.0);
        }
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = Some(color);
    }

    pub fn background_color(&self) -> Color {
        self.background_color
            .clone()
            .expect("failed to access CSS property: background_color")
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = Some(color);
    }

    pub fn color(&self) -> Color {
        self.color
            .clone()
            .expect("failed to access CSS property: color")
    }

    pub fn set_display(&mut self, display: DisplayType) {
        self.display = Some(display);
    }

    pub fn display(&self) -> DisplayType {
        self.display
            .expect("failed to access CSS property: display")
    }

    pub fn font_size(&self) -> FontSize {
        self.font_size
            .expect("failed to access CSS property: font_size")
    }

    pub fn text_decoration(&self) -> TextDecoration {
        self.text_decoration
            .expect("failed to access CSS property: text_decoration")
    }

    pub fn set_height(&mut self, height: f64) {
        self.height = Some(height);
    }

    pub fn height(&self) -> f64 {
        self.height.expect("failed to access CSS property: height")
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = Some(width);
    }

    pub fn width(&self) -> f64 {
        self.width.expect("failed to access CSS property: width")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    name: Option<String>,
    code: String,
}

impl Color {
    pub fn from_name(name: &str) -> Result<Self, Error> {
        let code = match name {
            "black" => "#000000".to_string(),
            "silver" => "#c0c0c0".to_string(),
            "gray" => "#808080".to_string(),
            "white" => "#ffffff".to_string(),
            "maroon" => "#800000".to_string(),
            "red" => "#ff0000".to_string(),
            "purple" => "#800080".to_string(),
            "fuchsia" => "#ff00ff".to_string(),
            "green" => "#008000".to_string(),
            "lime" => "#00ff00".to_string(),
            "olive" => "#808000".to_string(),
            "yellow" => "#ffff00".to_string(),
            "navy" => "#000080".to_string(),
            "blue" => "#0000ff".to_string(),
            "teal" => "#008080".to_string(),
            "aqua" => "#00ffff".to_string(),
            "orange" => "#ffa500".to_string(),
            "lightgray" => "#d3d3d3".to_string(),
            _ => {
                return Err(Error::UnexpectedInput(format!(
                    "color name {:?} is not supported yet",
                    name
                )));
            }
        };

        Ok(Self {
            name: Some(name.to_string()),
            code,
        })
    }

    pub fn from_code(code: &str) -> Result<Self, Error> {
        if code.chars().nth(0) != Some('#') || code.len() != 7 {
            // #fff などの省略された形式のカラーコードは使用不可
            return Err(Error::UnexpectedInput(format!(
                "invalid color code {}",
                code
            )));
        }

        let name = match code {
            "#000000" => "black".to_string(),
            "#c0c0c0" => "silver".to_string(),
            "#808080" => "gray".to_string(),
            "#ffffff" => "white".to_string(),
            "#800000" => "maroon".to_string(),
            "#ff0000" => "red".to_string(),
            "#800080" => "purple".to_string(),
            "#ff00ff" => "fuchsia".to_string(),
            "#008000" => "green".to_string(),
            "#00ff00" => "lime".to_string(),
            "#808000" => "olive".to_string(),
            "#ffff00" => "yellow".to_string(),
            "#000080" => "navy".to_string(),
            "#0000ff" => "blue".to_string(),
            "#008080" => "teal".to_string(),
            "#00ffff" => "aqua".to_string(),
            "#ffa500" => "orange".to_string(),
            "#d3d3d3" => "lightgray".to_string(),
            _ => {
                return Err(Error::UnexpectedInput(format!(
                    "color code {:?} is not supported yet",
                    code
                )));
            }
        };

        Ok(Self {
            name: Some(name),
            code: code.to_string(),
        })
    }

    pub fn white() -> Self {
        Self {
            name: Some("white".to_string()),
            code: "#ffffff".to_string(),
        }
    }

    pub fn black() -> Self {
        Self {
            name: Some("black".to_string()),
            code: "#000000".to_string(),
        }
    }

    pub fn code_u32(&self) -> u32 {
        u32::from_str_radix(self.code.trim_start_matches('#'), 16).unwrap()
    }
}

/// https://www.w3.org/TR/css-fonts-4/#absolute-size-mapping
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FontSize {
    Medium,
    XLarge,
    XXLarge,
}

impl FontSize {
    fn default(node: &Rc<RefCell<Node>>) -> Self {
        match &node.borrow().kind() {
            NodeKind::Element(element) => match element.kind() {
                ElementKind::H1 => FontSize::XXLarge, // h1タグのデフォルトサイズ
                ElementKind::H2 => FontSize::XLarge,  // h2タグのデフォルトサイズ
                _ => FontSize::Medium,
            },
            _ => FontSize::Medium,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DisplayType {
    /// https://www.w3.org/TR/css-display-3/#valdef-display-block
    Block,
    /// https://www.w3.org/TR/css-display-3/#valdef-display-inline
    Inline,
    /// https://www.w3.org/TR/css-display-3/#valdef-display-none
    DisplayNone,
}

impl DisplayType {
    fn default(node: &Rc<RefCell<Node>>) -> Self {
        match &node.borrow().kind() {
            NodeKind::Document => DisplayType::Block,
            NodeKind::Element(e) => {
                if e.is_block_element() {
                    DisplayType::Block
                } else {
                    DisplayType::Inline
                }
            }
            NodeKind::Text(_) => DisplayType::Inline,
        }
    }

    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "block" => Ok(Self::Block),
            "inline" => Ok(Self::Inline),
            "none" => Ok(Self::DisplayNone),
            _ => Err(Error::UnexpectedInput(format!(
                "display {:?} is not supported yet",
                s
            ))),
        }
    }
}

/// https://w3c.github.io/csswg-drafts/css-text-decor/#text-decoration-property
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TextDecoration {
    None,
    Underline,
}

impl TextDecoration {
    fn default(node: &Rc<RefCell<Node>>) -> Self {
        match &node.borrow().kind() {
            NodeKind::Element(element) => match element.kind() {
                ElementKind::A => TextDecoration::Underline,
                _ => TextDecoration::None,
            },
            _ => TextDecoration::None,
        }
    }
}
