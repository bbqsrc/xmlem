use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use super::element::Element;

#[derive(Debug)]
pub struct Text<'a> {
    inner_text: Rc<RefCell<String>>,

    pub parent: Option<&'a Element<'a>>,
}

impl Clone for Text<'_> {
    fn clone(&self) -> Self {
        let borrow = RefCell::borrow(&*self.inner_text);

        Self {
            inner_text: Rc::new(RefCell::new(borrow.clone())),

            parent: None,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone();
    }
}

impl Display for Text<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&RefCell::borrow(&*self.inner_text), f)
    }
}

#[derive(Debug)]
pub enum Node<'a> {
    Element(Element<'a>),
    Text(Text<'a>),
}

impl Clone for Node<'_> {
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

impl Display for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Element(element) => Display::fmt(element, f),
            Node::Text(text) => Display::fmt(text, f),
        }
    }
}
