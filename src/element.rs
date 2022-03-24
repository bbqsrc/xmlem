use std::{
    cell::RefCell,
    collections::BTreeMap,
    fmt::Display,
    rc::{Rc, Weak},
};

use url::Url;

use crate::{
    node::Node, process_entities, qname::is_valid_qname, qname::Namespace, qname::QName, Error,
};

#[derive(Debug)]
pub struct Element {
    name: QName,
    local_main_namespace: RefCell<Option<Url>>,
    local_namespaces: RefCell<BTreeMap<String, Rc<RefCell<Namespace>>>>,
    attributes: Rc<RefCell<BTreeMap<QName, String>>>,
    children: Rc<RefCell<Vec<Node>>>,
    parent: Option<Weak<RefCell<Element>>>,
}

impl Element {
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

    pub fn set_local_main_namespace(&self, url: Option<Url>) {
        *self.local_main_namespace.borrow_mut() = url;
    }

    pub fn add_local_namespace(
        &self,
        url: Url,
        short_name: impl Into<String>,
    ) -> Rc<RefCell<Namespace>> {
        let short_name = short_name.into();
        let ns = Rc::new(RefCell::new(Namespace {
            url,
            short_name: short_name.clone(),
        }));

        self.local_namespaces
            .borrow_mut()
            .insert(short_name, ns.clone());

        ns
    }

    pub fn set_local_ns_short_name(&self, old_name: &str, new_name: &str) {
        let mut ns = self.local_namespaces.borrow_mut();
        let namespace = ns.remove(old_name).unwrap();
        namespace.borrow_mut().short_name = new_name.to_string();
        ns.insert(new_name.to_string(), namespace);
    }

    /// Get all parent elements
    pub fn ancestors(&self) -> impl Iterator<Item = Rc<RefCell<Element>>> {
        let mut parent = self.parent.clone();
        std::iter::from_fn(move || {
            let cur = parent.clone();
            if let Some(cur) = cur.and_then(|x| x.upgrade()) {
                parent = cur.borrow().parent.clone();
                Some(cur)
            } else {
                None
            }
        })
    }

    pub fn namespaces(&self) -> BTreeMap<String, Rc<RefCell<Namespace>>> {
        let mut ns = BTreeMap::new();

        let mut parents = self.ancestors().collect::<Vec<_>>();
        parents.reverse();

        for parent in parents {
            let element = parent.borrow();

            for (k, v) in element.local_namespaces.borrow().iter() {
                ns.insert(k.to_string(), v.clone());
            }
        }

        for (k, v) in self.local_namespaces.borrow().iter() {
            ns.insert(k.to_string(), v.clone());
        }

        ns
    }

    pub fn new_child(
        this: &Rc<RefCell<Element>>,
        name: impl Into<String>,
    ) -> Result<Rc<RefCell<Element>>, Error> {
        let parent = Some(this.clone());
        let el = this.borrow();
        let element = Element::new(QName::new(&el, name)?, parent);
        el.add_child(element.clone());
        Ok(element)
    }

    pub fn new_child_ns(
        this: &Rc<RefCell<Element>>,
        qname: QName,
    ) -> Result<Rc<RefCell<Element>>, Error> {
        let parent = Some(this.clone());
        let el = this.borrow();
        let element = Element::new(qname, parent);
        el.add_child(element.clone());
        Ok(element)
    }

    pub fn children(&self) -> Rc<RefCell<Vec<Node>>> {
        self.children.clone()
    }

    fn add_child(&self, child: Rc<RefCell<Element>>) {
        let mut children = RefCell::borrow_mut(&self.children);
        children.push(child.into());
    }

    pub fn set_parent(&mut self, parent: Rc<RefCell<Element>>) {
        let weak_parent = Rc::downgrade(&parent);

        self.parent = Some(weak_parent);
    }

    pub fn add_text(&self, text: Rc<RefCell<String>>) {
        let mut children = RefCell::borrow_mut(&self.children);
        children.push(Node::Text(text));
    }

    pub fn add_cdata(&self, text: Rc<RefCell<String>>) {
        let mut children = RefCell::borrow_mut(&self.children);
        children.push(Node::CData(text));
    }

    pub fn add_attr(&self, key: QName, value: impl Into<String>) {
        let mut attrs = RefCell::borrow_mut(&self.attributes);
        attrs.insert(key, value.into());
    }
}

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
                    Element::new_child(&parent, name).unwrap();
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
}

impl Clone for Element {
    fn clone(&self) -> Self {
        let name = self.name.clone();

        let url_borrow = self.local_main_namespace.borrow();

        let namespaces_borrow = self.local_namespaces.borrow();
        let mut new_namespace_map = BTreeMap::new();
        for (key, value) in namespaces_borrow.iter() {
            let inner_namespace = &*value.borrow();

            let x = inner_namespace.clone();
            new_namespace_map.insert(key.clone(), Rc::new(RefCell::new(x)));
        }

        let attributes_borrow = &*self.attributes.borrow();
        let new_attributes_map = attributes_borrow.clone();

        let child_vec_borrow = &*self.children.borrow();
        let new_child_vec = child_vec_borrow.clone();

        Self {
            name,
            local_main_namespace: RefCell::new(url_borrow.clone()),
            local_namespaces: RefCell::new(new_namespace_map),
            attributes: Rc::new(RefCell::new(new_attributes_map)),
            children: Rc::new(RefCell::new(new_child_vec)),
            parent: None,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone();
    }
}

impl Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let element = self;

        f.write_fmt(format_args!("<{}", element.name))?;

        if let Some(xmlns) = self.local_main_namespace.borrow().as_ref() {
            f.write_fmt(format_args!(" xmlns=\"{}\"", xmlns))?;
        }

        for ns in self.local_namespaces.borrow().values() {
            let ns = ns.borrow();
            f.write_fmt(format_args!(" xmlns:{}=\"{}\"", ns.short_name, ns.url))?;
        }

        let attributes = RefCell::borrow(&element.attributes);
        for (key, value) in attributes.iter() {
            f.write_fmt(format_args!(" {}=\"{}\"", key, process_entities(value)))?;
        }

        let children = RefCell::borrow(&*element.children);

        if children.is_empty() {
            return f.write_str("/>");
        }

        f.write_fmt(format_args!(">"))?;

        for child in children.iter() {
            Display::fmt(&*child, f)?;
        }

        f.write_fmt(format_args!("</{}>", &element.name))
    }
}
