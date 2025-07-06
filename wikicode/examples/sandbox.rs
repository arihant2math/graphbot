fn main() {
    let input = "
#REDIRECT [[United States#History]]
".to_string();
    println!("Tokenizing input:\n{}", input);

    match wikicode::tokenize(input) {
        Ok(out) => {
            println!("{:#?}", out);
        }
        Err(e) => {
            // print the error
            eprintln!("Error parsing input: {}", e);
        }
    }
}
