from xmlrpc.server import SimpleXMLRPCServer
from xmlrpc.server import SimpleXMLRPCRequestHandler
import json
import tomllib
import mwparserfromhell

config_path = "conf/main.toml"
# Load configuration
config = tomllib.load(open(config_path, 'rb'))
print(config)

# Restrict to a particular path.
class RequestHandler(SimpleXMLRPCRequestHandler):
    rpc_paths = (config["rpc"]["path"],)

# Create server
with SimpleXMLRPCServer(('localhost', config["rpc"]["port"]),
                        requestHandler=RequestHandler) as server:
    server.register_introspection_functions()

    # Register a function under a different name
    def parser(text: str) -> str:
        templates = mwparserfromhell.parse(text).filter_templates()
        output = []
        for template in templates:
            output.append({
                "name": str(template.name).strip(),
                "params": {str(param.name).strip(): str(param.value).strip() for param in template.params},
                "wikitext": str(template).strip()
            })
        return json.dumps({"data": output}, indent=2, ensure_ascii=False)

    server.register_function(parser, 'parse')

    # Run the server's main loop
    server.serve_forever()
