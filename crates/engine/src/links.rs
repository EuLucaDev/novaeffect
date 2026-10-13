//! Community and project links (shown in Help ▸, the About dialog, the start screen and the
//! header's Suporte button).

pub const APP_NAME: &str = "Nova Effect";
pub const APP_SLUG: &str = "novaeffect";
pub const DISCORD: &str = "https://workflowind.agcunae.com/inbox";
pub const WEBSITE: &str = "https://workflowind.agcunae.com/apps";
pub const APP_PAGE: &str = "https://workflowind.agcunae.com/apps/novaeffect";
pub const GITHUB: &str = "https://github.com/EuLucaDev/novaeffect";
pub const ISSUES: &str = "https://github.com/EuLucaDev/novaeffect/issues";

/// Sibling apps of the Nova Foods family: (name, what it is, GitHub repo, app page).
pub const SIBLINGS: &[(&str, &str)] = &[
    ("Nova Photo", "novaphoto"),
    ("Nova Vector", "novavector"),
    ("Nova Film", "novafilm"),
    ("Nova Light", "novalight"),
    ("Nova PDF", "novapdf"),
    ("Nova Design", "novadesign"),
];

pub fn github_of(slug: &str) -> String {
    format!("https://github.com/EuLucaDev/{slug}")
}
pub fn page_of(slug: &str) -> String {
    format!("https://workflowind.agcunae.com/apps/{slug}")
}
