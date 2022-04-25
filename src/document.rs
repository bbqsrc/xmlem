use indexmap::IndexMap;
use slotmap::{SlotMap, SparseSecondaryMap};

use crate::{
    element::Element,
    key::DocKey,
    value::{ElementValue, ItemValue, NodeValue},
};

#[derive(Debug)]
pub struct Document {
    pub(crate) items: SlotMap<DocKey, ItemValue>,
    pub(crate) parents: SparseSecondaryMap<DocKey, Element>,
    pub(crate) attrs: SparseSecondaryMap<DocKey, IndexMap<String, String>>,
    pub(crate) root_key: Element,
    pub(crate) doctype: Option<String>,
    pub(crate) decl: Option<Declaration>,
}

#[derive(Debug)]
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
            doctype: None,
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

        let mut doctype: Option<String> = None;
        let mut decl: Option<Declaration> = None;

        let mut doc = loop {
            match r.read_event(&mut buf) {
                Ok(Event::DocType(d)) => {
                    doctype = Some(d.unescape_and_decode(&r).unwrap());
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
                Ok(Event::Start(e)) => {
                    let name = std::str::from_utf8(e.name()).unwrap().to_string();
                    let mut document = Document::new(&name);
                    let root = document.root();

                    for attr in e.attributes().filter_map(Result::ok) {
                        let value = attr.unescape_and_decode_value(&r).unwrap();
                        let s = std::str::from_utf8(attr.key)?;

                        root.set_attribute(&mut document, s, &value);
                    }

                    break document;
                }
                Ok(Event::Text(e)) if e.len() == 0 => {
                    continue;
                }
                Ok(Event::Text(e))
                    if e.unescape_and_decode(&r)
                        .map(|x| x.trim().len() == 0)
                        .unwrap_or(false) =>
                {
                    continue;
                }
                x => panic!("Not a root? {:?}", x),
            }
        };

        doc.doctype = doctype;
        doc.decl = decl;

        let mut element_stack = vec![doc.root()];

        loop {
            match r.read_event(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = std::str::from_utf8(e.name()).unwrap().to_string();
                    let parent = element_stack.last().unwrap();
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
                    let parent = element_stack.last().unwrap();
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
                        let el = element_stack.last().unwrap();
                        el.append_text(&mut doc, &text);
                    }
                }
                Ok(Event::CData(cdata)) => {
                    let text = cdata.unescape_and_decode(&r)?;
                    let el = element_stack.last().unwrap();
                    el.append_cdata(&mut doc, &text);
                }
                Ok(Event::Comment(_comment)) => {
                    continue;
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
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(doc)
    }
}
