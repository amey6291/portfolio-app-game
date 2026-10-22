//! Which content lives in which level. Every category and headline number
//! must be assigned exactly once; `check` enforces that at build time.

use crate::data::Skills;

pub struct LevelDef {
    pub id: &'static str,
    pub categories: &'static [&'static str],
    /// Indices into `skills.json` `headline_numbers`.
    pub achievements: &'static [usize],
}

pub const LEVELS: &[LevelDef] = &[
    LevelDef {
        id: "level-1",
        categories: &["Mainframe & Legacy", "Domain Knowledge"],
        achievements: &[],
    },
    LevelDef {
        id: "level-2",
        categories: &["Integration, Data & Content Platforms"],
        achievements: &[2, 3, 4],
    },
    LevelDef {
        id: "level-3",
        categories: &["CI/CD & DevOps", "Infrastructure as Code & Containers"],
        achievements: &[0, 1],
    },
    LevelDef {
        id: "level-4",
        categories: &["Cloud & AWS", "Security, Identity & Compliance"],
        achievements: &[6],
    },
    LevelDef {
        id: "level-5",
        categories: &["Observability, SRE & Reliability"],
        achievements: &[],
    },
    LevelDef {
        id: "level-6",
        categories: &["AI-Assisted Engineering"],
        achievements: &[],
    },
    LevelDef {
        id: "hq",
        categories: &["Architecture & Delivery Leadership"],
        achievements: &[5],
    },
];

/// Panics if any category or headline number is unassigned or assigned twice.
pub fn check(skills: &Skills) {
    for c in &skills.categories {
        let n = LEVELS
            .iter()
            .filter(|l| l.categories.contains(&c.category.as_str()))
            .count();
        assert_eq!(n, 1, "category {:?} is in {n} levels", c.category);
    }
    for i in 0..skills.headline_numbers.len() {
        let n = LEVELS.iter().filter(|l| l.achievements.contains(&i)).count();
        assert_eq!(n, 1, "headline_numbers[{i}] is in {n} levels");
    }
    for l in LEVELS {
        for c in l.categories {
            skills.category(c);
        }
    }
}
