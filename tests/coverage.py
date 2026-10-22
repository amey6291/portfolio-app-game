"""Confirm every skill, certification and headline number in skills.json
appears as text on each built page. Usage: python3 tests/coverage.py"""
import html, json, re, sys
from pathlib import Path

root = Path(__file__).resolve().parent.parent
data = json.loads((root / "src/data/skills.json").read_text())
want = [s["skill"] for c in data["categories"] for s in c["skills"]]
want += [c["name"] for c in data["certifications"]] + data["headline_numbers"]

def text(p):
    t = re.sub(r"<[^>]+>", " ", (root / "dist" / p).read_text())
    return re.sub(r"\s+", " ", html.unescape(t))

failed = False
for page in ["index.html", "resume/index.html"]:
    t = text(page)
    missing = [w for w in want if w not in t]
    print(f"{page}: {len(want) - len(missing)}/{len(want)} present")
    if missing:
        failed = True
        print("  missing:", *missing, sep="\n    ")
sys.exit(failed)
