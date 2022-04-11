use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Display;

use crate::element::Element;

#[derive(Debug)]
pub struct Text {
    inner_text: Rc<RefCell<String>>,
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

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Element(element) => Display::fmt(element, f),
            Node::Text(text) => Display::fmt(text, f),
        }
    }
}
