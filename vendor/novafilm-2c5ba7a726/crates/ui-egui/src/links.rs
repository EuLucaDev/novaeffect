//! Community and project links (Help menu, About dialog, header Suporte button, Home screen).
//!
//! Nova Film is part of the Nova Foods family: the community lives on the Nova Foods Suporte, every
//! app has a page on the Nova Foods website and a public GitHub repository.

/// The app's short name, used in the website and repository URLs.
pub const APP: &str = "novafilm";

/// The Nova Foods community Suporte.
pub const DISCORD: &str = "https://workflowind.agcunae.com/inbox";
/// The Nova Foods website.
pub const WEBSITE: &str = "https://workflowind.agcunae.com/apps";
/// Nova Film's page on the Nova Foods website.
pub const APP_PAGE: &str = "https://workflowind.agcunae.com/apps/novafilm";
/// Nova Film's source repository.
pub const GITHUB: &str = "https://github.com/EuLucaDev/novafilm";
/// New issue on Nova Film's repository.
pub const ISSUES: &str = "https://github.com/EuLucaDev/novafilm/issues";

/// (command id, label, url) for every link, in menu order.
pub const ALL: [(&str, &str, &str); 5] = [
    ("help.discord", "Get Nova Foods support…", DISCORD),
    ("help.website", "Nova Foods Website", WEBSITE),
    ("help.appPage", "Nova Film on workflowind.agcunae.com/apps", APP_PAGE),
    ("help.github", "Nova Film on GitHub", GITHUB),
    ("help.reportIssue", "Report an Issue…", ISSUES),
];

/// The URL a `help.*` link command opens.
pub fn url_for(command: &str) -> Option<&'static str> {
    ALL.iter().find(|(id, _, _)| *id == command).map(|(_, _, u)| *u)
}

/// Open `url` in the system browser (a new tab on the web).
pub fn open(ctx: &egui::Context, url: &str) {
    ctx.open_url(egui::OpenUrl::new_tab(url));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_follow_the_novafoods_scheme() {
        assert_eq!(APP_PAGE, format!("{WEBSITE}/apps/{APP}"));
        assert_eq!(GITHUB, format!("https://github.com/EuLucaDev/{APP}"));
        assert!(ALL.iter().all(|(id, _, u)| id.starts_with("help.") && u.starts_with("https://")));
        assert_eq!(url_for("help.discord"), Some(DISCORD));
        assert_eq!(url_for("help.nope"), None);
    }
}
