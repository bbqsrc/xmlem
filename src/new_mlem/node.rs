use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::element::Element;

#[derive(Debug)]
pub struct Text {
    inner_text: Rc<RefCell<String>>,
}

impl Clone for Text {
    fn clone(&self) -> Self {
        let borrow = RefCell::borrow(&*self.inner_text);

        Self {
            inner_text: Rc::new(RefCell::new(borrow.clone())),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone();
    }
}

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&RefCell::borrow(&*self.inner_text), f)
    }
}

#[derive(Debug)]
pub enum Node {
    Element(Element),
    Text(Text),
}

impl Clone for Node {
    fn clone(&self) -> Self {
        match self {
            Node::Element(element) => Node::Element(element.clone()),
            Node::Text(text) => Node::Text(text.clone()),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone();
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Element(element) => Display::fmt(element, f),
            Node::Text(text) => Display::fmt(text, f),
        }
    }
}
