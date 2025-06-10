# stdlib imports
import json
import tomllib
import time
from xmlrpc.server import SimpleXMLRPCServer
from xmlrpc.server import SimpleXMLRPCRequestHandler

# parser import
import mwparserfromhell


# Load configuration
config_path = "conf/main.toml"
config = tomllib.load(open(config_path, "rb"))


# Restrict to a particular path.
class RequestHandler(SimpleXMLRPCRequestHandler):
    rpc_paths = (config["rpc"]["path"],)


# Create server
print(f"Starting XML-RPC server on port {config['rpc']['port']}...")
with SimpleXMLRPCServer(
    ("localhost", config["rpc"]["port"]), requestHandler=RequestHandler
) as server:
    server.register_introspection_functions()

    def parser(text: str) -> str:
        t1 = time.time()
        wikitext = mwparserfromhell.parse(text)
        templates = []
        for template in wikitext.filter_templates():
            templates.append(
                {
                    "name": str(template.name).strip(),
                    "params": {
                        str(param.name).strip(): str(param.value).strip().strip('"')
                        for param in template.params
                    },
                    "wikitext": str(template).strip(),
                }
            )
        tags = []
        for tag in wikitext.filter_tags():
            if str(tag.tag.nodes[0]) == "graph":
                tags.append(str(tag.contents).strip())
        t2 = time.time()
        return {"templates": templates, "tags": tags, "elapsed": t2 - t1}

    server.register_function(parser, "parse")

    # Run the server's main loop
    server.serve_forever()
