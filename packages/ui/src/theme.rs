use dioxus::prelude::*;

const THEME_CSS: Asset = asset!("/assets/styling/theme.css");

#[component]
pub fn Theme() -> Element {
    let script = r#"
        function applyTheme() {
            let theme = localStorage.getItem('settings_theme') || 'dark';
            if (theme === 'System Default' || theme === 'system') {
                theme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
            }
            if (theme === 'Light Mode' || theme === 'light') {
                document.documentElement.setAttribute('data-theme', 'light');
            } else if (theme === 'OLED Black' || theme === 'oled') {
                document.documentElement.setAttribute('data-theme', 'oled');
            } else {
                document.documentElement.setAttribute('data-theme', 'dark');
            }
        }
        applyTheme();
        window.addEventListener('storage', (e) => {
            if (e.key === 'settings_theme') applyTheme();
        });
        window.addEventListener('local-storage-update', () => applyTheme());
        window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
            let t = localStorage.getItem('settings_theme');
            if (t === 'System Default' || t === 'system') applyTheme();
        });
    "#;

    rsx! {
        document::Link { rel: "stylesheet", href: THEME_CSS }
        document::Script { "{script}" }
    }
}
