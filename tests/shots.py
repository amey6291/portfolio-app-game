"""Screenshots across the journey at desktop and phone width.

Usage: .venv/bin/python tests/shots.py [BASE_URL] [STOPS | #level-id] [N]
  STOPS: comma-separated scroll fractions (default covers the whole world)
  #level-id: N evenly spaced shots (default 7) while the hero crosses that level
Writes tests/screenshots/<viewport>[-level]-<nn>.png and fails on console errors.
"""
import sys
from pathlib import Path
from playwright.sync_api import sync_playwright

BASE = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:8080/"
ARG = sys.argv[2] if len(sys.argv) > 2 else ""
LEVEL = ARG if ARG.startswith("#") else None
N = int(sys.argv[3]) if len(sys.argv) > 3 else 7
STOPS = [float(s) for s in ARG.split(",")] if ARG and not LEVEL else [0, .08, .16, .25, .35, .45, .55, .65, .75, .87, 1]
OUT = Path(__file__).parent / "screenshots"
OUT.mkdir(exist_ok=True)

VIEWPORTS = {
    "desktop": dict(viewport={"width": 1440, "height": 900}),
    "phone": dict(viewport={"width": 390, "height": 844}, is_mobile=True, has_touch=True, device_scale_factor=2),
}

errors = []
with sync_playwright() as p:
    browser = p.chromium.launch(channel="chrome")
    for name, opts in VIEWPORTS.items():
        page = browser.new_page(**opts)
        page.on("console", lambda m: m.type == "error" and errors.append(f"{name}: {m.text}"))
        page.on("pageerror", lambda e: errors.append(f"{name}: {e}"))
        page.goto(BASE, wait_until="networkidle")
        page.wait_for_selector("html.ready", timeout=5000)
        max_y = page.evaluate("document.documentElement.scrollHeight - innerHeight")
        if LEVEL:
            # Scroll range over which the hero crosses the level.
            a, b = page.evaluate("""sel => { const w = document.querySelector('#world').getBoundingClientRect().left,
                r = document.querySelector(sel).getBoundingClientRect(), h = document.querySelector('#hero').getBoundingClientRect();
                const hx = h.left + h.width / 2; return [r.left - w - hx, r.right - w - hx]; }""", LEVEL)
            a, b = max(a, 0), min(b, max_y)
            targets = [a + (b - a) * i / (N - 1) for i in range(N)]
            page.evaluate(f"scrollTo(0, {max(a - 400, 0)})")
            page.wait_for_timeout(200)
        else:
            targets = [max_y * f for f in STOPS]
        tag = f"{name}-{LEVEL[1:]}" if LEVEL else name
        for i, target in enumerate(targets):
            # Walk there in steps so reveals/toasts fire as they would for a reader.
            target = int(target)
            page.evaluate(
                """async t => { const s = scrollY, n = 12;
                   for (let i = 1; i <= n; i++) { scrollTo(0, s + (t - s) * i / n);
                     await new Promise(r => requestAnimationFrame(r)); } }""",
                target,
            )
            page.wait_for_timeout(700)
            page.screenshot(path=str(OUT / f"{tag}-{i:02d}.png"))
        page.close()

    if LEVEL:
        browser.close()
        if errors:
            print("console errors:\n  " + "\n  ".join(errors)); sys.exit(1)
        print(f"ok: {LEVEL} shots in {OUT}"); sys.exit(0)
    # Reduced motion: static layout, no game class.
    page = browser.new_page(viewport={"width": 1440, "height": 900}, reduced_motion="reduce")
    page.goto(BASE, wait_until="networkidle")
    assert page.evaluate("!document.documentElement.classList.contains('game')"), "game mode under reduced motion"
    page.screenshot(path=str(OUT / "reduced-motion.png"), full_page=True)
    page.goto(BASE + "resume/", wait_until="networkidle")
    page.screenshot(path=str(OUT / "resume-desktop.png"), full_page=True)
    page.close()
    page = browser.new_page(**VIEWPORTS["phone"])
    page.goto(BASE + "resume/", wait_until="networkidle")
    page.screenshot(path=str(OUT / "resume-phone.png"))
    browser.close()

if errors:
    print("console errors:\n  " + "\n  ".join(errors))
    sys.exit(1)
print(f"ok: {len(list(OUT.glob('*.png')))} screenshots in {OUT}")
