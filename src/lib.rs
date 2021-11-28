use std::{
    borrow::Cow,
    cell::RefCell,
    collections::BTreeMap,
    fmt::Display,
    rc::{Rc, Weak},
};

use url::Url;

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
                Ok(Event::End(_)) => {
                    element_stack.pop();
                }
                Ok(Event::Text(e)) => {
                    let el = element_stack.last().unwrap().borrow();
                    el.add_text(Rc::new(RefCell::new(e.unescape_and_decode(&r)?)));
                }
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                Err(e) => panic!("Error at position {}: {:?}", r.buffer_position(), e),
                _ => (), // There are several other `Event`s we do not consider here
            }
        }

        Ok(root)
    }
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

    fn add_child(&self, child: Rc<RefCell<Element>>) {
        let mut children = RefCell::borrow_mut(&self.children);
        children.push(child.into());
    }

    pub fn add_text(&self, text: Rc<RefCell<String>>) {
        let mut children = RefCell::borrow_mut(&self.children);
        children.push(Node::Text(text));
    }

    pub fn add_attr(&self, key: QName, value: impl Into<String>) {
        let mut attrs = RefCell::borrow_mut(&self.attributes);
        attrs.insert(key, value.into());
    }
}

impl Display for QName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.namespace.as_ref().map(|x| &**x).map(RefCell::borrow) {
            Some(ns) => f.write_fmt(format_args!("{}:{}", ns.short_name, self.name)),
            None => f.write_str(&self.name),
        }
    }
}

fn process_entities(input: &str) -> Cow<'_, str> {
    if input.contains(|c| ['<', '>', '\'', '"', '&'].contains(&c)) {
        let mut s = String::with_capacity(input.len());
        input.chars().for_each(|ch| {
            s.push_str(match ch {
                '\'' => "&apos;",
                '"' => "&quot;",
                '&' => "&amp;",
                '<' => "&lt;",
                '>' => "&gt;",
                ch if ch.is_ascii_control() => {
                    s.push_str(&format!("&#x{:X};", ch as u32));
                    return;
                }
                other => {
                    s.push(other);
                    return;
                }
            })
        });
        Cow::Owned(s)
    } else {
        Cow::Borrowed(input)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Namespace {
    url: Url,
    short_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct QName {
    namespace: Option<Rc<RefCell<Namespace>>>,
    name: String,
}

fn is_valid_qname(input: &str) -> bool {
    fn is_name_start_char(ch: char) -> bool {
        match ch {
            ':' | 'A'..='Z' | '_' | 'a'..='z' => return true,
            _ => {}
        }
        match ch as u32 {
            0xC0..=0xD6
            | 0xD8..=0xF6
            | 0xF8..=0x2FF
            | 0x370..=0x37D
            | 0x37F..=0x1FFF
            | 0x200C..=0x200D
            | 0x2070..=0x218F
            | 0x2C00..=0x2FEF
            | 0x3001..=0xD7FF
            | 0xF900..=0xFDCF
            | 0xFDF0..=0xFFFD
            | 0x10000..=0xEFFFF => true,
            _ => false,
        }
    }

    fn is_name_char(ch: char) -> bool {
        if is_name_start_char(ch) {
            return true;
        }

        match ch {
            '-' | '.' | '0'..='9' => return true,
            _ => {}
        }

        match ch as u32 {
            0xb7 | 0x0300..=0x036F | 0x203F..=0x2040 => true,
            _ => false,
        }
    }

    let mut chars = input.chars();
    let is_valid = match chars.next() {
        Some(ch) => is_name_start_char(ch),
        None => false,
    };
    if !is_valid {
        return false;
    }

    chars.all(is_name_char)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid QName: {0}")]
    InvalidQName(String),
}

impl QName {
    pub fn new(element: &Element, name: impl Into<String>) -> Result<Self, Error> {
        let name = name.into();

        if !is_valid_qname(&name) {
            return Err(Error::InvalidQName(name));
        }

        match name.split_once(":") {
            Some((a, b)) => match element.namespaces().get(a) {
                Some(ns) => {
                    return Ok(Self {
                        namespace: Some(ns.clone()),
                        name: b.to_string(),
                    })
                }
                None => {
                    return Err(Error::InvalidQName(name));
                }
            },
            None => {}
        }

        Ok(Self {
            namespace: None,
            name,
        })
    }

    pub fn with_namespace(ns: Rc<RefCell<Namespace>>, name: impl Into<String>) -> Self {
        let name = name.into();
        if !is_valid_qname(&name) {
            panic!("Invalid qname");
        }

        Self {
            namespace: Some(ns),
            name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Element {
    name: QName,
    local_main_namespace: RefCell<Option<Url>>,
    local_namespaces: RefCell<BTreeMap<String, Rc<RefCell<Namespace>>>>,
    attributes: Rc<RefCell<BTreeMap<QName, String>>>,
    children: Rc<RefCell<Vec<Node>>>,
    parent: Option<Weak<RefCell<Element>>>,
}

#[derive(Debug, Clone)]
pub enum Node {
    Element(Rc<RefCell<Element>>),
    Text(Rc<RefCell<String>>),
}

impl From<Rc<RefCell<Element>>> for Node {
    fn from(x: Rc<RefCell<Element>>) -> Self {
        Self::Element(x)
    }
}

impl From<Rc<RefCell<String>>> for Node {
    fn from(x: Rc<RefCell<String>>) -> Self {
        Self::Text(x)
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Element(x) => Display::fmt(&RefCell::borrow(x), f),
            Node::Text(x) => Display::fmt(&RefCell::borrow(x), f),
        }
    }
}

#[test]
fn smoke() {
    let root = Element::root("root").unwrap();
    let root_el = root.borrow();
    root_el.set_local_main_namespace(Some(Url::parse("http://wat.lol").unwrap()));
    let mlem_ns = root_el.add_local_namespace(Url::parse("http://test.url/lol/").unwrap(), "mlem");

    let test = Element::new_child(&root, "test").unwrap();
    {
        Element::new_child(&test, "mlem").unwrap();

        let e = Element::new_child_ns(&test, QName::with_namespace(mlem_ns.clone(), "AHHHHHH"))
            .unwrap();
        {
            let e_ref = e.borrow();
            let ns = mlem_ns.clone();
            e_ref.add_attr(QName::with_namespace(ns.clone(), "test"), "amusement");

            e_ref.add_attr(
                QName::with_namespace(ns, "smart"),
                "<injection attack \0\0\0\0\0 \"'&'\"/>",
            )
        }
        Element::new_child(&test, "mlem2").unwrap();
    }

    println!("{}", root.borrow());
}

#[test]
fn smoke2() {
    let mut root = Element::from_str(
        r#"<root xmlns:x="http://lol" someattr="true">lol <x:sparta/><sparta derp="9000"></sparta> </root>"#,
    ).unwrap();
    root.borrow().set_local_ns_short_name("x", "huehuehue");
    println!("{}", root.borrow());
}

#[test]
fn smoke3() {
    let input = r#"<俄语 լեզու="ռուսերեն">данные</俄语>"#;
    let root = Element::from_str(input).unwrap();

    println!("{}", root.borrow());
}
