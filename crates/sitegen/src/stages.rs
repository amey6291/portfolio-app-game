//! The levels, in journey order. Content assignment lives in `levels.rs`;
//! this file is the set dressing around it.

use maud::{html, Markup, PreEscaped};

use crate::game::{achievement, job_sign, level_card, skill_cluster, Ctx};
use crate::levels::LEVELS;

pub fn all(cx: &Ctx) -> Markup {
    html! {
        @for l in LEVELS { (stage(cx, l.id)) }
        (continue_screen(cx))
    }
}

fn stage(cx: &Ctx, id: &str) -> Markup {
    match id {
        "level-1" => level1(cx),
        "level-2" => level2(cx),
        "level-3" => level3(cx),
        "level-4" => level4(cx),
        "level-5" => level5(cx),
        "level-6" => level6(cx),
        "hq" => hq(cx),
        other => panic!("no stage for level {other:?}"),
    }
}

fn level1(cx: &Ctx) -> Markup {
    let jobs = &cx.profile.jobs;
    let last = jobs[1].end.as_deref().expect("Capgemini end date");
    html! {
        section.level.lv1 #level-1 data-name="1 · Mainframe Vault" aria-label="Level 1: The Mainframe Vault" {
            div.bg aria-hidden="true" {
                div.layer.racks data-depth="0.6" {}
                div.layer.cables.mobile-off data-depth="0.3" {}
                div.scanlines {}
            }
            div.stage {
                (level_card("Level 1", "The Mainframe Vault", "2014 – 2017"))
                (job_sign(cx, 1, &jobs[0]))
                (skill_cluster("Skill cache", cx.skills.category("Mainframe & Legacy")))
                div.beat.obstacle data-jump aria-hidden="true" { (tape_reels()) }
                (job_sign(cx, 2, &jobs[1]))
                (skill_cluster("Domain data", cx.skills.category("Domain Knowledge")))
                div.beat.exit data-reveal data-year=(cx.years_at(last))
                    data-toast-title="Level clear" data-toast="The Mainframe Vault is complete." {
                    div.door { span { "EXIT" } }
                    p { "Next: Middleware City" }
                }
            }
            div.ground aria-hidden="true" {}
        }
    }
}

/// Nodes in the Layer7 to webMethods migration (headline_numbers[2]).
const BOSS_NODES: usize = 10;

fn level2(cx: &Ctx) -> Markup {
    let deloitte = &cx.profile.jobs[2];
    html! {
        section.level.lv2 #level-2 data-name="2 · Middleware City" aria-label="Level 2: Middleware City" {
            div.bg aria-hidden="true" {
                div.layer.towers data-depth="0.6" {}
                div.layer.pipes.mobile-off data-depth="0.3" {}
            }
            div.stage {
                (level_card("Level 2", "Middleware City", "Pipes, gateways and message queues"))
                (job_sign(cx, 3, deloitte))
                (skill_cluster("Message queue", cx.skills.category("Integration, Data & Content Platforms")))
                (achievement(cx, 3))
                div.beat.obstacle data-jump aria-hidden="true" { (valve()) }
                (achievement(cx, 4))
                section.beat.boss data-progress aria-label="Boss: Layer7 to webMethods migration" {
                    p.k { "Boss fight" }
                    h3 { "The Legacy Gateway" }
                    p.vs { span.old { "Broadcom Layer7" } " → " span.new { "Software AG webMethods" } }
                    div.boss-art aria-hidden="true" { (boss_gate()) }
                    div.hp aria-hidden="true" {
                        span { "Layer7 nodes" }
                        span.hp-bar { i {} }
                    }
                    ol.nodes aria-hidden="true" {
                        @for i in 0..BOSS_NODES {
                            li style=(format!("--i:{i}")) { b.o { "L7" } b.n { "wM" } }
                        }
                    }
                    p.result { (cx.headline(2)) }
                    span.toast-mark data-toast=(cx.headline(2)) {}
                }
            }
            div.ground aria-hidden="true" {}
        }
    }
}

fn level3(cx: &Ctx) -> Markup {
    html! {
        section.level.lv3 #level-3 data-name="3 · Pipeline Factory" data-hero="ride"
            aria-label="Level 3: The Pipeline Factory" {
            div.bg aria-hidden="true" {
                div.layer.windows data-depth="0.6" {}
                // Belt surface moves with the hero: depth 1 keeps it still on screen.
                div.layer.belt data-depth="1" {}
                template data-decor { (gears()) }
            }
            div.stage {
                (level_card("Level 3", "The Pipeline Factory", "Build · test · scan · deploy"))
                (machine("Build", "Compile and package"))
                (skill_cluster("Line 1", cx.skills.category("CI/CD & DevOps")))
                (machine("Test", "Automated test gates"))
                (achievement(cx, 1))
                (machine("Scan", "Quality and security scans"))
                (skill_cluster("Line 2", cx.skills.category("Infrastructure as Code & Containers")))
                (achievement(cx, 0))
                div.beat.deploy data-toast-title="Pipeline green" data-toast="Build, test, scan, deploy: every stage passed." {
                    (machine("Deploy", "Ship to production"))
                }
            }
            div.ground aria-hidden="true" {}
        }
    }
}

/// A pipeline station whose lamp and meter go green as the hero arrives.
fn machine(stage: &str, what: &str) -> Markup {
    html! {
        div.beat.machine data-progress data-reveal {
            div.housing {
                p.stage-name { (stage) }
                p.what { (what) }
                span.meter aria-hidden="true" { i {} }
                p.status aria-hidden="true" { b.run { "Running" } b.ok { "Passed" } }
                span.lamp aria-hidden="true" {}
            }
            div.chute aria-hidden="true" {}
        }
    }
}

fn trophy() -> Markup {
    PreEscaped(r##"<svg viewBox="0 0 48 56" width="48" height="56"><path d="M12 4h24v14a12 12 0 0 1-24 0Z" fill="#e0a526"/><path d="M12 8H5v4a8 8 0 0 0 8 8M36 8h7v4a8 8 0 0 1-8 8" fill="none" stroke="#e0a526" stroke-width="3"/><rect x="21" y="29" width="6" height="12" fill="#c48a17"/><rect x="13" y="41" width="22" height="8" rx="2" fill="#2f3e5c"/><path d="M17 8v9" stroke="#fff3c4" stroke-width="3" stroke-linecap="round" opacity=".7"/></svg>"##.to_string())
}

fn gears() -> Markup {
    let gear = |cls: &str, r: u32| {
        let teeth: String = (0..12)
            .map(|i| format!(r#"<rect x="-5" y="{}" width="10" height="14" rx="2" transform="rotate({})"/>"#, -(r as i32) - 10, i * 30))
            .collect();
        format!(r##"<svg class="gear {cls}" viewBox="{v0} {v0} {vs} {vs}" width="{vs}" height="{vs}"><g fill="currentColor">{teeth}<circle r="{r}"/></g><circle r="{h}" fill="#2b2f3a"/></svg>"##,
            v0 = -(r as i32) - 12, vs = 2 * r + 24, h = r / 3)
    };
    PreEscaped(format!("{}{}{}", gear("g1", 70), gear("g2", 44), gear("g3", 90)))
}

fn level4(cx: &Ctx) -> Markup {
    html! {
        section.level.lv4 #level-4 data-name="4 · Cloud Sky" data-hero="float" aria-label="Level 4: Cloud Sky" {
            div.bg aria-hidden="true" {
                div.layer.clouds-far data-depth="0.75" {}
                div.layer.clouds-near.mobile-off data-depth="0.4" {}
                template data-decor { div.sun {} }
            }
            div.stage {
                (level_card("Level 4", "Cloud Sky", "Up into the cloud"))
                (skill_cluster("Sky cache", cx.skills.category("Cloud & AWS")))
                (achievement(cx, 6))
                (skill_cluster("Shield bay", cx.skills.category("Security, Identity & Compliance")))
                div.beat.landing data-reveal { p { "Landing zone" } }
            }
            div.ground aria-hidden="true" {}
        }
    }
}

fn level5(cx: &Ctx) -> Markup {
    const STEPS: [&str; 4] = ["Detect", "Triage", "Mitigate", "Root cause"];
    html! {
        section.level.lv5 #level-5 data-name="5 · Control Tower" aria-label="Level 5: Control Tower" {
            div.bg aria-hidden="true" {
                div.layer.monitors data-depth="0.55" {}
                template data-decor { div.tower { span.beacon {} } div.radar {} }
            }
            div.stage {
                (level_card("Level 5", "Control Tower", "Watch, alert, respond"))
                (skill_cluster("Instrument panel", cx.skills.category("Observability, SRE & Reliability")))
                section.beat.incident data-progress data-toast-title="Alarm cleared"
                    data-toast="Incident resolved and the root cause written up." aria-label="Incident drill" {
                    p.k { "Incident drill" }
                    p.state aria-hidden="true" { b.bad { span { "Alerting" } } b.good { "Resolved" } }
                    div.graph aria-hidden="true" {
                        svg viewBox="0 0 300 100" preserveAspectRatio="none" {
                            line.slo x1="0" y1="40" x2="300" y2="40" {}
                            polyline.spike points="0,80 40,78 80,76 110,30 130,12 160,18 190,70 230,78 300,80" {}
                            polyline.calm points="0,80 40,78 80,76 110,74 130,76 160,75 190,78 230,77 300,79" {}
                        }
                        span.slo-label { "SLO" }
                    }
                    ol.steps {
                        @for (i, step) in STEPS.iter().enumerate() {
                            li style=(format!("--i:{i}")) { (step) }
                        }
                    }
                }
            }
            div.ground aria-hidden="true" {}
        }
    }
}

fn level6(cx: &Ctx) -> Markup {
    // The loop the "MCP servers" proof describes.
    const LOOP: [&str; 4] = ["MCP servers", "AI agents", "Flag scan findings", "Enforce unit tests"];
    html! {
        section.level.lv6 #level-6 data-name="6 · AI Frontier" data-hero="drone" aria-label="Level 6: AI Frontier" {
            div.bg aria-hidden="true" {
                div.layer.net data-depth="0.6" {}
                div.horizon {}
            }
            div.stage {
                (level_card("Level 6", "AI Frontier", "Engineering with AI on the team"))
                div.beat.companion data-reveal data-toast-title="Companion unlocked"
                    data-toast="A helper drone joins the run." {
                    p.k { "Companion unlocked" }
                    p { "Helper drone online" }
                }
                (skill_cluster("Neural cache", cx.skills.category("AI-Assisted Engineering")))
                section.beat.agent data-progress aria-label="Agent loop" {
                    p.k { "Agent loop" }
                    ol.loop {
                        @for (i, step) in LOOP.iter().enumerate() {
                            li style=(format!("--i:{i}")) { (step) }
                        }
                    }
                }
            }
            div.ground aria-hidden="true" {}
        }
    }
}

fn hq(cx: &Ctx) -> Markup {
    html! {
        section.level.hq #hq data-name="Final · HQ" data-hero="drone" aria-label="Final level: HQ" {
            div.bg aria-hidden="true" {
                div.layer.glass data-depth="0.5" {}
                template data-decor { div.hq-tower { span { "HQ" } } }
            }
            div.stage {
                (level_card("Final level", "HQ", "Architecture and delivery leadership"))
                (skill_cluster("Blueprint archive", cx.skills.category("Architecture & Delivery Leadership")))
                (achievement(cx, 5))
                section.beat.trophies data-reveal aria-label="Certifications" {
                    h3 { span.tag { "Trophy room" } "Certifications" }
                    ul {
                        @for c in &cx.skills.certifications {
                            li { span.cup aria-hidden="true" { (trophy()) } span.cert { (c.name) } }
                        }
                    }
                    div.shelf aria-hidden="true" {}
                }
            }
            div.ground aria-hidden="true" {}
        }
    }
}

fn continue_screen(cx: &Ctx) -> Markup {
    let c = &cx.profile.contact;
    html! {
        section.level.continue #continue data-name="Continue?" data-hero="drone"
            data-year=(cx.years_now()) aria-label="Continue? Contact" {
            div.bg aria-hidden="true" { div.scanlines {} }
            div.stage {
                div.beat.continue-card data-reveal {
                    h2 { "Continue?" span.count data-countdown aria-hidden="true" { "9" } }
                    p.lede { "Thanks for playing. Press a button to start a conversation." }
                    ul.contact-links {
                        li { a.btn href=(format!("mailto:{}", c.email)) { "Email" } }
                        li { a.btn href=(c.linkedin) { "LinkedIn" } }
                    }
                    p.alt { "Prefer paper? Read the " a href="resume/" { "plain resume" } "." }
                }
            }
            div.ground aria-hidden="true" {}
        }
    }
}

fn tape_reels() -> Markup {
    PreEscaped(r##"<svg viewBox="0 0 120 80" width="120" height="80"><rect x="2" y="30" width="116" height="48" rx="4" fill="#123222" stroke="#39ff7a" stroke-width="2"/><circle cx="34" cy="54" r="17" fill="#04120a" stroke="#39ff7a" stroke-width="2"/><circle cx="86" cy="54" r="17" fill="#04120a" stroke="#39ff7a" stroke-width="2"/><circle cx="34" cy="54" r="5" fill="#39ff7a"/><circle cx="86" cy="54" r="5" fill="#39ff7a"/><rect x="18" y="6" width="84" height="24" rx="3" fill="#0b2015" stroke="#1d7a3f" stroke-width="2"/><rect x="26" y="13" width="40" height="4" fill="#1d7a3f"/><rect x="26" y="20" width="24" height="4" fill="#1d7a3f"/></svg>"##.to_string())
}

fn valve() -> Markup {
    PreEscaped(r##"<svg viewBox="0 0 130 90" width="130" height="90"><rect x="0" y="52" width="130" height="26" rx="4" fill="#b8733a"/><rect x="0" y="56" width="130" height="5" fill="#e0a06a" opacity=".6"/><rect x="40" y="44" width="50" height="42" rx="6" fill="#8a5428"/><rect x="61" y="18" width="8" height="28" fill="#5b3a22"/><circle cx="65" cy="16" r="15" fill="none" stroke="#36d6c3" stroke-width="6"/><path d="M50 16 H80 M65 1 V31" stroke="#36d6c3" stroke-width="4"/></svg>"##.to_string())
}

/// The boss: a fortified gateway whose angry red face gives way to a calm
/// teal one as `--p` rises.
fn boss_gate() -> Markup {
    PreEscaped(r##"<svg viewBox="0 0 220 170" width="220" height="170">
<g class="o"><path d="M10 170 V60 L30 40 V20 H60 V40 H90 V10 H130 V40 H160 V20 H190 V40 L210 60 V170Z" fill="#5a1f2e" stroke="#ff5a6e" stroke-width="4"/><path d="M70 170 V115 A40 40 0 0 1 150 115 V170Z" fill="#1c0b12"/><path d="M78 125 L92 140 L106 125 L120 140 L134 125 L142 140" fill="none" stroke="#ff5a6e" stroke-width="4"/><path d="M60 72 L95 84 M160 72 L125 84" stroke="#ff5a6e" stroke-width="6" stroke-linecap="round"/><circle cx="88" cy="92" r="7" fill="#ffcf5a"/><circle cx="132" cy="92" r="7" fill="#ffcf5a"/></g>
<g class="n"><path d="M10 170 V60 L30 40 V20 H60 V40 H90 V10 H130 V40 H160 V20 H190 V40 L210 60 V170Z" fill="#163f4a" stroke="#36d6c3" stroke-width="4"/><path d="M70 170 V115 A40 40 0 0 1 150 115 V170Z" fill="#0b2228"/><circle cx="88" cy="88" r="7" fill="#e8fffb"/><circle cx="132" cy="88" r="7" fill="#e8fffb"/><path d="M92 104 Q110 118 128 104" fill="none" stroke="#e8fffb" stroke-width="4" stroke-linecap="round"/></g>
</svg>"##.to_string())
}
