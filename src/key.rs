use slotmap::new_key_type;

use crate::element::Element;

new_key_type! {
    pub(crate) struct DocKey;
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Text(pub(crate) DocKey);

#[derive(Debug, Copy, PartialEq, Eq, Clone)]
pub enum Node {
    Element(Element),
    Text(Text),
}

impl Node {
    pub(crate) fn as_key(self) -> DocKey {
        match self {
            Node::Element(e) => e.0,
            Node::Text(t) => t.0,
        }
    }
}
