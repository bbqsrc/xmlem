use std::fmt::Display;

use super::qname::QName;

#[derive(Debug)]
pub struct Attribute {
    key: QName,
    value: String,
}

impl Clone for Attribute {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone();
    }
}

impl Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.key, f)?;
        Display::fmt(&self.value, f)?;

        Ok(())
    }
}

pub fn create_attribute(
    key: impl Into<String>,
    value: impl Into<String>,
) -> Result<Attribute, super::Error> {
    let qname = QName::new_without_namespace(key)?;

    Ok(Attribute {
        key: qname,
        value: value.into(),
    })
}
