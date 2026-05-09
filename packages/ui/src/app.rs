use dioxus::prelude::*;
use crate::{DropdownSection, ExploreDropdown, Navbar};
use crate::views::*;

#[derive(Debug, Clone, PartialEq)]
pub enum AuthState {
    Loading,
    Authenticated(api::User),
    Unauthenticated,
}

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppNavbar)]
        #[route("/")]
        Home {},
        #[route("/wallpaper/:id")]
        WallpaperDetail { id: String },
        #[route("/search/:query")]
        Search { query: String },
        #[route("/explore/:tag")]
        Explore { tag: String },
        #[route("/latest")]
        Latest {},
        #[route("/popular")]
        PopularSelection {},
        #[route("/popular/:period")]
        PopularGrid { period: String },
        #[route("/editorial")]
        Editorial {},
        #[route("/ai-generated")]
        AiGenerated {},
        #[route("/live-wallpapers")]
        LiveWallpapers {},
        #[route("/upload")]
        Upload {},
        #[route("/profile")]
        Profile {},
        #[route("/login")]
        Login {},
        #[route("/register")]
        Register {},
        #[route("/settings")]
        Settings {},
        #[route("/collections")]
        Collections {},
        #[route("/collection/:id")]
        CollectionDetail { id: String },
        #[route("/terms")]
        TermsOfService {},
        #[route("/privacy")]
        PrivacyPolicy {},
        #[route("/about")]
        About {},
        #[route("/faq")]
        FAQ {},
        #[route("/contact")]
        ContactUs {},
        #[route("/user/:username")]
        PublicProfile { username: String },
        #[route("/admin")]
        Admin {},
        #[route("/admin/users")]
        AdminUsers {},
}

#[component]
pub fn AppNavbar() -> Element {
    #[allow(unused_mut)]
    let mut show_search = use_context::<Signal<bool>>();
    let mut user = use_context::<Signal<AuthState>>();

    let route = use_route::<Route>();
    let is_home = matches!(route, Route::Home {});

    let _auth_resource = use_resource(move || async move {
        if let Ok(Some(u)) = api::get_current_user().await {
            user.set(AuthState::Authenticated(u));
            if let Ok(ids) = api::get_all_user_favorite_ids().await {
                *crate::FAVORITED_IDS.write() = std::collections::HashSet::from_iter(ids);
            }
        } else {
            user.set(AuthState::Unauthenticated);
            crate::FAVORITED_IDS.write().clear();
        }
    });

    let _scroll_handle = use_hook(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            std::rc::Rc::new(Some(gloo_events::EventListener::new(
                &window,
                "scroll",
                move |_| {
                    #[allow(unused_mut, unused_variables)]
                    let mut show_search = show_search;
                    let window = web_sys::window().unwrap();
                    let scroll_y = window.scroll_y().unwrap_or(0.0);
                    let visible = scroll_y > 400.0;
                    if *show_search.read() != visible {
                        show_search.set(visible);
                    }
                },
            )))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::rc::Rc::new(())
        }
    });

    let display_search = if is_home { show_search() } else { true };

    rsx! {
        div {
            style: "display: flex; flex-direction: column; min-height: 100vh;",
            Navbar {
                home_route: Route::Home {},
                upload_route: Route::Upload {},
                settings_route: Route::Settings {},
                profile_route: Route::Profile {},
                user: match user() {
                    AuthState::Authenticated(u) => Some(u),
                    _ => None,
                },
                show_search: display_search,

                onlogout: move |_| {
                    spawn(async move {
                        let _ = api::logout().await;
                        user.set(AuthState::Unauthenticated);
                        crate::FAVORITED_IDS.write().clear();
                    });
                },

                login_action: rsx! {
                    Link {
                        to: Route::Login {},
                        style: "color: var(--text-secondary); text-decoration: none; font-size: 14px; font-weight: 600;",
                        "Login"
                    }
                },

                ExploreDropdown {
                    sections: rsx! {
                        DropdownSection {
                            title: "Trending Categories",
                            div {
                                class: "explore-categories-list",
                                style: "display: grid; grid-template-columns: 1fr 1fr; gap: 8px;",
                                for (val, label) in api::tags::CATEGORIES.iter().take(6) {
                                    Link {
                                        key: "{val}",
                                        to: Route::Explore { tag: val.to_string() },
                                        class: "menu-item-hover",
                                        style: "padding: 10px 12px; border-radius: 10px; background: rgba(255,255,255,0.02); border: 1px solid rgba(255,255,255,0.05); text-decoration: none; color: white; font-size: 14px; font-weight: 600; transition: all 0.2s;",
                                        "{label}"
                                    }
                                }
                            }
                            Link {
                                to: Route::Explore { tag: "all".into() },
                                class: "glow-hover",
                                style: "margin-top: 8px; display: inline-flex; align-items: center; gap: 6px; color: var(--accent-primary); font-size: 13px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; text-decoration: none; padding: 4px 0; transition: opacity 0.2s;",
                                "Browse All Categories"
                                span { style: "font-size: 16px;", "→" }
                            }
                        }
                        DropdownSection {
                            title: "Discover",
                            separator: true,
                            Link { to: Route::Latest {}, "Latest wallpapers" }
                            Link { to: Route::PopularSelection {}, "Popular wallpapers" }
                            Link { to: Route::Editorial {}, "Editorial selections" }
                            Link { to: Route::AiGenerated {}, "AI Generated" }
                            Link { to: Route::LiveWallpapers {}, "Live wallpapers" }
                        }
                        DropdownSection {
                            title: "Info",
                            separator: true,
                            Link { to: Route::About {}, "About" }
                            Link { to: Route::FAQ {}, "FAQ" }
                            Link { to: Route::ContactUs {}, "Contact us" }
                            Link { to: Route::TermsOfService {}, "Terms & Conditions" }
                            Link { to: Route::PrivacyPolicy {}, "Privacy Policy" }
                            if let AuthState::Authenticated(u) = user() {
                                if u.role == "admin" || u.role == "super_admin" || u.role == "moderator" {
                                    Link { to: Route::Admin {}, "Admin Dashboard" }
                                }
                            }
                        }
                    }
                }
            }
            main { style: "flex: 1;", Outlet::<Route> {} }
            crate::Footer {
                home_route: Route::Home {},
                terms_route: Route::TermsOfService {},
                privacy_route: Route::PrivacyPolicy {},
                about_route: Route::About {},
                faq_route: Route::FAQ {},
            }
        }
    }
}
