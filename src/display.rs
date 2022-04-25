use std::{borrow::Cow, fmt::Display};

use indexmap::IndexMap;

use crate::{
    document::{Declaration, Document},
    key::DocKey,
    value::{ElementValue, ItemValue, NodeValue},
};

impl Display for Declaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<?xml ")?;

        if let Some(version) = self.version.as_deref() {
            write!(f, "version=\"{}\"", version)?;
        }

        if let Some(encoding) = self.encoding.as_deref() {
            write!(f, "encoding=\"{}\"", encoding)?;
        }

        if let Some(standalone) = self.standalone.as_deref() {
            write!(f, "standalone=\"{}\"", standalone)?;
        }

        f.write_str("?>")?;

        if f.alternate() {
            f.write_str("\n")?;
        }

        Ok(())
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(decl) = self.decl.as_ref() {
            Display::fmt(&decl, f)?;
        }

        if let Some(doctype) = self.doctype.as_ref() {
            write!(f, "<!DOCTYPE{}>", doctype)?;

            if f.alternate() {
                f.write_str("\n")?;
            }
        }

        let element = self
            .items
            .get(self.root_key.0)
            .unwrap()
            .as_element()
            .unwrap();

        element.display_fmt(self, self.root_key.0, f, 0)
    }
}

fn fmt_attrs(
    f: &mut std::fmt::Formatter<'_>,
    attrs: &IndexMap<String, String>,
) -> std::fmt::Result {
    let mut iter = attrs.iter();

    if let Some((k, v)) = iter.next() {
        write!(f, "{}=\"{}\"", k, process_entities(v))?;
    } else {
        return Ok(());
    }

    while let Some((k, v)) = iter.next() {
        write!(f, " {}=\"{}\"", k, process_entities(v))?;
    }

    Ok(())
}

impl ElementValue {
    fn display_fmt(
        &self,
        doc: &Document,
        k: DocKey,
        f: &mut std::fmt::Formatter<'_>,
        indent: usize,
    ) -> std::fmt::Result {
        if self.children.is_empty() {
            match doc.attrs.get(k) {
                Some(attrs) if !attrs.is_empty() => {
                    write!(f, "{:>indent$}<{} ", "", self.name, indent = indent)?;
                    fmt_attrs(f, attrs)?;
                    write!(f, " />")?;
                    if f.alternate() {
                        f.write_str("\n")?;
                    }
                    return Ok(());
                }
                _ => {
                    write!(f, "{:>indent$}<{} />", "", self.name, indent = indent)?;
                    if f.alternate() {
                        f.write_str("\n")?;
                    }
                    return Ok(());
                }
            }
        }

        match doc.attrs.get(k) {
            Some(attrs) if !attrs.is_empty() => {
                write!(f, "{:>indent$}<{} ", "", self.name, indent = indent)?;
                fmt_attrs(f, attrs)?;
                write!(f, ">")?;
                if f.alternate() {
                    f.write_str("\n")?;
                }
            }
            _ => {
                write!(f, "{:>indent$}<{}>", "", self.name, indent = indent)?;
                if f.alternate() {
                    f.write_str("\n")?;
                }
            }
        }

        let child_indent = if f.alternate() { indent + 2 } else { 0 };
        for child in self.children.iter() {
            let value = doc.items.get(child.as_key()).unwrap();
            value.display_fmt(doc, child.as_key(), f, child_indent)?;
        }
        write!(f, "{:>indent$}</{}>", "", self.name, indent = indent)?;

        if f.alternate() {
            f.write_str("\n")?;
        }

        Ok(())
    }
}

impl ItemValue {
    fn display_fmt(
        &self,
        doc: &Document,
        k: DocKey,
        f: &mut std::fmt::Formatter<'_>,
        indent: usize,
    ) -> std::fmt::Result {
        match self {
            ItemValue::Node(n) => n.display_fmt(doc, k, f, indent),
        }
    }
}

impl NodeValue {
    fn display_fmt(
        &self,
        doc: &Document,
        k: DocKey,
        f: &mut std::fmt::Formatter<'_>,
        indent: usize,
    ) -> std::fmt::Result {
        match self {
            NodeValue::Element(e) => e.display_fmt(doc, k, f, indent),
            NodeValue::Text(t) => f.write_str(&*process_entities(t)),
            NodeValue::CData(t) => write!(f, "<![CDATA[[{}]]>", t),
        }
    }
}

fn process_entities(input: &str) -> Cow<'_, str> {
    if input.contains(['<', '>', '\'', '"', '&']) || input.contains(|c: char| c.is_ascii_control())
    {
        let mut s = String::with_capacity(input.len());
        input.chars().for_each(|ch| {
            s.push_str(match ch {
                '\'' => "&apos;",
                '"' => "&quot;",
                '&' => "&amp;",
                '<' => "&lt;",
                '>' => "&gt;",
                ch if ch.is_ascii_control() => {
                    s.push_str(&format!("&#x{:>04X};", ch as u32));
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
