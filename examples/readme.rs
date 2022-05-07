use std::str::FromStr;

use xmlem::{Declaration, Document, Selector};

fn main() {
    let mut doc = Document::from_str("<root><potato /></root>").unwrap();
    let root = doc.root();

    let potato = root
        .query_selector(&doc, &Selector::new("potato").unwrap())
        .unwrap();
    potato.append_new_element(
        &mut doc,
        ("wow", [("easy", "true"), ("x", "200"), ("a-null", "\0")]),
    );

    let decl = Declaration::v1_1();
    doc.set_declaration(Some(decl));
    doc.set_doctype(Some("not-html"));

    println!("{}", doc.to_string_pretty());
}
