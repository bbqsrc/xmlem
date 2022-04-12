mod attribute;
mod document;
mod element;
mod node;
mod qname;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error from quick_xml: `{0}`")]
    QuickXml(#[from] quick_xml::Error),

    #[error("Invalid QName: {0}")]
    InvalidQName(String),

    #[error("UTF-8 error from std: `{0}`")]
    StdUtf8(#[from] std::str::Utf8Error),
}
