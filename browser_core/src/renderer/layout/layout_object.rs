use crate::renderer::css::cssom::StyleSheet;
use crate::renderer::dom::node::Node;
use crate::renderer::layout::computed_style::ComputedStyle;
use alloc::rc::Rc;
use alloc::rc::Weak;
use core::cell::RefCell;

use super::computed_style::DisplayType;

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct LayoutPoint {
    x: i64,
    y: i64
}

impl LayoutPoint {
    pub fn new(x: i64, y: i64) -> Self {
        Self {x, y}
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
    height: i64
}

impl LayoutSize {
    pub fn new(width: i64, height: i64) -> Self {
        Self {width, height}
    }

    pub fn width(&self)-> i64 {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LayoutObjectKind {
    Block,
    Inline,
    Text
}

#[derive(Debug, Clone)]
pub struct LayoutObject {
    kind: LayoutObjectKind,
    node: Rc<RefCell<Node>>,
    first_child: Option<Rc<RefCell<LayoutObject>>>,
    next_sibling: Option<Rc<RefCell<LayoutObject>>>,
    parent: Weak<RefCell<LayoutObject>,
    style: ComputedStyle,
    point: LayoutPoint,
    size: LayoutSize,
}

impl LayoutObject {
    pub fn new(
        node: Rc<RefCell<Node>>,parent_obj: &Option<Rc<RefCell<LayoutObject>>>
    ) -> Self {
        let parent = match parent_obj {
            Some(p) => Rc::downgrade(p),
            None => Weak::new()
        };

        Self {
            kind: LayoutObjectKind::Block,
            node: node.clone(),
            first_child: None,
            next_sibling: None,
            parent,
            style: ComputedStyle::new(),
            point: LayoutPoint::new(0,0),
            size: LayoutSize::new(0,0)
        }
    }

    pub fn create_layout_object(node: &Option<Rc<RefCell<Node>>>, parent_obj: &Option<Rc<RefCell<LayoutObject>>>, cssom: &StyleSheet) -> Option<Rc<RefCell<LayoutObject>>> {
        if let Some(n) = node {
            let layout_object = Rc::new(RefCell::new(LayoutObject::new(n.clone(), parent_obj)));

            // CSSのルールを、セレクタで選択されたノードに適用
            for rule in &cssom.rules {
                if layout_object.borrow().is_node_selected(&rule.selector) {
                    layout_object.borrow_mut().cascading_style(rule.declarations.clone());
                }
            }

            // CSSでスタイルが指定されていない場合、デフォルト値または親ノードから継承した値を使用
            let parent_style = if let Some(parent) = parent_obj {
                Some(parent.borrow().style())
            } else {
                None
            };
            layout_object.borrow_mut().defaulting_style(n, parent_style);

            // displayプロパティがnoneの場合、ノードを生成しない
            if layout_object.borrow().style().display() == DisplayType::DisplayNone {
                return None;
            }

            // displayプロパティの最終的な値を用いてノードの種類を決定
            layout_object.borrow_mut().update_kind();
            return Some(layout_object);
        }
        None
    }

    pub fn kind(&self) -> LayoutObjectKind {
        self.kind
    }

    pub fn node_kind(&self)-> NodeKind {
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