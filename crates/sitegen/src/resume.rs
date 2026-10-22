//! The plain, printable resume (`dist/resume/index.html`).

use maud::{html, Markup, PreEscaped, DOCTYPE};

use crate::data::{Profile, Skills, Tier};

pub fn page(skills: &Skills, p: &Profile) -> Markup {
    let c = &p.contact;
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (p.name) " · Resume" }
                meta name="description" content=(format!("Resume of {}, {}.", p.name, p.title));
                link rel="icon" href=(crate::FAVICON);
                link rel="stylesheet" href="resume.css";
            }
            body {
                nav.actions aria-label="Page actions" {
                    a href="../" { "← Play the interactive version" }
                    button type="button" onclick="window.print()" { "Print / save as PDF" }
                }
                main {
                    header {
                        h1 { (p.name) }
                        p.title { (p.title) }
                        p.meta { (p.location) " · " (p.experience) " · " (p.domain) }
                        ul.contact {
                            li { a href=(format!("mailto:{}", c.email)) { (c.email) } }
                            li { a href=(c.linkedin) { "LinkedIn" } }
                        }
                    }
                    section {
                        h2 { "Summary" }
                        p { (p.intro) }
                    }
                    section {
                        h2 { "Highlights" }
                        ul { @for h in &skills.headline_numbers { li { (h) } } }
                    }
                    section {
                        h2 { "Experience" }
                        @for j in p.jobs.iter().rev() {
                            article.job {
                                h3 { (j.role) ", " (j.company) }
                                p.dates { (j.dates()) }
                                p { (j.summary) }
                            }
                        }
                    }
                    section {
                        h2 { "Skills" }
                        p.legend { "Core skills are in bold." }
                        @for cat in &skills.categories {
                            div.cat {
                                h3 { (cat.category) }
                                ul.skills {
                                    @for s in &cat.skills {
                                        li class=(s.tier.class()) {
                                            @if s.tier == Tier::Core { strong { (s.skill) } } @else { (s.skill) }
                                            @if let Some(proof) = &s.proof {
                                                span.proof { (PreEscaped(" — ")) (proof) }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    section {
                        h2 { "Certifications" }
                        ul { @for cert in &skills.certifications { li { (cert.name) } } }
                    }
                }
            }
        }
    }
}
