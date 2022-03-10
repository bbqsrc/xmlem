use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::element::Element;

#[derive(Debug, Clone)]
pub enum Node {
    Element(Rc<RefCell<Element>>),
    Text(Rc<RefCell<String>>),
}

impl From<Rc<RefCell<Element>>> for Node {
    fn from(x: Rc<RefCell<Element>>) -> Self {
        Self::Element(x)
    }
}

impl From<Rc<RefCell<String>>> for Node {
    fn from(x: Rc<RefCell<String>>) -> Self {
        Self::Text(x)
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Element(x) => Display::fmt(&RefCell::borrow(x), f),
            Node::Text(x) => Display::fmt(&RefCell::borrow(x), f),
        }
    }
}
