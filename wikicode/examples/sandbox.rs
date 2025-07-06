fn main() {
    let input = "{{Toolforge bot}}
'''This bot is in production, please report all bugs/mistakes to the operator ([[User:GalStar]]) if possible.'''

== Invocation ==
At the moment the chart extension doesn't support every usecase that [[Template:Graph:Chart]] supports.
".to_string();

    // == See Also ==
    //     * [[:c:User:GraphBot]] - more details
    //     * [[Template:PortGraph]]
    //     * [[:Category:Pages using the Graph extension]] - Graphs to port
    //     * [[:Category:Graphs to Port]] - Graphs that have been queued for processing. If a graph remains in this queue for too long it might mean that there has been a failure converting the graph; [[User:GraphBot/Conversion Errors]] might be of assistance.

        // parse the input
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