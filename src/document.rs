use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::Element;

pub struct XmlemDocument {
    root: Rc<RefCell<Element>>,
}

impl XmlemDocument {
    pub fn from_str(s: &str) -> Result<XmlemDocument, quick_xml::Error> {
        let root = Element::from_str(s)?;

        Ok(XmlemDocument { root })
    }
}

impl Clone for XmlemDocument {
    fn clone(&self) -> Self {
        let borrow_root = &*self.root.borrow();

        Self {
            root: Rc::new(RefCell::new(borrow_root.clone())),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone();
    }
}

impl Display for XmlemDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", &*self.root.borrow()))
    }
}
