use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::Element;

use crate::node::Node;

pub struct Document {
    root: Rc<RefCell<Element>>,
}

impl Document {
    pub fn from_str(s: &str) -> Result<Document, quick_xml::Error> {
        let root = Element::from_str(s)?;

        Ok(Document { root })
    }
}

impl Clone for Document {
    fn clone(&self) -> Self {
        let borrow_root = &*self.root.borrow();

        let childrens = borrow_root.children();

        for child in &mut *childrens.borrow_mut() {
            parentify(child, self.root.clone())
        }

        Self {
            root: Rc::new(RefCell::new(borrow_root.clone())),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone();
    }
}

fn parentify(child: &mut Node, parent: Rc<RefCell<Element>>) {
    match child {
        Node::Element(elem) => {
            {
                let borrowed_elem = &mut *elem.borrow_mut();
                borrowed_elem.set_parent(parent);
            }

            let borrowed_elem = &*elem.borrow();

            let childrens = borrowed_elem.children();

            for child in &mut *childrens.borrow_mut() {
                let repacked_elem = elem.clone();

                parentify(child, repacked_elem);
            }
        }
        Node::Text(_) => {}
        Node::CData(_) => {}
    };
}

impl Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", &*self.root.borrow()))
    }
}
