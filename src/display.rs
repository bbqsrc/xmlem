use std::{
    borrow::Cow,
    fmt::Display,
    io::{self, Write},
};

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
            write!(f, "version=\"{}\" ", version)?;
        }

        if let Some(encoding) = self.encoding.as_deref() {
            write!(f, "encoding=\"{}\" ", encoding)?;
        }

        if let Some(standalone) = self.standalone.as_deref() {
            write!(f, "standalone=\"{}\" ", standalone)?;
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

        for node in self.before.iter() {
            let node_value = self.items.get(node.as_key()).unwrap().as_node().unwrap();
            node_value
                .display_fmt(self, node.as_key(), f, 0)
                .map_err(|_| std::fmt::Error)?;
        }

        let element = self
            .items
            .get(self.root_key.0)
            .unwrap()
            .as_element()
            .unwrap();

        element
            .display_fmt(self, self.root_key.0, f, 0)
            .map_err(|_e| std::fmt::Error)?;

        for node in self.after.iter() {
            let node_value = self.items.get(node.as_key()).unwrap().as_node().unwrap();
            node_value
                .display_fmt(self, node.as_key(), f, 0)
                .map_err(|_e| std::fmt::Error)?;
        }

        Ok(())
    }
}

fn fmt_attrs(f: &mut dyn Write, attrs: &IndexMap<String, String>) -> io::Result<()> {
    let mut iter = attrs.iter();

    if let Some((k, v)) = iter.next() {
        write!(f, "{}=\"{}\"", k, process_entities(v))?;
    } else {
        return Ok(());
    }

    for (k, v) in iter {
        write!(f, " {}=\"{}\"", k, process_entities(v))?;
    }

    Ok(())
}

impl ElementValue {
    pub(crate) fn display(
        &self,
        doc: &Document,
        k: DocKey,
        f: &mut dyn Write,
        indent: usize,
        alternate: bool,
    ) -> io::Result<()> {
        if self.children.is_empty() {
            match doc.attrs.get(k) {
                Some(attrs) if !attrs.is_empty() => {
                    write!(f, "{:>indent$}<{} ", "", self.name, indent = indent)?;
                    fmt_attrs(f, attrs)?;
                    write!(f, " />")?;
                    if alternate {
                        writeln!(f)?;
                    }
                    return Ok(());
                }
                _ => {
                    write!(f, "{:>indent$}<{} />", "", self.name, indent = indent)?;
                    if alternate {
                        writeln!(f)?;
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
                if alternate {
                    writeln!(f)?;
                }
            }
            _ => {
                write!(f, "{:>indent$}<{}>", "", self.name, indent = indent)?;
                if alternate {
                    writeln!(f)?;
                }
            }
        }

        let child_indent = if alternate { indent + 2 } else { 0 };
        for child in self.children.iter() {
            let value = doc.items.get(child.as_key()).unwrap();
            value.display(doc, child.as_key(), f, child_indent, alternate)?;
        }
        write!(f, "{:>indent$}</{}>", "", self.name, indent = indent)?;

        if alternate {
            writeln!(f)?;
        }

        Ok(())
    }

    pub(crate) fn display_fmt(
        &self,
        doc: &Document,
        k: DocKey,
        f: &mut std::fmt::Formatter<'_>,
        indent: usize,
    ) -> io::Result<()> {
        let alternate = f.alternate();
        self.display(doc, k, &mut FmtWriter(f), indent, alternate)
    }
}

impl ItemValue {
    fn display(
        &self,
        doc: &Document,
        k: DocKey,
        f: &mut dyn Write,
        indent: usize,
        alternate: bool,
    ) -> io::Result<()> {
        match self {
            ItemValue::Node(n) => n.display(doc, k, f, indent, alternate),
        }
    }
}

impl NodeValue {
    pub(crate) fn display(
        &self,
        doc: &Document,
        k: DocKey,
        f: &mut dyn Write,
        indent: usize,
        alternate: bool,
    ) -> io::Result<()> {
        if let NodeValue::Element(e) = self {
            return e.display(doc, k, f, indent, alternate);
        }

        if alternate {
            write!(f, "{:>indent$}", "", indent = indent)?;
        }

        match self {
            NodeValue::Text(t) => write!(f, "{}", &*process_entities(t).trim()),
            NodeValue::CData(t) => write!(f, "<![CDATA[{}]]>", t),
            NodeValue::DocumentType(t) => write!(f, "<!DOCTYPE{}>", t),
            NodeValue::Comment(t) => write!(f, "<!--{}-->", t),
            NodeValue::Element(_) => unreachable!(),
        }?;

        if alternate {
            writeln!(f)?;
        }

        Ok(())
    }

    fn display_fmt(
        &self,
        doc: &Document,
        k: DocKey,
        f: &mut std::fmt::Formatter<'_>,
        indent: usize,
    ) -> io::Result<()> {
        let alternate = f.alternate();
        self.display(doc, k, &mut FmtWriter(f), indent, alternate)
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

struct FmtWriter<'a, 'b>(&'b mut std::fmt::Formatter<'a>);

impl Write for FmtWriter<'_, '_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = std::str::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        self.0
            .write_str(s)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(s.as_bytes().len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
