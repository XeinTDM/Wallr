pub mod dashboard;
pub mod dmca;
pub mod reports;
pub mod users;
pub mod appeals;
pub mod editorial;

use crate::app::{AuthState, Route};
use dioxus::prelude::*;
use lucide_dioxus::{Activity, ShieldAlert, Flag, Users, Scale, Library};

const ADMIN_CSS: Asset = asset!("/assets/styling/settings.css"); // Reusing settings CSS for identical sidebar layout

#[component]
pub fn AdminLayout() -> Element {
    let auth_state = use_context::<Signal<AuthState>>();
    let nav = use_navigator();
    let _route = use_route::<Route>();

    let is_allowed = match auth_state() {
        AuthState::Authenticated(u) => {
            u.role == "admin" || u.role == "super_admin" || u.role == "moderator"
        }
        AuthState::Loading => return rsx! { crate::LoadingScreen {} },
        AuthState::Unauthenticated => false,
    };

    if !is_allowed {
        nav.push(Route::Home {});
        return rsx! {};
    }

    rsx! {
        document::Stylesheet { href: ADMIN_CSS }
        div {
            class: "settings-page",
            style: "padding: 100px 32px 80px; max-width: 1200px; margin: 0 auto; display: flex; gap: 40px; min-height: calc(100vh - 80px);",

            // Sidebar
            div {
                class: "settings-sidebar",
                style: "width: 280px; flex-shrink: 0; display: flex; flex-direction: column; gap: 12px;",
                div {
                    class: "section-title",
                    "Admin Portal"
                }
                Link {
                    to: Route::AdminDashboard {},
                    class: "settings-nav-item",
                    active_class: "active",
                    Activity { size: 18 }
                    "Dashboard"
                }
                Link {
                    to: Route::AdminUsers {},
                    class: "settings-nav-item",
                    active_class: "active",
                    Users { size: 18 }
                    "Users"
                }
                Link {
                    to: Route::AdminReports {},
                    class: "settings-nav-item",
                    active_class: "active",
                    Flag { size: 18 }
                    "Reports"
                }
                Link {
                    to: Route::AdminAppeals {},
                    class: "settings-nav-item",
                    active_class: "active",
                    Scale { size: 18 }
                    "Appeals"
                }
                Link {
                    to: Route::AdminDmca {},
                    class: "settings-nav-item",
                    active_class: "active",
                    ShieldAlert { size: 18 }
                    "DMCA Claims"
                }
                Link {
                    to: Route::AdminEditorial {},
                    class: "settings-nav-item",
                    active_class: "active",
                    Library { size: 18 }
                    "Editorial"
                }
            }

            // Main Content Area
            div {
                class: "settings-content",
                style: "flex: 1; max-width: 800px;",
                Outlet::<Route> {}
            }
        }
    }
}
