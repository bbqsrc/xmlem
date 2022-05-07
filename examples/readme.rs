use std::str::FromStr;

use xmlem::{Document, Selector, Declaration};

fn main() {
    let mut doc = Document::from_str("<root><potato /></root>").unwrap();
    let root = doc.root();
    
    let potato = root.query_selector(&doc, &Selector::new("potato").unwrap()).unwrap();
    potato.append_new_element(&mut doc, (
        "wow",
        [
            ("easy", "true"),
            ("x", "200"),
        ],
    ));
        
    let decl = Declaration {
        version: Some("1.1".to_string()),
        encoding: Some("utf-8".to_string()),
        standalone: None,
    };
    doc.set_declaration(Some(decl));

    println!("{}", doc.to_string_pretty());
}