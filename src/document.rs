use indexmap::IndexMap;
use slotmap::{SlotMap, SparseSecondaryMap};

use crate::{
    element::{self, Element},
    key::{CDataSection, Comment, DocKey, DocumentType, Text},
    value::{ElementValue, ItemValue, NodeValue},
    Node,
};

#[derive(Debug, Clone)]
pub struct Document {
    pub(crate) items: SlotMap<DocKey, ItemValue>,
    pub(crate) parents: SparseSecondaryMap<DocKey, Element>,
    pub(crate) attrs: SparseSecondaryMap<DocKey, IndexMap<String, String>>,
    pub(crate) root_key: Element,
    pub(crate) before: Vec<Node>,
    pub(crate) after: Vec<Node>,
    pub(crate) decl: Option<Declaration>,
}

#[derive(Debug, Clone)]
pub struct Declaration {
    pub version: Option<String>,
    pub encoding: Option<String>,
    pub standalone: Option<String>,
}

impl Document {
    pub fn new(root_name: &str) -> Self {
        let mut items = SlotMap::with_key();
        let parents = SparseSecondaryMap::new();
        let attrs = SparseSecondaryMap::new();

        let root_key = Element(
            items.insert(ItemValue::Node(NodeValue::Element(ElementValue {
                name: root_name.into(),
                children: vec![],
            }))),
        );

        Self {
            items,
            parents,
            attrs,
            root_key,
            before: vec![],
            after: vec![],
            decl: None,
        }
    }

    pub fn root(&self) -> Element {
        self.root_key
    }

    pub fn to_string(&self) -> String {
        format!("{}", self)
    }

    pub fn to_string_pretty(&self) -> String {
        format!("{:#}", self)
    }

    pub fn from_str(s: &str) -> Result<Document, quick_xml::Error> {
        use quick_xml::events::Event;
        use quick_xml::Reader;

        let mut r = Reader::from_str(s);
        let mut buf = Vec::new();

        let mut decl: Option<Declaration> = None;

        let mut items = SlotMap::with_key();
        let parents = SparseSecondaryMap::new();
        let attrs = SparseSecondaryMap::new();

        let mut before: Vec<Node> = vec![];
        let mut element_stack = vec![];

        let mut doc = loop {
            match r.read_event(&mut buf) {
                Ok(Event::DocType(d)) => {
                    before.push(Node::DocumentType(DocumentType(items.insert(
                        ItemValue::Node(NodeValue::DocumentType(
                            d.unescape_and_decode(&r).unwrap(),
                        )),
                    ))));
                }
                Ok(Event::Decl(d)) => {
                    let version = d
                        .version()
                        .map(|x| std::str::from_utf8(&x).unwrap().to_string())
                        .ok();
                    let standalone = d.standalone().and_then(|x| match x {
                        Ok(x) => Some(std::str::from_utf8(&x).unwrap().to_string()),
                        Err(_) => None,
                    });
                    let encoding = d.encoding().and_then(|x| match x {
                        Ok(x) => Some(std::str::from_utf8(&x).unwrap().to_string()),
                        Err(_) => None,
                    });

                    decl = Some(Declaration {
                        version,
                        standalone,
                        encoding,
                    });
                }
                Ok(ref x @ (Event::Start(ref e) | Event::Empty(ref e))) => {
                    let name = std::str::from_utf8(e.name()).unwrap().to_string();

                    let root_key = Element(items.insert(ItemValue::Node(NodeValue::Element(
                        ElementValue {
                            name: name.into(),
                            children: vec![],
                        },
                    ))));

                    let mut document = Document {
                        items,
                        parents,
                        attrs,
                        root_key,
                        decl,
                        before,
                        after: vec![],
                    };

                    let root = document.root();

                    if matches!(x, Event::Start(_)) {
                        element_stack.push(root);
                    }

                    for attr in e.attributes().filter_map(Result::ok) {
                        let value = attr.unescape_and_decode_value(&r).unwrap();
                        let s = std::str::from_utf8(attr.key)?;

                        root.set_attribute(&mut document, s, &value);
                    }

                    break document;
                }
                Ok(Event::Text(e)) => {
                    if e.len() == 0 {
                        continue;
                    }
                    if e.unescape_and_decode(&r)
                        .map(|x| x.trim().len() == 0)
                        .unwrap_or(false)
                    {
                        continue;
                    }
                    before.push(Node::Text(Text(items.insert(ItemValue::Node(
                        NodeValue::Text(e.unescape_and_decode(&r).unwrap()),
                    )))));
                }
                Ok(Event::Comment(e)) => {
                    before.push(Node::Comment(Comment(items.insert(ItemValue::Node(
                        NodeValue::Comment(e.unescape_and_decode(&r).unwrap()),
                    )))));
                }
                Ok(Event::CData(e)) => {
                    before.push(Node::CDataSection(CDataSection(items.insert(
                        ItemValue::Node(NodeValue::CData(e.unescape_and_decode(&r).unwrap())),
                    ))));
                }
                Ok(x) => {
                    panic!("Uhh... {:?}", x);
                }
                Err(e) => return Err(e),
            }
        };

        loop {
            match r.read_event(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = std::str::from_utf8(e.name()).unwrap().to_string();
                    let parent = match element_stack.last() {
                        Some(v) => v,
                        None => {
                            return Err(quick_xml::Error::UnexpectedToken(name));
                        }
                    };
                    let mut attrs = IndexMap::new();
                    for attr in e.attributes().filter_map(Result::ok) {
                        let value = attr.unescape_and_decode_value(&r)?;
                        attrs.insert(std::str::from_utf8(attr.key).unwrap().to_string(), value);
                    }
                    let element =
                        parent.append_new_element(&mut doc, crate::NewElement { name, attrs });
                    element_stack.push(element);
                }
                Ok(Event::Empty(e)) => {
                    let name = std::str::from_utf8(e.name()).unwrap().to_string();
                    let parent = match element_stack.last() {
                        Some(v) => v,
                        None => {
                            return Err(quick_xml::Error::UnexpectedToken(name));
                        }
                    };
                    let mut attrs = IndexMap::new();
                    for attr in e.attributes().filter_map(Result::ok) {
                        let value = attr.unescape_and_decode_value(&r)?;
                        attrs.insert(std::str::from_utf8(attr.key).unwrap().to_string(), value);
                    }
                    parent.append_new_element(&mut doc, crate::NewElement { name, attrs });
                }
                Ok(Event::End(_e)) => {
                    element_stack.pop();
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape_and_decode(&r)?;
                    if text.trim().len() > 0 {
                        match element_stack.last() {
                            Some(el) => {
                                el.append_text(&mut doc, &text);
                            }
                            None => {
                                doc.after.push(Node::Text(Text(
                                    doc.items.insert(ItemValue::Node(NodeValue::Text(text))),
                                )));
                            }
                        }
                    }
                }
                Ok(Event::CData(cdata)) => {
                    let text = cdata.unescape_and_decode(&r)?;
                    match element_stack.last() {
                        Some(el) => {
                            el.append_cdata(&mut doc, &text);
                        }
                        None => {
                            doc.after.push(Node::CDataSection(CDataSection(
                                doc.items.insert(ItemValue::Node(NodeValue::CData(text))),
                            )));
                        }
                    }
                }
                Ok(Event::Comment(comment)) => {
                    let text = comment.unescape_and_decode(&r)?;
                    match element_stack.last() {
                        Some(el) => {
                            el.append_comment(&mut doc, &text);
                        }
                        None => {
                            doc.after.push(Node::Comment(Comment(
                                doc.items.insert(ItemValue::Node(NodeValue::Comment(text))),
                            )));
                        }
                    }
                }
                Ok(Event::PI(_processing_instruction)) => {
                    continue;
                }
                Ok(Event::Decl(_decl)) => {
                    continue;
                }
                Ok(Event::DocType(_doctype)) => {
                    continue;
                }
                Ok(Event::Eof) => {
                    break;
                } // exits the loop when reaching end of file
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(doc)
    }
}
