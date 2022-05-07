use std::io::BufRead;

use indexmap::IndexMap;
use slotmap::{SlotMap, SparseSecondaryMap};

use crate::{
    display::{self, Config, Print, State},
    element::Element,
    key::{CDataSection, Comment, DocKey, DocumentType, Text},
    qname::QName,
    value::{ElementValue, NodeValue},
    Node,
};

#[derive(Debug, Clone)]
pub struct Document {
    pub(crate) nodes: SlotMap<DocKey, NodeValue>,
    pub(crate) parents: SparseSecondaryMap<DocKey, Element>,
    pub(crate) attrs: SparseSecondaryMap<DocKey, IndexMap<QName, String>>,
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

impl Declaration {
    pub fn v1_0() -> Self {
        Self {
            version: Some("1.0".to_string()),
            encoding: Some("utf-8".to_string()),
            standalone: None,
        }
    }

    pub fn v1_1() -> Self {
        Self {
            version: Some("1.1".to_string()),
            encoding: Some("utf-8".to_string()),
            standalone: None,
        }
    }
}

impl Document {
    pub fn new(root_name: &str) -> Self {
        let mut nodes = SlotMap::with_key();
        let parents = SparseSecondaryMap::new();
        let attrs = SparseSecondaryMap::new();

        let root_key = Element(nodes.insert(NodeValue::Element(ElementValue {
            name: root_name.parse().unwrap(),
            children: vec![],
        })));

        Self {
            nodes,
            parents,
            attrs,
            root_key,
            before: vec![],
            after: vec![],
            decl: None,
        }
    }

    pub fn set_declaration(&mut self, decl: Option<Declaration>) {
        self.decl = decl;
    }

    pub fn declaration(&self) -> Option<&Declaration> {
        self.decl.as_ref()
    }

    pub fn set_doctype(&mut self, doctype: Option<&str>) {
        match doctype {
            Some(v) => {
                let id = Node::DocumentType(DocumentType(
                    self.nodes.insert(NodeValue::DocumentType(v.to_string())),
                ));

                for i in 0..self.before.len() {
                    if self.before[i].as_document_type().is_some() {
                        self.nodes.remove(self.before[i].as_key());
                        self.before[i] = id;
                        return;
                    }
                }

                // If that failed, push it ourselves.
                self.before.insert(0, id);
            }
            None => {
                for i in 0..self.before.len() {
                    if self.before[i].as_document_type().is_some() {
                        self.before.remove(i);
                        return;
                    }
                }
            }
        }
    }

    pub fn doctype(&self) -> Option<&str> {
        for i in 0..self.before.len() {
            if let Some(v) = self.before[i].as_document_type() {
                return Some(self.nodes.get(v.0).unwrap().as_document_type().unwrap());
            }
        }

        None
    }

    #[inline]
    pub fn root(&self) -> Element {
        self.root_key
    }

    #[inline]
    pub fn to_string_pretty(&self) -> String {
        let mut s = vec![];
        self.print(&mut s, &Config::default_pretty(), &State::new(self))
            .unwrap();
        String::from_utf8(s).expect("invalid UTF-8")
    }

    #[inline]
    pub fn to_string_pretty_with_config(&self, config: &display::Config) -> String {
        let mut s = vec![];
        self.print(&mut s, &config, &State::new(self)).unwrap();
        String::from_utf8(s).expect("invalid UTF-8")
    }

    #[inline]
    pub fn from_file(file: std::fs::File) -> Result<Document, quick_xml::Error> {
        let reader = std::io::BufReader::new(file);
        Self::from_reader(reader)
    }

    #[inline]
    pub fn from_reader<R: BufRead>(reader: R) -> Result<Document, quick_xml::Error> {
        use quick_xml::events::Event;
        use quick_xml::Reader;

        let mut r = Reader::from_reader(reader);
        let mut buf = Vec::new();

        let mut decl: Option<Declaration> = None;

        let mut nodes = SlotMap::with_key();
        let parents = SparseSecondaryMap::new();
        let attrs = SparseSecondaryMap::new();

        let mut before: Vec<Node> = vec![];
        let mut element_stack = vec![];

        let mut doc = loop {
            match r.read_event(&mut buf) {
                Ok(Event::DocType(d)) => {
                    before.push(Node::DocumentType(DocumentType(nodes.insert(
                        NodeValue::DocumentType(
                            d.unescape_and_decode(&r).unwrap().trim().to_string(),
                        ),
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
                    let name: QName = std::str::from_utf8(e.name()).unwrap().parse().unwrap();

                    let root_key = Element(nodes.insert(NodeValue::Element(ElementValue {
                        name,
                        children: vec![],
                    })));

                    let mut document = Document {
                        nodes,
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
                        .map(|x| x.trim().is_empty())
                        .unwrap_or(false)
                    {
                        continue;
                    }
                    before.push(Node::Text(Text(
                        nodes.insert(NodeValue::Text(e.unescape_and_decode(&r).unwrap())),
                    )));
                }
                Ok(Event::Comment(e)) => {
                    before.push(Node::Comment(Comment(
                        nodes.insert(NodeValue::Comment(e.unescape_and_decode(&r).unwrap())),
                    )));
                }
                Ok(Event::CData(e)) => {
                    before.push(Node::CDataSection(CDataSection(
                        nodes.insert(NodeValue::CData(e.unescape_and_decode(&r).unwrap())),
                    )));
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
                    let name: QName = std::str::from_utf8(e.name()).unwrap().parse().unwrap();
                    let parent = match element_stack.last() {
                        Some(v) => v,
                        None => {
                            return Err(quick_xml::Error::UnexpectedToken(name.prefixed_name));
                        }
                    };
                    let mut attrs = IndexMap::new();
                    for attr in e.attributes().filter_map(Result::ok) {
                        let value = attr.unescape_and_decode_value(&r)?;
                        attrs.insert(
                            std::str::from_utf8(attr.key).unwrap().parse().unwrap(),
                            value,
                        );
                    }
                    let element =
                        parent.append_new_element(&mut doc, crate::NewElement { name, attrs });
                    element_stack.push(element);
                }
                Ok(Event::Empty(e)) => {
                    let name: QName = std::str::from_utf8(e.name()).unwrap().parse().unwrap();
                    let parent = match element_stack.last() {
                        Some(v) => v,
                        None => {
                            return Err(quick_xml::Error::UnexpectedToken(name.prefixed_name));
                        }
                    };
                    let mut attrs = IndexMap::new();
                    for attr in e.attributes().filter_map(Result::ok) {
                        let value = attr.unescape_and_decode_value(&r)?;
                        attrs.insert(
                            std::str::from_utf8(attr.key).unwrap().parse().unwrap(),
                            value,
                        );
                    }
                    parent.append_new_element(&mut doc, crate::NewElement { name, attrs });
                }
                Ok(Event::End(_e)) => {
                    element_stack.pop();
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape_and_decode(&r)?;
                    if !text.trim().is_empty() {
                        match element_stack.last() {
                            Some(el) => {
                                el.append_text(&mut doc, &text);
                            }
                            None => {
                                doc.after.push(Node::Text(Text(
                                    doc.nodes.insert(NodeValue::Text(text)),
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
                                doc.nodes.insert(NodeValue::CData(text)),
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
                                doc.nodes.insert(NodeValue::Comment(text)),
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
                    // exits the loop when reaching end of file
                    break;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(doc)
    }
}

impl std::str::FromStr for Document {
    type Err = quick_xml::Error;

    fn from_str(s: &str) -> Result<Document, quick_xml::Error> {
        Self::from_reader(std::io::Cursor::new(s))
    }
}
