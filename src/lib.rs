use std::{borrow::Cow, cell::RefCell, collections::BTreeMap, fmt::Display, rc::Rc};

use url::Url;

#[derive(Debug, Clone)]
pub struct Root {
    main_namespace: Option<Url>,
    namespaces: BTreeMap<String, Rc<RefCell<Namespace>>>,
    element: Rc<RefCell<Element>>,
}

impl Display for Root {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let element = self.element.borrow();

        f.write_fmt(format_args!("<{}", element.name))?;

        if let Some(xmlns) = self.main_namespace.as_ref() {
            f.write_fmt(format_args!(" xmlns=\"{}\"", xmlns))?;
        }

        for ns in self.namespaces.values() {
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

impl Root {
    pub fn new(name: impl Into<String>) -> Root {
        let name = name.into();

        let element = Element {
            attributes: Default::default(),
            name: QName {
                namespace: None,
                name,
            },
            children: Default::default(),
        };

        Self {
            main_namespace: Default::default(),
            namespaces: Default::default(),
            element: Rc::new(RefCell::new(element)),
        }
    }

    pub fn set_main_namespace(&mut self, url: Url) {
        self.main_namespace = Some(url);
    }

    pub fn add_namespace(
        &mut self,
        url: Url,
        short_name: impl Into<String>,
    ) -> Rc<RefCell<Namespace>> {
        let short_name = short_name.into();
        let ns = Rc::new(RefCell::new(Namespace {
            url,
            short_name: short_name.clone(),
        }));

        self.namespaces.insert(short_name, ns.clone());

        ns
    }

    pub fn set_ns_short_name(&mut self, old_name: &str, new_name: &str) {
        let namespace = self.namespaces.remove(old_name).unwrap();
        namespace.borrow_mut().short_name = new_name.to_string();
        self.namespaces.insert(new_name.to_string(), namespace);
    }

    pub fn element(&self, name: impl Into<String>) -> Rc<RefCell<Element>> {
        Element::new(QName::new(&self, name))
    }

    pub fn from_str(s: &str) -> Root {
        use quick_xml::events::Event;
        use quick_xml::Reader;

        let mut r = Reader::from_str(s);
        let mut buf = Vec::new();

        let root = loop {
            match r.read_event(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = std::str::from_utf8(e.name()).unwrap();
                    let mut root = Root::new(name);
                    {
                        for attr in e.attributes().filter_map(Result::ok) {
                            let value = attr.unescape_and_decode_value(&r).unwrap();
                            let s = std::str::from_utf8(attr.key).unwrap();

                            if s == "xmlns" {
                                let url = Url::parse(&value).unwrap();
                                root.set_main_namespace(url);
                                continue;
                            }

                            if s.starts_with("xmlns:") {
                                let url = Url::parse(&value).unwrap();
                                root.add_namespace(url, s.split_once(":").unwrap().1);
                                continue;
                            }

                            let key: QName = QName::new(&root, s);
                            let el = root.element.borrow();
                            el.add_attr(key, value);
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

        let mut element_stack = vec![root.element.clone()];

        loop {
            match r.read_event(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = std::str::from_utf8(e.name()).unwrap();
                    let element = root.element(name);
                    {
                        let el = element.borrow();
                        for attr in e.attributes().filter_map(Result::ok) {
                            let key: QName =
                                QName::new(&root, std::str::from_utf8(attr.key).unwrap());
                            let value = attr.unescape_and_decode_value(&r).unwrap();
                            el.add_attr(key, value);
                        }
                    }
                    element_stack
                        .last()
                        .unwrap()
                        .borrow()
                        .add_child(element.clone());
                    element_stack.push(element);
                }
                Ok(Event::Empty(e)) => {
                    let name = std::str::from_utf8(e.name()).unwrap();
                    let element = root.element(name);
                    element_stack
                        .last()
                        .unwrap()
                        .borrow()
                        .add_child(element.clone());
                }
                Ok(Event::End(_)) => {
                    element_stack.pop();
                }
                Ok(Event::Text(e)) => {
                    let el = element_stack.last().unwrap().borrow();
                    el.add_text(Rc::new(RefCell::new(e.unescape_and_decode(&r).unwrap())));
                }
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                Err(e) => panic!("Error at position {}: {:?}", r.buffer_position(), e),
                _ => (), // There are several other `Event`s we do not consider here
            }
        }

        root
    }
}

impl Element {
    pub fn new(name: QName) -> Rc<RefCell<Element>> {
        let name = name.into();

        Rc::new(RefCell::new(Element {
            attributes: Default::default(),
            name,
            children: Default::default(),
        }))
    }

    pub fn add_child(&self, child: Rc<RefCell<Element>>) {
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

impl Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("<{}", self.name))?;

        let attributes = RefCell::borrow(&self.attributes);
        for (key, value) in attributes.iter() {
            f.write_fmt(format_args!(" {}=\"{}\"", key, process_entities(value)))?;
        }

        let children = RefCell::borrow(&*self.children);

        if children.is_empty() {
            return f.write_str("/>");
        }

        f.write_fmt(format_args!(">"))?;

        for child in children.iter() {
            Display::fmt(&*child, f)?;
        }

        f.write_fmt(format_args!("</{}>", self.name))
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

impl QName {
    pub fn new(root: &Root, name: impl Into<String>) -> Self {
        let name = name.into();

        if !is_valid_qname(&name) {
            panic!("Invalid qname");
        }

        match name.split_once(":") {
            Some((a, b)) => match root.namespaces.get(a) {
                Some(ns) => {
                    return Self {
                        namespace: Some(ns.clone()),
                        name: b.to_string(),
                    }
                }
                None => {
                    panic!("No namespace found oh no: {}", a);
                }
            },
            None => {}
        }

        Self {
            namespace: None,
            name,
        }
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
    attributes: Rc<RefCell<BTreeMap<QName, String>>>,
    children: Rc<RefCell<Vec<Node>>>,
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
    let mut root = Root::new("root");
    root.set_main_namespace(Url::parse("http://wat.lol").unwrap());
    let mlem_ns = root.add_namespace(Url::parse("http://test.url/lol/").unwrap(), "mlem");

    let test = root.element("test");
    {
        let test_ref = test.borrow();
        test_ref.add_child(root.element("mlem"));

        let e = Element::new(QName::with_namespace(mlem_ns.clone(), "AHHHHHH"));
        {
            let e_ref = e.borrow();
            let ns = mlem_ns.clone();
            e_ref.add_attr(QName::with_namespace(ns.clone(), "test"), "amusement");

            e_ref.add_attr(
                QName::with_namespace(ns, "smart"),
                "<injection attack \0\0\0\0\0 \"'&'\"/>",
            )
        }

        test_ref.add_child(e);
        test_ref.add_child(root.element("mlem"));
    }

    root.element.borrow().add_child(test);
    println!("{}", root);
}

#[test]
fn smoke2() {
    let mut root = Root::from_str(
        r#"<root xmlns:x="http://lol" someattr="true">lol <x:sparta/><sparta derp="9000"></sparta> </root>"#,
    );
    root.set_ns_short_name("x", "huehuehue");
    println!("{}", root);
}
