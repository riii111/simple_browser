use crate::alloc::string::ToString;
use crate::constants::{
    CHAR_HEIGHT_WITH_PADDING, CHAR_WIDTH, CONTENT_AREA_WIDTH, WINDOW_PADDING, WINDOW_WIDTH,
};
use crate::display_item::DisplayItem;
use crate::renderer::css::cssom::ComponentValue;
use crate::renderer::css::cssom::Declaration;
use crate::renderer::css::cssom::Selector;
use crate::renderer::css::cssom::StyleSheet;
use crate::renderer::dom::node::Node;
use crate::renderer::dom::node::NodeKind;
use crate::renderer::layout::computed_style::Color;
use crate::renderer::layout::computed_style::ComputedStyle;
use crate::renderer::layout::computed_style::DisplayType;
use crate::renderer::layout::computed_style::FontSize;
use alloc::rc::Rc;
use alloc::rc::Weak;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;

/// https://drafts.csswg.org/css-text/#word-break-property
fn find_index_for_line_break(line: String, max_index: usize) -> usize {
    for i in (0..max_index).rev() {
        if line.chars().collect::<Vec<char>>()[i] == ' ' {
            return i;
        }
    }
    max_index
}

/// https://drafts.csswg.org/css-text/#word-break-property
fn split_text(line: String, char_width: i64) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    if line.len() as i64 * char_width > (WINDOW_WIDTH + WINDOW_PADDING) {
        let s = line.split_at(find_index_for_line_break(
            line.clone(),
            ((WINDOW_WIDTH + WINDOW_PADDING) / char_width) as usize,
        ));
        result.push(s.0.to_string());
        result.extend(split_text(s.1.trim().to_string(), char_width))
    } else {
        result.push(line);
    }
    result
}
pub fn create_layout_object(
    node: &Option<Rc<RefCell<Node>>>,
    parent_obj: &Option<Rc<RefCell<LayoutObject>>>,
    cssom: &StyleSheet,
) -> Option<Rc<RefCell<LayoutObject>>> {
    if let Some(n) = node {
        // LayoutObjectを作成する
        let layout_object = Rc::new(RefCell::new(LayoutObject::new(n.clone(), parent_obj)));

        // CSSのルールをセレクタで選択されたノードに適用する
        for rule in &cssom.rules {
            if layout_object.borrow().is_node_selected(&rule.selector) {
                layout_object
                    .borrow_mut()
                    .cascading_style(rule.declarations.clone());
            }
        }

        // CSSでスタイルが指定されていない場合、デフォルトの値または親のノードから継承した値を使用する
        let parent_style = parent_obj.as_ref().map(|parent| parent.borrow().style());
        layout_object.borrow_mut().defaulting_style(n, parent_style);

        // displayプロパティがnoneの場合、ノードを作成しない
        if layout_object.borrow().style().display() == DisplayType::DisplayNone {
            return None;
        }

        // displayプロパティの最終的な値を使用してノードの種類を決定する
        layout_object.borrow_mut().update_kind();
        return Some(layout_object);
    }
    None
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LayoutObjectKind {
    Block,
    Inline,
    Text,
}

#[derive(Debug, Clone)]
pub struct LayoutObject {
    kind: LayoutObjectKind,
    node: Rc<RefCell<Node>>,
    first_child: Option<Rc<RefCell<LayoutObject>>>,
    next_sibling: Option<Rc<RefCell<LayoutObject>>>,
    parent: Weak<RefCell<LayoutObject>>,
    style: ComputedStyle,
    point: LayoutPoint,
    size: LayoutSize,
}

impl PartialEq for LayoutObject {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl LayoutObject {
    pub fn new(node: Rc<RefCell<Node>>, parent_obj: &Option<Rc<RefCell<LayoutObject>>>) -> Self {
        let parent = match parent_obj {
            Some(p) => Rc::downgrade(p),
            None => Weak::new(),
        };

        Self {
            kind: LayoutObjectKind::Block,
            node: node.clone(),
            first_child: None,
            next_sibling: None,
            parent,
            style: ComputedStyle::new(),
            point: LayoutPoint::new(0, 0),
            size: LayoutSize::new(0, 0),
        }
    }

    pub fn paint(&mut self) -> Vec<DisplayItem> {
        if self.style.display() == DisplayType::DisplayNone {
            return vec![];
        }

        match self.kind {
            LayoutObjectKind::Block => {
                // (d1)
                if let NodeKind::Element(_e) = self.node_kind() {
                    return vec![DisplayItem::Rect {
                        style: self.style(),
                        layout_point: self.point(),
                        layout_size: self.size(),
                    }];
                }
            }
            LayoutObjectKind::Inline => { // (d2)
                 // 本書のブラウザでは、描画するインライン要素はない。
                 // <img>タグなどをサポートした場合はこのアームの中で処理をする
            }
            LayoutObjectKind::Text => {
                // (d3)
                if let NodeKind::Text(t) = self.node_kind() {
                    let mut v = vec![];

                    let ratio = match self.style.font_size() {
                        FontSize::Medium => 1,
                        FontSize::XLarge => 2,
                        FontSize::XXLarge => 3,
                    };
                    let plain_text = t
                        .replace('\n', " ")
                        .split(' ')
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ");
                    let lines = split_text(plain_text, CHAR_WIDTH * ratio);
                    for (i, line) in lines.into_iter().enumerate() {
                        let item = DisplayItem::Text {
                            text: line,
                            style: self.style(),
                            layout_point: LayoutPoint::new(
                                self.point().x(),
                                self.point().y() + CHAR_HEIGHT_WITH_PADDING * i as i64,
                            ),
                        };
                        v.push(item);
                    }

                    return v;
                }
            }
        }

        vec![]
    }

    pub fn compute_size(&mut self, parent_size: LayoutSize) {
        let mut size = LayoutSize::new(0, 0);

        match self.kind() {
            LayoutObjectKind::Block => {
                size.set_width(parent_size.width());

                // 全ての子ノードの高さを足し合わせた結果が高さになる。
                // ただし、インライン要素が横に並んでいる場合は注意が必要
                let mut height = 0;
                let mut child = self.first_child();
                let mut previous_child_kind = LayoutObjectKind::Block;
                while child.is_some() {
                    let c = match child {
                        Some(c) => c,
                        None => panic!("first child should exist"),
                    };

                    if previous_child_kind == LayoutObjectKind::Block
                        || c.borrow().kind() == LayoutObjectKind::Block
                    {
                        height += c.borrow().size.height();
                    }

                    previous_child_kind = c.borrow().kind();
                    child = c.borrow().next_sibling();
                }
                size.set_height(height);
            }
            LayoutObjectKind::Inline => {
                // 全ての子ノードの高さと横幅を足し合わせた結果が現在のノードの高さと横幅とになる
                let mut width = 0;
                let mut height = 0;
                let mut child = self.first_child();
                while child.is_some() {
                    let c = match child {
                        Some(c) => c,
                        None => panic!("first child should exist"),
                    };

                    width += c.borrow().size.width();
                    height += c.borrow().size.height();

                    child = c.borrow().next_sibling();
                }

                size.set_width(width);
                size.set_height(height);
            }
            LayoutObjectKind::Text => {
                if let NodeKind::Text(t) = self.node_kind() {
                    let ratio = match self.style.font_size() {
                        FontSize::Medium => 1,
                        FontSize::XLarge => 2,
                        FontSize::XXLarge => 3,
                    };
                    let width = CHAR_WIDTH * ratio * t.len() as i64;
                    if width > CONTENT_AREA_WIDTH {
                        // テキストが複数行のとき
                        size.set_width(CONTENT_AREA_WIDTH);
                        let line_num = if width.wrapping_rem(CONTENT_AREA_WIDTH) == 0 {
                            width.wrapping_div(CONTENT_AREA_WIDTH)
                        } else {
                            width.wrapping_div(CONTENT_AREA_WIDTH) + 1
                        };
                        size.set_height(CHAR_HEIGHT_WITH_PADDING * ratio * line_num);
                    } else {
                        // テキストが1行に収まるとき
                        size.set_width(width);
                        size.set_height(CHAR_HEIGHT_WITH_PADDING * ratio);
                    }
                }
            }
        }

        self.size = size;
    }

    pub fn compute_position(
        &mut self,
        parent_point: LayoutPoint,
        previous_sibiling_kind: LayoutObjectKind,
        previous_sibiling_point: Option<LayoutPoint>,
        previous_sibiling_size: Option<LayoutSize>,
    ) {
        let mut point = LayoutPoint::new(0, 0);

        match (self.kind(), previous_sibiling_kind) {
            // もしブロック要素が兄弟ノードの場合、Y軸方向に進む
            (LayoutObjectKind::Block, _) | (_, LayoutObjectKind::Block) => {
                if let (Some(size), Some(pos)) = (previous_sibiling_size, previous_sibiling_point) {
                    point.set_y(pos.y() + size.height());
                } else {
                    point.set_y(parent_point.y());
                }
                point.set_x(parent_point.x());
            }
            // もしインライン要素が並ぶ場合、X軸方向に進む
            (LayoutObjectKind::Inline, LayoutObjectKind::Inline) => {
                if let (Some(size), Some(pos)) = (previous_sibiling_size, previous_sibiling_point) {
                    point.set_x(pos.x() + size.width());
                    point.set_y(pos.y());
                } else {
                    point.set_x(parent_point.x());
                    point.set_y(parent_point.y());
                }
            }
            _ => {
                point.set_x(parent_point.x());
                point.set_y(parent_point.y());
            }
        }

        self.point = point;
    }

    pub fn is_node_selected(&self, selector: &Selector) -> bool {
        match &self.node_kind() {
            NodeKind::Element(e) => match selector {
                Selector::TypeSelector(type_name) => {
                    if e.kind().to_string() == *type_name {
                        return true;
                    }
                    false
                }
                Selector::ClassSelector(class_name) => {
                    for attr in &e.attributes() {
                        if attr.name() == "class" && attr.value() == *class_name {
                            return true;
                        }
                    }
                    false
                }
                Selector::IdSelector(id_name) => {
                    for attr in &e.attributes() {
                        if attr.name() == "id" && attr.value() == *id_name {
                            return true;
                        }
                    }
                    false
                }
                Selector::UnknownSelector => false,
            },
            _ => false,
        }
    }

    pub fn cascading_style(&mut self, declarations: Vec<Declaration>) {
        for declaration in declarations {
            match declaration.property.as_str() {
                "background-color" => {
                    if let ComponentValue::Ident(value) = &declaration.value {
                        let color = match Color::from_name(value) {
                            Ok(color) => color,
                            Err(_) => Color::white(),
                        };
                        self.style.set_background_color(color);
                        continue;
                    }

                    if let ComponentValue::HashToken(color_code) = &declaration.value {
                        let color = match Color::from_code(color_code) {
                            Ok(color) => color,
                            Err(_) => Color::white(),
                        };
                        self.style.set_background_color(color);
                        continue;
                    }
                }
                "color" => {
                    if let ComponentValue::Ident(value) = &declaration.value {
                        let color = match Color::from_name(value) {
                            Ok(color) => color,
                            Err(_) => Color::black(),
                        };
                        self.style.set_color(color);
                    }

                    if let ComponentValue::HashToken(color_code) = &declaration.value {
                        let color = match Color::from_code(color_code) {
                            Ok(color) => color,
                            Err(_) => Color::black(),
                        };
                        self.style.set_color(color);
                    }
                }
                "display" => {
                    if let ComponentValue::Ident(value) = declaration.value {
                        let display_type = match DisplayType::from_str(&value) {
                            Ok(display_type) => display_type,
                            Err(_) => DisplayType::DisplayNone,
                        };
                        self.style.set_display(display_type)
                    }
                }
                _ => {}
            }
        }
    }

    pub fn defaulting_style(
        &mut self,
        node: &Rc<RefCell<Node>>,
        parent_style: Option<ComputedStyle>,
    ) {
        self.style.defaulting(node, parent_style);
    }

    pub fn update_kind(&mut self) {
        match self.node_kind() {
            NodeKind::Document => panic!("should not create a layout object for a Document node"),
            NodeKind::Element(_) => {
                let display = self.style.display();
                match display {
                    DisplayType::Block => self.kind = LayoutObjectKind::Block,
                    DisplayType::Inline => self.kind = LayoutObjectKind::Inline,
                    DisplayType::DisplayNone => {
                        panic!("should not create a layout object for display:none")
                    }
                }
            }
            NodeKind::Text(_) => self.kind = LayoutObjectKind::Text,
        }
    }

    pub fn kind(&self) -> LayoutObjectKind {
        self.kind
    }

    pub fn node_kind(&self) -> NodeKind {
        self.node.borrow().kind().clone()
    }

    pub fn set_first_child(&mut self, first_child: Option<Rc<RefCell<LayoutObject>>>) {
        self.first_child = first_child;
    }

    pub fn first_child(&self) -> Option<Rc<RefCell<LayoutObject>>> {
        self.first_child.as_ref().cloned()
    }

    pub fn set_next_sibling(&mut self, next_sibling: Option<Rc<RefCell<LayoutObject>>>) {
        self.next_sibling = next_sibling;
    }

    pub fn next_sibling(&self) -> Option<Rc<RefCell<LayoutObject>>> {
        self.next_sibling.as_ref().cloned()
    }

    pub fn parent(&self) -> Weak<RefCell<Self>> {
        self.parent.clone()
    }

    pub fn style(&self) -> ComputedStyle {
        self.style.clone()
    }

    pub fn point(&self) -> LayoutPoint {
        self.point
    }

    pub fn size(&self) -> LayoutSize {
        self.size
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct LayoutPoint {
    x: i64,
    y: i64,
}

impl LayoutPoint {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> i64 {
        self.x
    }

    pub fn y(&self) -> i64 {
        self.y
    }

    pub fn set_x(&mut self, x: i64) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: i64) {
        self.y = y;
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct LayoutSize {
    width: i64,
    height: i64,
}

impl LayoutSize {
    pub fn new(width: i64, height: i64) -> Self {
        Self { width, height }
    }

    pub fn width(&self) -> i64 {
        self.width
    }

    pub fn height(&self) -> i64 {
        self.height
    }

    pub fn set_width(&mut self, width: i64) {
        self.width = width;
    }

    pub fn set_height(&mut self, height: i64) {
        self.height = height;
    }
}
