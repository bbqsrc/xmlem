use std::borrow::Cow;

pub use element::Element;

pub mod doc;
mod element;
pub mod node;
mod qname;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid QName: {0}")]
    InvalidQName(String),
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

#[test]
fn smoke() {
    let root = Element::root("root").unwrap();
    let root_el = root.borrow();
    root_el.set_local_main_namespace(Some(Url::parse("http://wat.lol").unwrap()));
    let mlem_ns = root_el.add_local_namespace(Url::parse("http://test.url/lol/").unwrap(), "mlem");

    let test = Element::new_child(&root, "test").unwrap();
    {
        Element::new_child(&test, "mlem").unwrap();

        let e = Element::new_child_ns(&test, QName::with_namespace(mlem_ns.clone(), "AHHHHHH"))
            .unwrap();
        {
            let e_ref = e.borrow();
            let ns = mlem_ns.clone();
            e_ref.add_attr(QName::with_namespace(ns.clone(), "test"), "amusement");

            e_ref.add_attr(
                QName::with_namespace(ns, "smart"),
                "<injection attack \0\0\0\0\0 \"'&'\"/>",
            )
        }
        Element::new_child(&test, "mlem2").unwrap();
    }

    println!("{}", root.borrow());
}

#[test]
fn smoke2() {
    let mut root = Element::from_str(
        r#"<root xmlns:x="http://lol" someattr="true">lol <x:sparta/><sparta derp="9000"></sparta> </root>"#,
    ).unwrap();
    root.borrow().set_local_ns_short_name("x", "huehuehue");
    println!("{}", root.borrow());
}

#[test]
fn smoke3() {
    let input = r#"<俄语 լեզու="ռուսերեն">данные</俄语>"#;
    let root = Element::from_str(input).unwrap();

    println!("{}", root.borrow());
}
