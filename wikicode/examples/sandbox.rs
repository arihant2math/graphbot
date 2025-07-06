fn main() {
    let input = "== Test ==".to_string();
    // parse the input
    println!("Parsing input:\n{}", input);
    match wikicode::parse(&input) {
        Ok(wikicode) => {
            // print the parsed wikicode
            println!("{:#?}", wikicode);
        }
        Err(e) => {
            // print the error
            eprintln!("Error parsing input: {}", e);
        }
    }
}