fn main() {
    let input = "{{AWB topicon}}
{{Discord topicon}}
{{Huggle topicon}}
{{Twinkle topicon}}

{{Infobox Wikipedia user
| name        = GalStar
| location    = [[Earth]]
| country     = [[United States]]
| nationality = [[United States|American]]
| joined_date = January 20, 2020
| first_edit  = January 20, 2020
| userboxes   = {{User en}}
{{User Wikipedia/Extended confirmed}}
{{User Wikipedia/Rollback}}
{{User Wikipedia/RC Patrol}}
{{User Anti-Vandalism}}
{{User script developer}}
{{User Wikipedia/Bot operator|GraphBot}}
{{User WikiProject Data Visualization}}
}}
    ".to_string();
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