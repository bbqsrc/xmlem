use indexmap::IndexMap;
use once_cell::sync::Lazy;
use qname::QName;

use crate::{
    display::{self, Print},
    document::Document,
    key::{CDataSection, Comment, DocKey, Node, Text},
    select::Selector,
    value::{ElementValue, NodeValue},
};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Element(pub(crate) DocKey);

impl From<Element> for Node {
    fn from(n: Element) -> Self {
        Node::Element(n)
    }
}

#[derive(Debug, Clone)]
pub struct NewElement {
    pub name: QName,
    pub attrs: IndexMap<QName, String>,
}

impl<const N: usize, T: ToString, U: ToString, V: ToString> From<(T, [(U, V); N])> for NewElement {
    fn from(x: (T, [(U, V); N])) -> Self {
        NewElement {
            name: x.0.to_string().parse().unwrap(),
            attrs: x
                .1
                .into_iter()
                .map(|x| (x.0.to_string().parse().unwrap(), x.1.to_string()))
                .collect(),
        }
    }
}

impl Element {
    pub fn as_node(&self) -> Node {
        Node::from(*self)
    }

    pub fn append_element(self, document: &mut Document, element: Element) {
        if let Some(parent) = element.parent(&document) {
            parent.remove_child(document, Node::Element(element));
        }

        document.parents.insert(element.0, self);
        document
            .nodes
            .get_mut(self.0)
            .unwrap()
            .as_element_mut()
            .unwrap()
            .children
            .push(Node::Element(element));
    }

    pub fn append_new_element(
        self,
        document: &mut Document,
        element: impl Into<NewElement>,
    ) -> Element {
        let element = element.into();
        let new_key = document.nodes.insert(NodeValue::Element(ElementValue {
            name: element.name,
            children: vec![],
        }));
        document.attrs.insert(new_key, element.attrs);
        document.parents.insert(new_key, self);
        document
            .nodes
            .get_mut(self.0)
            .unwrap()
            .as_element_mut()
            .unwrap()
            .children
            .push(Node::Element(Element(new_key)));
        Element(new_key)
    }

    pub fn append_new_element_after(
        self,
        document: &mut Document,
        new_element: impl Into<NewElement>,
    ) -> Element {
        let element = new_element.into();
        let new_key = document.nodes.insert(NodeValue::Element(ElementValue {
            name: element.name,
            children: vec![],
        }));
        document.attrs.insert(new_key, element.attrs);

        let parent = self.parent(document).expect("no parent");
        document.parents.insert(new_key, parent);

        let children = &mut document
            .nodes
            .get_mut(parent.0)
            .unwrap()
            .as_element_mut()
            .unwrap()
            .children;

        let index = children.iter().position(|x| x == &self.as_node()).unwrap() + 1;
        if index >= children.len() {
            children.push(Node::Element(Element(new_key)));
        } else {
            children.insert(index, Node::Element(Element(new_key)))
        }

        Element(new_key)
    }

    pub fn append_text(self, document: &mut Document, text: &str) -> Text {
        let new_key = document.nodes.insert(NodeValue::Text(text.to_string()));
        document.parents.insert(new_key, self);
        document
            .nodes
            .get_mut(self.0)
            .unwrap()
            .as_element_mut()
            .unwrap()
            .children
            .push(Node::Text(Text(new_key)));
        Text(new_key)
    }

    pub fn append_cdata(self, document: &mut Document, text: &str) -> CDataSection {
        let new_key = document.nodes.insert(NodeValue::CData(text.to_string()));
        document.parents.insert(new_key, self);
        document
            .nodes
            .get_mut(self.0)
            .unwrap()
            .as_element_mut()
            .unwrap()
            .children
            .push(Node::CDataSection(CDataSection(new_key)));
        CDataSection(new_key)
    }

    pub fn append_comment(self, document: &mut Document, text: &str) -> Comment {
        let new_key = document.nodes.insert(NodeValue::Comment(text.to_string()));
        document.parents.insert(new_key, self);
        document
            .nodes
            .get_mut(self.0)
            .unwrap()
            .as_element_mut()
            .unwrap()
            .children
            .push(Node::Comment(Comment(new_key)));
        Comment(new_key)
    }

    pub fn set_text(self, document: &mut Document, text: &str) {
        let new_key = document.nodes.insert(NodeValue::Text(text.to_string()));
        document
            .nodes
            .get_mut(self.0)
            .unwrap()
            .as_element_mut()
            .unwrap()
            .children = vec![Node::Text(Text(new_key))];
    }

    pub fn remove_child(self, document: &mut Document, node: Node) {
        let element = document
            .nodes
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
        document.parents.remove(node.as_key());
    }

    pub fn parent(self, document: &Document) -> Option<Element> {
        document.parents.get(self.0).copied()
    }

    pub fn child_nodes(self, document: &Document) -> &[Node] {
        let element = document.nodes.get(self.0).unwrap().as_element().unwrap();
        &element.children
    }

    pub fn children(self, document: &Document) -> Vec<Element> {
        let element = document.nodes.get(self.0).unwrap().as_element().unwrap();
        element
            .children
            .iter()
            .filter_map(|x| x.as_element())
            .collect()
    }

    pub fn name<'d>(&self, document: &'d Document) -> &'d str {
        let element = document.nodes.get(self.0).unwrap().as_element().unwrap();
        &element.name.prefixed_name()
    }

    pub fn prefix<'d>(&self, document: &'d Document) -> Option<&'d str> {
        let element = document.nodes.get(self.0).unwrap().as_element().unwrap();
        element.name.namespace()
    }

    pub fn attributes<'d>(&self, document: &'d Document) -> &'d IndexMap<QName, String> {
        match document.attrs.get(self.0) {
            Some(x) => x,
            None => &EMPTY_INDEXMAP,
        }
    }

    pub fn attribute<'d>(&self, document: &'d Document, name: &str) -> Option<&'d str> {
        let attrs = self.attributes(document);

        attrs.get(&name.parse::<QName>().unwrap()).map(|x| &**x)
    }

    pub fn set_attribute(&self, document: &mut Document, name: &str, value: &str) {
        if !document.attrs.contains_key(self.0) {
            document.attrs.insert(self.0, Default::default());
        }

        let attrs = document.attrs.get_mut(self.0).unwrap();
        attrs.insert(name.parse().unwrap(), value.into());
    }

    pub fn display(&self, document: &Document) -> String {
        let element = document.nodes.get(self.0).unwrap().as_element().unwrap();
        let mut s = Vec::<u8>::new();
        element
            .print(
                &mut s,
                &display::Config::default_pretty(),
                &display::State::new(document),
            )
            .expect("Invalid string somehow");
        String::from_utf8(s).expect("Invalid UTF-8")
    }

    pub fn walk<'d>(&self, doc: &'d Document) -> Box<dyn Iterator<Item = Element> + 'd> {
        walk_tree(doc, *self)
    }

    pub fn next_sibling_element(&self, doc: &Document) -> Option<Element> {
        let parent = match self.parent(doc) {
            Some(v) => v,
            None => return None,
        };

        let children = parent.children(doc);
        let mut index = children
            .iter()
            .position(|x| x == self)
            .expect("element has to be child of parent");
        index += 1;

        if index < children.len() {
            return Some(children[index]);
        }

        None
    }

    pub fn prev_sibling_element(&self, doc: &Document) -> Option<Element> {
        let parent = match self.parent(doc) {
            Some(v) => v,
            None => return None,
        };

        let children = parent.children(doc);
        let mut index = children
            .iter()
            .position(|x| x == self)
            .expect("element has to be child of parent");

        if index == 0 {
            return None;
        }
        index -= 1;

        if index == 0 {
            return None;
        }

        Some(children[index])
    }

    pub fn query_selector(&self, doc: &Document, selector: &Selector) -> Option<Element> {
        self.walk(doc).find(|x| selector.matches(doc, *x))
    }

    pub fn query_selector_all(&self, doc: &Document, selector: &Selector) -> Vec<Element> {
        self.walk(doc)
            .filter(|x| selector.matches(doc, *x))
            .collect()
    }
}

fn walk_tree<'a>(doc: &'a Document, element: Element) -> Box<dyn Iterator<Item = Element> + 'a> {
    let children = element.children(doc).to_vec();
    let mut index = 0usize;

    let mut last_child: Option<Box<dyn Iterator<Item = Element>>> = None;

    Box::new(std::iter::from_fn(move || loop {
        if let Some(iter) = last_child.as_mut() {
            if let Some(next) = iter.next() {
                return Some(next);
            } else {
                last_child = None;
            }
        }

        if index >= children.len() {
            return None;
        }

        let child = children[index];
        index += 1;

        last_child = Some(Box::new(walk_tree(doc, child)));
        return Some(child);
    }))
}

static EMPTY_INDEXMAP: Lazy<IndexMap<QName, String>> = Lazy::new(IndexMap::new);
