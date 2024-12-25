use crate::constants::CONTENT_AREA_WIDTH;
use crate::renderer::css::cssom::StyleSheet;
use crate::renderer::dom::api::get_target_element_node;
use crate::renderer::dom::node::ElementKind;
use crate::renderer::dom::node::Node;
use crate::renderer::layout::layout_object::{LayoutObject, LayoutPoint, LayoutSize};
use alloc::rc::Rc;
use core::cell::RefCell;

use super::layout_object::LayoutObjectKind;

#[derive(Debug, Clone)]
pub struct LayoutView {
    root: Option<Rc<RefCell<LayoutObject>>>,
}

impl LayoutView {
    pub fn new(root: Rc<RefCell<Node>>, cssom: &StyleSheet) -> Self {
        // レイアウトツリーは描画される要素のみを持つツリーなので、<body>タグを取得し
        // その子要素以下をレイアウトツリーのノードに変換する
        let body_root = get_target_element_node(Some(root), ElementKind::Body);

        let mut tree = Self {
            root: build_layout_tree(&body_root, &None, cssom),
        };

        tree.update_layout();

        tree
    }

    pub fn root(&self) -> Option<Rc<RefCell<LayoutObject>>> {
        self.root.clone()
    }

    fn update_layout(&mut self) {
        Self::calculate_node_size(&self.root, LayoutSize::new(CONTENT_AREA_WIDTH, 0));

        Self::calculate_node_position(
            &self,
            root,
            LayoutPoint::new(0, 0),
            LayoutObjectKind::Block,
            None,
            None,
        )
    }

    fn calculate_node_size(node: &Option<Rc<RefCell<LayoutObject>>>, parent_size: LayoutSize) {
        if let Some(n) = node {
            // ノードがブロック要素の場合、子ノードのレイアウトを計算する前に横幅を決める
            if n.borrow().kind() == LayoutObjectKind::Block {
                n.borrow_mut().compute_size(parent_size); // ブロック要素は親の横幅いっぱいまで広がるので、親ノードの横幅と同等扱いでOK
            }

            let first_child = n.borrow().first_child();
            Self::calculate_node_size(&first_child, n.borrow().size());

            let next_sibling = n.borrow().next_sibling();
            Self::calculate_node_size(&next_sibling, parent_size);

            // 子ノードのサイズが決まってからサイズを計算する
            // ブロック要素なら高さは子ノードに依存
            // インライン要素なら高さも横幅も子ノードに依存
            n.borrow_mut().compute_size(parent_size);
        }
    }
}

/// 再帰的に呼び出しレイアウトツリーを構築していく
/// DOMツリーをルートノードから走査しながらDOMノードからレイアウトオブジェクトを作成する
fn build_layout_tree(
    node: &Option<Rc<RefCell<Node>>>,
    parent_obj: &Option<Rc<RefCell<LayoutObject>>>,
    cssom: &StyleSheet,
) -> Option<Rc<RefCell<LayoutObject>>> {
    // DOMノードに対応するレイアウトオブジェクトを生成
    // ただし"display:none"が指定されていた場合、ノードは生成しない
    let mut target_node = node.clone();
    let mut layout_object = LayoutObject::create_layout_object(node, parent_obj, cssom);

    // ノードが生成されなかった場合、兄弟ノードを使用してレイアウトオブジェクトの生成を試みる（生成されるまで兄弟ノードを辿り続ける）
    while layout_object.is_none() {
        if let Some(n) = target_node {
            target_node = n.borrow().next_sibling().clone();
            layout_object = LayoutObject::create_layout_object(&target_node, parent_obj, cssom);
        } else {
            // 兄弟ノードがなくなれば、処理すべきDOMツリーは終了したのでこれまでのレイアウトツリーを返す
            return layout_object;
        }
    }

    if let Some(n) = target_node {
        let original_first_child = n.borrow().first_child();
        let original_next_sibling = n.borrow().next_sibling();
        // 子ノード・兄弟ノードに対して再帰的にレイアウトオブジェクトを生成
        let mut first_child = build_layout_tree(&original_first_child, layout_object, cssom);
        let mut next_sibling = build_layout_tree(&original_next_sibling, &None, cssom);

        // ノードが生成されなかった場合、兄弟ノードを使用してレイアウトオブジェクトの生成を試みる（生成されるまで兄弟ノードを辿り続ける）
        if first_child.is_none() && original_first_child.is_some() {
            let mut original_dom_node = original_first_child
                .expect("first child should exist")
                .borrow()
                .next_sibling();

            loop {
                first_child = build_layout_tree(&original_dom_node, &layout_object, cssom);

                if first_child.is_none() && original_dom_node.is_some() {
                    original_dom_node = original_dom_node
                        .expect("next sibling should exist")
                        .borrow()
                        .next_sibling();
                    continue;
                }

                break;
            }
        }
        if next_sibling.is_none() && n.borrow().next_sibling().is_some() {
            let mut original_dom_node = original_next_sibling
                .expect("first child should exist")
                .borrow()
                .next_sibling();

            loop {
                next_sibling = build_layout_tree(&original_dom_node, &None, cssom);

                if next_sibling.is_none() && original_dom_node.is_some() {
                    original_dom_node = original_dom_node
                        .expect("next sibling should exist")
                        .borrow()
                        .next_sibling();
                    continue;
                }

                break;
            }
        }

        let obj = match layout_object {
            Some(ref obj) => obj,
            None => panic!("render object should exist here"),
        };
        obj.borrow_mut().set_first_child(first_child);
        obj.borrow_mut().set_next_sibling(next_sibling);
    }

    layout_object
}
