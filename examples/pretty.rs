fn main() {
    let path = std::env::args()
        .skip(1)
        .next()
        .expect("Path to SVG missing");
    let doc = {
        let file = std::fs::File::open(&path).expect("Could not open file");
        xmlem::Document::from_file(file).expect("Got invalid XML document")
    };
    println!("{:#}", doc);
}
