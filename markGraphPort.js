mw.loader.using(['mediawiki.util', 'mediawiki.api'], function () {
    const elements = document.querySelectorAll('[class="mbox-text-span"]');
    const text = "This graph was using the legacy Graph extension, which is no longer supported. It needs to be converted to the new Chart extension.";
    let config = mw.config.get([
        'debug',
        'wgAction',
        'wgArticleId',
        'wgCategories',
        'wgMonthNames',

        'wgNamespaceNumber',
        'wgPageName',
        'wgRelevantUserName'
    ]);
    const commons_api = new mw.ForeignApi( 'https://commons.wikimedia.org/w/api.php' );

    var counter = 0;
    elements.forEach(element => {
        if (element.textContent === text) {
            let space = document.createTextNode(' ');
            element.appendChild(space);
            let portLink = document.createElement('a');
            // clone the value of "counter" to use in the onclick function
            let clone = structuredClone(counter);
            portLink.onclick = function (ev) {
                ev.preventDefault();
                portGraph(clone);
            };
            portLink.href = "#";
            portLink.textContent = "[Port graph]";
            portLink.classList.add('extiw');
            element.appendChild(portLink);
            console.log(element);
            counter++;
        }
    });

    async function portGraphInner(number, name) {
        if (!name) {
            throw new Error("Name is required");
        }
        if (name.length > 255) {
            throw new Error("Name is too long");
        }
        if (name.length < 3) {
            throw new Error("Name is too short");
        }
        if (name.endsWith(".chart") || name.endsWith(".tab")) {
            throw new Error("Name should not include file extension");
        }
        if (name.startsWith("Data:")) {
            throw new Error("Name should not include 'Data:' prefix");
        }
        if (name.includes(":")) {
            throw new Error("Name should not include namespace prefix");
        }
        if (name.includes("|") || name.includes("{") || name.includes("}") || name.includes("[") || name.includes("]" || name.includes("//"))) {
            throw new Error("Name contains invalid character(s)");
        }
        let urlEdName = name.replace(' ', '_');
        let response = await commons_api.get({
            action: 'query',
            titles: [`Data:${urlEdName}.chart`, `Data:${urlEdName}.tab`],
            format: 'json',
        });
        let pages = response.query.pages;
        console.log(Object.keys(pages));
        if (!(Object.keys(pages).includes("-1") && Object.keys(pages).includes("-2"))) {
            throw Error("Pages exist on commons");
        }
        let pageid = config.wgArticleId;
        new mw.Api().get({
            action: 'parse',
            pageid: pageid,
            prop: 'wikitext',
            formatversion: 2,
        }).done(function (response) {
            try {
                console.log(response);
                let content = response.parse.wikitext;
                // Find the nth graph
                console.log(content.includes("{{Graph:Chart"));
                let graphs = content.match(/\{\{Graph:?Chart[\s\S]*?\}\}/mgi);
                console.log(graphs);
                if (!graphs) {
                    throw new Error("Graph not found");
                }
                if (counter !== graphs.length) {
                    throw new Error("Mismatch in graph count");
                }
                let graph = graphs[number];
                console.log("Found graph: " + graph);
                // Convert to chart
                let ported = graph.replace(/{{Graph:?Chart/g, '{{PortGraph|name=' + name + '|');
                console.log("Ported graph: " + ported);
                let redone = content.replace(graph, ported);
                // Create a new page with the ported graph
                new mw.Api().postWithToken('edit', {
                    action: 'edit',
                    pageid: pageid,
                    text: redone,
                    summary: 'Marked [[Help:Graph|Graph]] for porting',
                }).done(function (response) {
                    console.log(response);
                    if (response.edit && response.edit.result === 'Success') {
                        window.location.reload();
                    } else {
                        throw new Error("Error editing page: " + JSON.stringify(response));
                    }
                }).fail(function (error) {
                    throw new Error("Error editing page: " + JSON.stringify(error));
                });
            } catch (e) {
                alert("Error: " + e.message);
            }
        });
    }

    function portGraph(number) {
        console.log("Porting graph #" + number);
        let name = prompt("What should the name of the new chart be?");
        try {
            portGraphInner(number, name);
        } catch (e) {
            alert("Error: " + e.message);
        }
    }
});
