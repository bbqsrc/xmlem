use crate::{key::Node, qname::QName};

#[derive(Debug, Clone)]
pub(crate) enum NodeValue {
    Element(ElementValue),
    Text(String),
    CData(String),
    Comment(String),
    DocumentType(String),
}

#[derive(Debug, Clone)]
pub struct ElementValue {
    pub(crate) name: QName,
    pub(crate) children: Vec<Node>,
}

impl NodeValue {
    pub fn as_element(&self) -> Option<&ElementValue> {
        match self {
            NodeValue::Element(e) => Some(e),
            _ => None,
        }
    }

    pub fn as_element_mut(&mut self) -> Option<&mut ElementValue> {
        match self {
            NodeValue::Element(e) => Some(e),
            _ => None,
        }
    }
}
