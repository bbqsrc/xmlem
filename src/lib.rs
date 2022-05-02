mod display;
mod document;
mod element;
mod key;
mod select;
mod value;

pub use document::Document;
pub use element::{Element, NewElement};
pub use key::Node;
pub use select::Selector;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::select::Selector;

    use super::*;

    #[test]
    fn test() {
        use indexmap::IndexMap;

        use crate::document::Document;

        let mut doc = Document::new("root");

        let new_el = doc.root().append_new_element(
            &mut doc,
            NewElement {
                name: "child".into(),
                attrs: Default::default(),
            },
        );
        new_el.append_new_element(
            &mut doc,
            NewElement {
                name: "child2".into(),
                attrs: Default::default(),
            },
        );
        let mut attrs = IndexMap::new();
        attrs.insert("hello".into(), "yes".into());
        attrs.insert("another-thing".into(), "yes".into());

        let foo = new_el.append_new_element(
            &mut doc,
            NewElement {
                name: "with-child2".into(),
                attrs,
            },
        );
        foo.append_new_element(
            &mut doc,
            NewElement {
                name: "child3".into(),
                attrs: Default::default(),
            },
        );
        foo.append_new_element(
            &mut doc,
            NewElement {
                name: "child3".into(),
                attrs: Default::default(),
            },
        );
        foo.append_new_element(
            &mut doc,
            NewElement {
                name: "child3".into(),
                attrs: Default::default(),
            },
        );
        new_el.append_new_element(
            &mut doc,
            NewElement {
                name: "child2".into(),
                attrs: Default::default(),
            },
        );

        let _potato = doc.root().append_new_element(
            &mut doc,
            NewElement {
                name: "potato".into(),
                attrs: Default::default(),
            },
        );

        foo.parent(&mut doc)
            .unwrap()
            .remove_child(&mut doc, Node::Element(foo));

        println!("{}", doc);
    }

    #[test]
    fn smoke2() {
        let doc = Document::from_str(
        r#"<root xmlns:x="http://lol" someattr="true">lol <x:sparta/><sparta derp="9000"></sparta> </root>"#,
    ).unwrap();
        println!("{}", doc);
    }

    #[test]
    fn smoke3() {
        let input = r#"<俄语 լեզու="ռուսերեն">данные</俄语>"#;
        let doc = Document::from_str(input).unwrap();

        println!("{}", doc);
    }

    #[test]
    fn keylayout() {
        let input = include_str!("/Users/brendan/Library/Keyboard Layouts/so.brendan.keyboards.keyboardlayout.brendan.bundle/Contents/Resources/enusaltsv.keylayout");
        let doc = Document::from_str(input).unwrap();

        println!("{:#}", doc);
    }

    #[test]
    fn clone() {
        let input = r#"<俄语 լեզու="ռուսերեն">данные</俄语>"#;
        let doc = Document::from_str(input).unwrap();
        let mut doc2 = doc.clone();

        let root = doc2.root();
        root.append_new_element(
            &mut doc2,
            NewElement {
                name: "lol".to_string(),
                attrs: Default::default(),
            },
        );

        println!("{}", doc);
        println!("{}", doc2);
    }

    #[test]
    fn hmm() {
        let input = "<?xml version=\"1.1\" ?>some random text<![CDATA[<hahaha>]]><!DOCTYPE root ahh ahhhh><!-- pre --><root/><!-- comment --> some other text";
        let doc = Document::from_str(input).unwrap();
        println!("{:#}", doc);
    }

    #[test]
    fn svg() {
        let input = std::fs::read_to_string("/Users/brendan/Downloads/keyboard-iso.svg").unwrap();
        let mut doc = Document::from_str(&input).unwrap();

        let nodes = doc.root().children(&doc);
        let g = nodes.last().unwrap();
        let nodes = g.children(&doc).to_vec();
        for element in nodes {
            let children = element.children(&doc).to_vec();
            for el in children {
                let g = el.append_new_element(&mut doc, ("g", [("class", "key-group")]));

                let primary = g.append_new_element(
                    &mut doc,
                    (
                        "text",
                        [
                            ("dy", "1em"),
                            ("y", "32"),
                            ("x", "32"),
                            ("class", "key-text-primary"),
                        ],
                    ),
                );

                primary.append_text(&mut doc, "lol");

                let secondary = g.append_new_element(
                    &mut doc,
                    NewElement {
                        name: "text".into(),
                        attrs: [
                            ("dy".to_string(), "-.4em".to_string()),
                            ("class".to_string(), "key-text-secondary".to_string()),
                        ]
                        .into(),
                    },
                );

                secondary.append_text(&mut doc, "LOL");
            }
        }

        let sel = Selector::new("g").unwrap();

        println!(
            "{:?}",
            doc.root()
                .query_selector(&doc, &sel)
                .unwrap()
                .attributes(&doc)
        );
        println!("Count: {}", doc.root().query_selector_all(&doc, &sel).len());

        println!("{:#}", doc);
    }
}
