import json
import mwparserfromhell

# Without utf-8 encoding, the script may not handle some pages correctly
text = ""
with open("in.txt", "r", encoding="utf-8") as f:
    text = f.read()

templates = mwparserfromhell.parse(text).filter_templates()

output = []
for template in templates:
    output.append({
        "name": str(template.name).strip(),
        "params": {str(param.name).strip(): str(param.value).strip() for param in template.params},
        "wikitext": str(template).strip()
    })

with open("out.json", "w", encoding="utf-8") as f:
    json.dump({"data": output}, f, indent=2, ensure_ascii=False)
