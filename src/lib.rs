mod display;
mod document;
mod element;
mod key;
mod value;

pub use document::Document;
pub use element::{Element, NewElement};
pub use key::Node;

#[cfg(test)]
mod tests {
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

        let potato = doc.root().append_new_element(
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
}
