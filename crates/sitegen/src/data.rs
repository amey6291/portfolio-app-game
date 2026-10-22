//! Typed views of `src/data/skills.json` and `src/data/profile.json`.

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Skills {
    pub categories: Vec<Category>,
    pub certifications: Vec<Certification>,
    pub headline_numbers: Vec<String>,
}

#[derive(Deserialize)]
pub struct Category {
    pub category: String,
    pub skills: Vec<Skill>,
}

#[derive(Deserialize)]
pub struct Skill {
    pub skill: String,
    pub tier: Tier,
    pub proof: Option<String>,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Core,
    Strong,
    Supporting,
}

impl Tier {
    pub fn class(self) -> &'static str {
        match self {
            Tier::Core => "t-core",
            Tier::Strong => "t-strong",
            Tier::Supporting => "t-supporting",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Tier::Core => "Core",
            Tier::Strong => "Strong",
            Tier::Supporting => "Supporting",
        }
    }

    /// Stat-bar fill, in percent.
    pub fn power(self) -> u8 {
        match self {
            Tier::Core => 100,
            Tier::Strong => 66,
            Tier::Supporting => 33,
        }
    }
}

#[derive(Deserialize)]
pub struct Certification {
    pub name: String,
}

#[derive(Deserialize)]
pub struct Profile {
    pub name: String,
    pub title: String,
    pub location: String,
    pub experience: String,
    pub domain: String,
    pub intro: String,
    pub career_start: String,
    pub jobs: Vec<Job>,
    pub contact: Contact,
}

#[derive(Deserialize)]
pub struct Job {
    pub role: String,
    pub company: String,
    pub start: String,
    pub end: Option<String>,
    pub summary: String,
}

#[derive(Deserialize)]
pub struct Contact {
    pub email: String,
    pub linkedin: String,
}

impl Skills {
    pub fn category(&self, name: &str) -> &Category {
        self.categories
            .iter()
            .find(|c| c.category == name)
            .unwrap_or_else(|| panic!("category {name:?} missing from skills.json"))
    }
}

/// Months since 0 AD for a "YYYY-MM" string.
pub fn month_index(ym: &str) -> i32 {
    let (y, m) = ym.split_once('-').expect("dates must be YYYY-MM");
    y.parse::<i32>().unwrap() * 12 + m.parse::<i32>().unwrap() - 1
}

/// This month's index, from the system clock.
pub fn current_month_index() -> i32 {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        / 86_400;
    // Civil-from-days (Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y * 12 + m - 1) as i32
}

pub fn month_label(ym: &str) -> String {
    const NAMES: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let i = month_index(ym);
    format!("{} {}", NAMES[(i % 12) as usize], i / 12)
}

impl Job {
    pub fn dates(&self) -> String {
        let end = self.end.as_deref().map_or("Present".to_string(), month_label);
        format!("{} – {}", month_label(&self.start), end)
    }
}
