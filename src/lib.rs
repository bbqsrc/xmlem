use std::{borrow::Cow, cell::RefCell, collections::{BTreeMap, BTreeSet}, fmt::{Display, LowerHex}, rc::Rc};

use url::Url;

pub struct Root {
    main_namespace: Option<Url>,
    namespaces: BTreeSet<Rc<Namespace>>,
    element: Rc<RefCell<Element>>,
}

impl Display for Root {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let element = self.element.borrow();

        f.write_fmt(format_args!("<{}", element.name))?;

        if let Some(xmlns) = self.main_namespace.as_ref() {
            f.write_fmt(format_args!(" xmlns=\"{}\"", xmlns))?;
        }

        for ns in &self.namespaces {
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
            let child = RefCell::borrow(child);
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
            name: QName::new(name),
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

    pub fn add_namespace(&mut self, url: Url, short_name: impl Into<String>) -> Rc<Namespace> {
        let ns = Rc::new(Namespace {
            url,
            short_name: short_name.into(),
        });

        self.namespaces.insert(ns.clone());

        ns
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
        children.push(child);
    }

    pub fn add_attr(&self, key: QName, value: impl Into<String>) {
        let mut attrs = RefCell::borrow_mut(&self.attributes);
        attrs.insert(key, value.into());
    }
}

impl Display for QName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.namespace.as_ref().map(|x| &**x) {
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
            let child = RefCell::borrow(child);
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct QName {
    namespace: Option<Rc<Namespace>>,
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
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        if !is_valid_qname(&name) {
            panic!("Invalid qname");
        }

        Self {
            namespace: None,
            name,
        }
    }

    pub fn with_namespace(ns: Rc<Namespace>, name: impl Into<String>) -> Self {
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

pub struct Element {
    name: QName,
    attributes: Rc<RefCell<BTreeMap<QName, String>>>,
    children: Rc<RefCell<Vec<Rc<RefCell<Element>>>>>,
}

impl TryFrom<String> for QName {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(QName::new(value))
    }
}

impl TryFrom<&str> for QName {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(QName::new(value))
    }
}

#[test]
fn smoke() {
    let mut root = Root::new("root");
    root.set_main_namespace(Url::parse("http://wat.lol").unwrap());
    let mlem_ns = root.add_namespace(Url::parse("http://test.url/lol/").unwrap(), "mlem");

    let test = Element::new("test".try_into().unwrap());
    {
        let test_ref = test.borrow();
        test_ref.add_child(Element::new("mlem".try_into().unwrap()));

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
        test_ref.add_child(Element::new("mlem".try_into().unwrap()));
    }

    root.element.borrow().add_child(test);
    println!("{}", root);
}
