use quick_xml::events::Event;
use quick_xml::Reader;

pub struct Document {}

impl Document {
    pub fn new(name: impl Into<String>) -> Result<Self, quick_xml::Error> {
        Ok(Document {})
    }

    /*
    pub fn root(name: impl Into<String>) -> Result<Rc<RefCell<Element>>, Error> {
        let name = name.into();

        if !is_valid_qname(&name) {
            return Err(Error::InvalidQName(name));
        }

        Ok(Self::new(
            QName {
                namespace: None,
                name,
            },
            None,
        ))
    }

    fn new(name: QName, parent: Option<Rc<RefCell<Element>>>) -> Rc<RefCell<Element>> {
        let name = name.into();

        Rc::new(RefCell::new(Element {
            attributes: Default::default(),
            name,
            children: Default::default(),
            parent: parent.map(|x| Rc::downgrade(&x)),
            local_main_namespace: Default::default(),
            local_namespaces: Default::default(),
        }))
    }
    */

    // Incomplete
    pub fn from_str(string: &str) -> Result<Self, quick_xml::Error> {
        let mut reader = Reader::from_str(string);
        let mut buffer = Vec::new();

        let document = loop {
            match reader.read_event(&mut buffer) {
                // Skip past valid but non-root items
                Ok(event) => match event {
                    Event::Text(text) => {
                        if text.len() == 0 {
                            continue;
                        }

                        if text
                            .unescape_and_decode(&reader)
                            .map(|string| string.trim().len() == 0)
                            .unwrap_or(false)
                        {
                            continue;
                        }

                        // What if it's text, but not 0 len?
                    }
                    Event::Decl(_) | Event::DocType(_) => {
                        continue;
                    }
                    // Root element
                    Event::Start(bytes) => {
                        let name = std::str::from_utf8(bytes.name()).unwrap().to_string();

                        let document = Document::new(&name).unwrap();
                        {}

                        break document;
                    }
                    other => panic!("Invalid item at root level {:?}", other),
                },
                Err(error) => panic!("Not reading from root! Error: {}", error),
            }
        };

        //let mut element_stack = vec![root.clone()];

        Ok(document)
    }
}

/*
impl Element {
    pub fn from_str(s: &str) -> Result<Rc<RefCell<Element>>, quick_xml::Error> {
        use quick_xml::events::Event;
        use quick_xml::Reader;

        let mut r = Reader::from_str(s);
        let mut buf = Vec::new();

        let root = loop {
            match r.read_event(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = std::str::from_utf8(e.name()).unwrap().to_string();
                    let root = Element::root(name).unwrap();
                    {
                        let root = root.borrow();

                        for attr in e.attributes().filter_map(Result::ok) {
                            let value = attr.unescape_and_decode_value(&r).unwrap();
                            let s = std::str::from_utf8(attr.key)?;

                            if s == "xmlns" {
                                let url = Url::parse(&value).unwrap();
                                root.set_local_main_namespace(Some(url));
                                continue;
                            }

                            if s.starts_with("xmlns:") {
                                let url = Url::parse(&value).unwrap();
                                root.add_local_namespace(url, s.split_once(":").unwrap().1);
                                continue;
                            }

                            let key: QName = QName::new(&root, s).unwrap();
                            root.add_attr(key, value);
                        }
                    }
                    break root;
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
                Ok(Event::Decl(_) | Event::DocType(_)) => {
                    continue;
                }
                x => panic!("Not a root? {:?}", x),
            }
        };

        let mut element_stack = vec![root.clone()];

        loop {
            match r.read_event(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = std::str::from_utf8(e.name()).unwrap();
                    let parent = element_stack.last().unwrap();
                    let element = Element::new_child(parent, name).unwrap();
                    {
                        let el = element.borrow();
                        for attr in e.attributes().filter_map(Result::ok) {
                            let root = root.borrow();
                            let key: QName =
                                QName::new(&root, std::str::from_utf8(attr.key).unwrap()).unwrap();
                            let value = attr.unescape_and_decode_value(&r)?;
                            el.add_attr(key, value);
                        }
                    }
                    element_stack.push(element);
                }
                Ok(Event::Empty(e)) => {
                    let name = std::str::from_utf8(e.name()).unwrap();
                    let parent = element_stack.last().unwrap();
                    let element = Element::new_child(parent, name).unwrap();
                    {
                        let el = element.borrow();
                        for attr in e.attributes().filter_map(Result::ok) {
                            let root = root.borrow();
                            let key: QName =
                                QName::new(&root, std::str::from_utf8(attr.key).unwrap()).unwrap();
                            let value = attr.unescape_and_decode_value(&r)?;
                            el.add_attr(key, value);
                        }
                    }
                }
                Ok(Event::End(_e)) => {
                    element_stack.pop();
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape_and_decode(&r)?;
                    if text.trim().len() > 0 {
                        let el = element_stack.last().unwrap().borrow();
                        el.add_text(Rc::new(RefCell::new(text)));
                    }
                }
                Ok(Event::CData(cdata)) => {
                    let text = cdata.unescape_and_decode(&r)?;
                    let el = element_stack.last().unwrap().borrow();
                    el.add_cdata(Rc::new(RefCell::new(text)));
                }
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                Err(e) => panic!("Error at position {}: {:?}", r.buffer_position(), e),
                _ => (), // There are several other `Event`s we do not consider here
            }
        }

        Ok(root)
    }
} */
