use crate::key::Node;

#[derive(Debug)]
pub(crate) enum NodeValue {
    Element(ElementValue),
    Text(String),
    CData(String),
}

#[derive(Debug)]
pub(crate) enum ItemValue {
    Node(NodeValue),
}

#[derive(Debug)]
pub(crate) struct ElementValue {
    pub(crate) name: String,
    pub(crate) children: Vec<Node>,
}

impl ItemValue {
    pub fn as_node(&self) -> Option<&NodeValue> {
        match self {
            ItemValue::Node(n) => Some(n),
        }
    }

    pub fn as_element(&self) -> Option<&ElementValue> {
        match self {
            ItemValue::Node(n) => match n {
                NodeValue::Element(e) => Some(e),
                _ => None,
            },
        }
    }

    pub fn as_element_mut(&mut self) -> Option<&mut ElementValue> {
        match self {
            ItemValue::Node(n) => match n {
                NodeValue::Element(e) => Some(e),
                _ => None,
            },
        }
    }
}
