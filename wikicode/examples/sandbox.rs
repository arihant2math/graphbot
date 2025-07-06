fn main() {
    let input = "{{Bot|GalStar|site=en|status=active}}
{{Toolforge bot}}

'''This bot is in production, please report all bugs/mistakes to the operator ([[User:GalStar]]) if possible.'''

'''To shut this bot down, create the page [[User:GraphBot/Shutdown]], this will ensure it shuts down gracefully.'''

== Invocation ==
At the moment the chart extension doesn't support every usecase that [[Template:Graph:Chart]] supports. As such, to mark a chart as ready for porting, you must instead change the template name to [[Template:PortGraph]]
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
