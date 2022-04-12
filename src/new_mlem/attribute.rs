use super::qname::QName;

pub struct Attribute {
    key: QName,
    value: String,
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
