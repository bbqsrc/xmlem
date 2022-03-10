use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::element::Element;

#[derive(Debug)]
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

impl Clone for Node {
    fn clone(&self) -> Self {
        match self {
            Node::Element(rccell_element) => {
                let inner_element = &*rccell_element.borrow();
                //Node::Element(Rc::new(RefCell::new(inner_element.clone()))
                todo!()
            }
            Node::Text(rccell_string) => {
                let inner_string = &*rccell_string.borrow();
                Node::Text(Rc::new(RefCell::new(inner_string.clone())))
            }
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone();
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
