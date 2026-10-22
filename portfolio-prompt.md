# Prompt: build my side-scrolling game portfolio

> Paste everything below the line into a Claude Code session started in an empty project folder.
> First copy `skills.json` into that folder at `src/data/skills.json`.

---

Build my personal portfolio website as a **scroll-driven side-scrolling platformer**, in the spirit of Robby Leonardi's interactive resume (rleonardi.com).

**How it should work:** the visitor scrolls the page normally, and that vertical scroll moves a character to the right through a game world. Each part of my career is a level. The content (skills, experience, achievements) appears in the world as signs, billboards, items and HUD panels as the character passes them.

**Important:** take inspiration from the *mechanic only*. Do not copy Robby Leonardi's character, artwork, layouts or text. All art must be original.

## About me (the content)

- **Name:** Amey Maldikar
- **Title:** Senior Consultant, Enterprise Architecture Project Lead at Deloitte Consulting (use this exact title, nothing inflated)
- **Location:** Mumbai, India
- **Experience:** 12+ years, Healthcare and Life Sciences
- **Career path:**
  - Software Engineer, Syntel (now Atos), Mar 2014 to Jun 2016: mainframe premium billing system for a US Medicare/Medicaid payer
  - Senior Software Engineer, Capgemini, Jun 2016 to Aug 2017: mainframe back end for a healthcare services provider
  - Senior Consultant, Deloitte Consulting, Aug 2017 to now: enterprise architecture, cloud, DevSecOps, SRE, integration platforms
- **Skills, certifications and achievements:** read all of these from `src/data/skills.json`. Do not hardcode or invent skills.
  - `categories[]`: 10 skill categories, each with `skills[]` holding `skill`, `tier` (core / strong / supporting) and an optional `proof`
  - `certifications[]`
  - `headline_numbers[]`
- **Contact:** leave placeholders (`EMAIL`, `LINKEDIN_URL`, `GITHUB_URL`) for me to fill in.

## Level design

My story is "from mainframe to cloud to AI". Use this level order as a starting point. Suggest changes if a better flow comes up.

1. **Start screen:** title, a short one-line intro, a "Press scroll to start" hint, and a **"Skip to plain resume"** button.
2. **Level 1, The Mainframe Vault:** a green-on-black terminal world. Covers Syntel and Capgemini, and the Mainframe & Legacy and Domain Knowledge categories.
3. **Level 2, Middleware City:** pipes, gateways and message queues. Covers the Integration, Data & Content Platforms category, plus the Layer7 to webMethods migration as a boss-style set piece.
4. **Level 3, The Pipeline Factory:** a conveyor-belt world where items move through build, test, scan and deploy. Covers CI/CD & DevOps and IaC & Containers.
5. **Level 4, Cloud Sky:** the character climbs or flies into the clouds. Covers Cloud & AWS and Security, Identity & Compliance.
6. **Level 5, Control Tower:** dashboards, alarms and an incident to "put out". Covers Observability, SRE & Reliability.
7. **Level 6, AI Frontier:** covers AI-Assisted Engineering.
8. **Final level, HQ:** covers Architecture & Delivery Leadership, certifications as trophies, and contact. End with a "Continue?" screen that leads to the contact links.

Give at least one level a different way of moving (for example floating, climbing or riding a conveyor) so it doesn't feel like one long walk.

## Game UI

- **HUD at the top:** current level name, a progress bar for the whole journey, and a "years of experience" counter that ticks up as you scroll through the timeline.
- **Skills as power-ups:** show skills as collectible items or stat bars. Map `tier` to visual weight: core is large or full bar, strong is medium, supporting is small. Show `proof` in a tooltip or speech bubble when it exists.
- **Achievements:** turn each `headline_numbers` entry into an "Achievement unlocked" toast that pops up once as you pass it.
- **Certifications:** show them as trophies or badges in the final level.

## Tech and build rules

- **Stack:** Vite and TypeScript, with GSAP ScrollTrigger for scroll-driven animation. Build the art in SVG and CSS. No game engine.
- **All text must be real HTML text**, not baked into images, so it is readable by screen readers and searchable.
- **Plain-resume view:** a simple, printable page built from the same `skills.json`. The "Skip" button links to it.
- **Mobile:** must work with touch scrolling at phone width. Simplify parallax layers on small screens if needed.
- **Reduced motion:** if `prefers-reduced-motion` is set, show the plain-resume view or a static version with no auto-animation.
- **Performance:** keep total assets small, lazy-load later levels, and target 60fps on a mid-range phone.
- **Art style:** pick one consistent style (for example clean flat vector or pixel art) and use a small, tight colour palette per level. My original character should be simple enough to animate with a few SVG parts (idle, walk, jump).
- **Output:** a static build in `dist/` that I can host anywhere.

## How to work

1. Start by sharing a short plan: file structure, how the scroll-to-world mapping works, and the art style you chose. Wait for my OK.
2. Build a **vertical slice first**: start screen, Level 1 complete, HUD and the plain-resume page. Run the dev server, take screenshots at desktop and phone width (Playwright), and show me.
3. Once I approve the slice, build the remaining levels one at a time, with screenshots after each.
4. Finish with a check: scroll the whole site at both widths, test reduced motion, confirm every skill in `skills.json` appears somewhere, and run a Lighthouse pass.
