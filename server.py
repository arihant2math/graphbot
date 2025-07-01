# stdlib imports
import json
import tomllib
import time
from xmlrpc.server import SimpleXMLRPCServer
from xmlrpc.server import SimpleXMLRPCRequestHandler

# parser imports
import mwparserfromhell
from mwparserfromhell import nodes
from mwparserfromhell.nodes import Node

def convert_node(node: Node):
    match node:
        case nodes.extras.Attribute():
            return {
                "text": str(node),
                "type": "Attribute",
                "pad_after_eq": node.pad_after_eq,
                "pad_before_eq": node.pad_before_eq,
                "pad_first": node.pad_first,
                "quotes": node.quotes if node.quotes is not None else None,
                "value": convert_wikicode(node.value) if node.value is not None else None,
            }
        case nodes.extras.Parameter():
            return {
                "text": str(node),
                "name": str(node.name),
                "type": "Parameter",
                "showkey": node.showkey,
                "value": str(node.value) if node.value is not None else None,
            }
        case nodes.Argument():
            return {
                "text": str(node),
                "type": "Argument",
                "name": convert_wikicode(node.name),
                "default": convert_wikicode(node.default) if node.default is not None else None,
            }
        case nodes.Comment():
            return {
                "text": str(node),
                "type": "Comment",
                "contents": str(node.contents)
            }
        case nodes.ExternalLink():
            return {
                "text": str(node),
                "type": "ExternalLink",
                "brackets": node.brackets,
                "title": convert_wikicode(node.title) if node.title is not None else None,
                "url": convert_wikicode(node.url),
            }
        case nodes.Heading():
            return {
                "text": str(node),
                "type": "Heading",
                "level": node.level,
                "title": convert_wikicode(node.title),
            }
        case nodes.HTMLEntity():
            return {
                "text": str(node),
                "type": "HTMLEntity",
                "hex_char": node.hex_char,
                "hexadecimal": node.hexadecimal,
                "named": node.named,
                "value": node.value,
            }
        case nodes.Tag():
            return {
                "text": str(node),
                "type": "Tag",
                "attributes": [convert_node(n) for n in node.attributes],
                "closing_tag": str(node.closing_tag) if node.closing_tag else None,
                "closing_wiki_markup": str(node.closing_wiki_markup) if node.closing_wiki_markup else None,
                "contents": convert_wikicode(node.contents),
                "implicit": node.implicit,
                "invalid": node.invalid,
                "padding": node.padding,
                "self_closing": node.self_closing,
                "tag": str(node.tag) if node.tag else None,
                "wiki_markup": str(node.wiki_markup) if node.wiki_markup else None,
                "wiki_style_separator": str(node.wiki_style_separator) if node.wiki_style_separator else None,
            }
        case nodes.Template():
            return {
                "text": str(node),
                "type": "Template",
                "name": convert_wikicode(node.name),
                "params": [convert_node(param) for param in node.params],
            }
        case nodes.Text():
            return {
                "text": str(node),
                "type": "Text",
                "value": str(node.value),
            }
        case nodes.Wikilink():
            return {
                "text": str(node),
                "type": "Wikilink",
                "txt": convert_wikicode(node.text) if node.text else None,
                "title": convert_wikicode(node.title),
            }
        case _:
            raise ValueError(f"Unsupported node type: {type(node)}")

def convert_wikicode(wikitext: mwparserfromhell.wikicode.Wikicode):
    return {
        "headings": [convert_node(h) for h in wikitext.filter_headings()],
        "templates": [convert_node(t) for t in wikitext.filter_templates()],
        "tags": [convert_node(t) for t in wikitext.filter_tags()],
        "nodes": [convert_node(n) for n in wikitext.nodes],
        "text": str(wikitext),
    }

# Load configuration
config_path = "conf/main.toml"
config = tomllib.load(open(config_path, "rb"))


# Restrict to a particular path.
class RequestHandler(SimpleXMLRPCRequestHandler):
    rpc_paths = (config["rpc"]["path"],)


# Create server
print(f"Starting XML-RPC server on port {config['rpc']['port']}...")
with SimpleXMLRPCServer(
    ("localhost", config["rpc"]["port"]), requestHandler=RequestHandler, allow_none=True
) as server:
    server.register_introspection_functions()

    def parser(text: str) -> dict[str, str | float]:
        t1 = time.time()
        wikitext = mwparserfromhell.parse(text)
        output = convert_wikicode(wikitext)
        t2 = time.time()
        return {"parsed": json.dumps(output), "elapsed": t2 - t1}

    server.register_function(parser, "parse")

    # Run the server's main loop
    server.serve_forever()
