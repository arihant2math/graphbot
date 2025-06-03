import mwparserfromhell

text = open("in.txt", "r").read()

templates = mwparserfromhell.parse(text).filter_templates()

output = []
for template in templates:
    output.append({
        "name": str(template.name).strip(),
        "params": {str(param.name).strip(): str(param.value).strip() for param in template.params}
    })

with open("out.json", "w") as f:
    import json
    json.dump({"data": output}, f, indent=2, ensure_ascii=False)
