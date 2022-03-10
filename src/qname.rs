use std::{cell::RefCell, fmt::Display, rc::Rc};

use url::Url;

use crate::{element::Element, Error};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Namespace {
    // Fields used to be private
    pub url: Url,
    pub short_name: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct QName {
    // Fields used to be private
    pub namespace: Option<Rc<RefCell<Namespace>>>,
    pub name: String,
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

impl Clone for QName {
    fn clone(&self) -> Self {
        let namespace = match self.namespace.as_ref() {
            Some(ref_namespace) => {
                let hmm = &*ref_namespace.borrow();

                let inner_clone = hmm.clone();
                let cell = RefCell::new(inner_clone);
                let rc = Rc::new(cell);
                Some(rc)
            },
            None => None,
        };

        Self {
            namespace,
            name: self.name.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) { 
        *self = source.clone();
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

pub fn is_valid_qname(input: &str) -> bool {
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
