use std::{cell::RefCell, rc::Rc};

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

/*
impl Clone for XmlemDocument {
    fn clone(&self) -> Self {

    }

    fn clone_from(&mut self, source: &Self) {

    }
}
*/
