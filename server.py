from xmlrpc.server import SimpleXMLRPCServer
from xmlrpc.server import SimpleXMLRPCRequestHandler
import json
import mwparserfromhell

# Restrict to a particular path.
class RequestHandler(SimpleXMLRPCRequestHandler):
    rpc_paths = ('/RPC2',)

# Create server
with SimpleXMLRPCServer(('localhost', 8000),
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
