use indexmap::IndexMap;
use once_cell::sync::Lazy;

use crate::{
    document::Document,
    key::{CDataSection, Comment, DocKey, Node, Text},
    value::{ElementValue, ItemValue, NodeValue},
};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Element(pub(crate) DocKey);

#[derive(Debug, Clone)]
pub struct NewElement {
    pub name: String,
    pub attrs: IndexMap<String, String>,
}

impl Element {
    pub fn append_new_element(self, document: &mut Document, element: NewElement) -> Element {
        let new_key = document
            .items
            .insert(ItemValue::Node(NodeValue::Element(ElementValue {
                name: element.name,
                children: vec![],
            })));
        document.attrs.insert(new_key, element.attrs);
        document.parents.insert(new_key, self);
        document
            .items
            .get_mut(self.0)
            .unwrap()
            .as_element_mut()
            .unwrap()
            .children
            .push(Node::Element(Element(new_key)));
        Element(new_key)
    }

    pub fn append_text(self, document: &mut Document, text: &str) -> Text {
        let new_key = document
            .items
            .insert(ItemValue::Node(NodeValue::Text(text.to_string())));
        document.parents.insert(new_key, self);
        document
            .items
            .get_mut(self.0)
            .unwrap()
            .as_element_mut()
            .unwrap()
            .children
            .push(Node::Text(Text(new_key)));
        Text(new_key)
    }

    pub fn append_cdata(self, document: &mut Document, text: &str) -> CDataSection {
        let new_key = document
            .items
            .insert(ItemValue::Node(NodeValue::CData(text.to_string())));
        document.parents.insert(new_key, self);
        document
            .items
            .get_mut(self.0)
            .unwrap()
            .as_element_mut()
            .unwrap()
            .children
            .push(Node::CDataSection(CDataSection(new_key)));
        CDataSection(new_key)
    }

    pub fn append_comment(self, document: &mut Document, text: &str) -> Comment {
        let new_key = document
            .items
            .insert(ItemValue::Node(NodeValue::Comment(text.to_string())));
        document.parents.insert(new_key, self);
        document
            .items
            .get_mut(self.0)
            .unwrap()
            .as_element_mut()
            .unwrap()
            .children
            .push(Node::Comment(Comment(new_key)));
        Comment(new_key)
    }

    pub fn remove_child(self, document: &mut Document, node: Node) {
        let element = document
            .items
            .get_mut(self.0)
            .unwrap()
            .as_element_mut()
            .unwrap();
        match element.children.iter().position(|x| x == &node) {
            Some(i) => {
                element.children.remove(i);
            }
            None => return,
        }
        document.items.remove(node.as_key());
    }

    pub fn parent(self, document: &Document) -> Option<Element> {
        document.parents.get(self.0).copied()
    }

    pub fn children(self, document: &Document) -> &[Node] {
        let element = document.items.get(self.0).unwrap().as_element().unwrap();
        &element.children
    }

    pub fn attributes<'d>(&self, document: &'d Document) -> &'d IndexMap<String, String> {
        match document.attrs.get(self.0) {
            Some(x) => x,
            None => &EMPTY_INDEXMAP,
        }
    }

    pub fn attribute<'d>(&self, document: &'d Document, name: &str) -> Option<&'d str> {
        let attrs = self.attributes(document);
        attrs.get(name).map(|x| &**x)
    }

    pub fn set_attribute(&self, document: &mut Document, name: &str, value: &str) {
        if !document.attrs.contains_key(self.0) {
            document.attrs.insert(self.0, Default::default());
        }

        let attrs = document.attrs.get_mut(self.0).unwrap();
        attrs.insert(name.into(), value.into());
    }

    pub fn display<'d>(&self, document: &'d Document) -> String {
        let element = document.items.get(self.0).unwrap().as_element().unwrap();
        let mut s = Vec::<u8>::new();
        element
            .display(&document, self.0, &mut s, 0, false)
            .expect("Invalid string somehow");
        let s = String::from_utf8(s).expect("Invalid string somehow");
        s
    }
}

static EMPTY_INDEXMAP: Lazy<IndexMap<String, String>> = Lazy::new(|| IndexMap::new());
