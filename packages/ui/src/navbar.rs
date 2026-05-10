use dioxus::prelude::*;
use lucide_dioxus::{Bell, ChevronDown, LogOut, Settings, User, X};

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");
const LOGO_PNG: Asset = asset!("/assets/logo.png");

#[derive(Props, Clone, PartialEq)]
pub struct NavbarProps<R: Routable + Clone + PartialEq + 'static> {
    pub children: Element,
    #[props(default = true)]
    pub show_search: bool,

    pub home_route: R,

    #[props(optional)]
    pub upload_route: Option<R>,

    #[props(optional)]
    pub settings_route: Option<R>,

    /*
    #[props(into, default)]
    pub onsearch: EventHandler<String>,
    */
    #[props(optional)]
    pub login_action: Option<Element>,
    #[props(optional)]
    pub profile_route: Option<R>,
    #[props(optional)]
    pub user: Option<api::User>,
    #[props(optional)]
    pub onlogout: Option<EventHandler<()>>,
}

#[component]
pub fn Navbar<R: Routable + Clone + PartialEq + 'static>(props: NavbarProps<R>) -> Element {
    let mut search_query = use_signal(String::new);
    let mut user_menu_open = use_signal(|| false);
    let mut notif_menu_open = use_signal(|| false);
    let nav = use_navigator();
    let i18n = crate::i18n::use_i18n();

    let has_user = props.user.is_some();
    let mut notifications_res = use_resource(move || async move {
        if has_user {
            api::get_my_notifications().await.unwrap_or_default()
        } else {
            vec![]
        }
    });

    use_effect(move || {
        if has_user {
            spawn(async move {
                loop {
                    #[cfg(target_arch = "wasm32")]
                    gloo_timers::future::sleep(std::time::Duration::from_secs(15)).await;

                    #[cfg(not(target_arch = "wasm32"))]
                    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

                    notifications_res.restart();
                }
            });
        }
    });

    let suggestions = use_resource(move || {
        let q = search_query();
        async move {
            if q.is_empty() {
                return vec![];
            }
            api::search_users_endpoint(q, 5).await.unwrap_or_default()
        }
    });

    let _shortcuts = use_hook(move || {
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_events::EventListener;
            use web_sys::wasm_bindgen::JsCast;

            let window = web_sys::window().unwrap();

            let listener = EventListener::new(&window, "keydown", move |event| {
                let e: &web_sys::KeyboardEvent = event.unchecked_ref();

                if e.ctrl_key() || e.meta_key() || e.alt_key() {
                    return;
                }

                let tag = e
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                    .map(|el| el.tag_name().to_lowercase());
                if let Some(t) = tag {
                    if t == "input" || t == "textarea" {
                        return;
                    }
                }

                let key_str: String = e.key().into();
                let key_str = key_str.to_lowercase();
                if key_str == "/" || key_str == "s" {
                    event.prevent_default();
                    if let Some(input) = web_sys::window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .get_element_by_id("nav-search-input")
                    {
                        let _ = input.dyn_ref::<web_sys::HtmlElement>().unwrap().focus();
                    }
                }
            });
            Some(std::rc::Rc::new(listener))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            None::<()>
        }
    });

    rsx! {
        document::Stylesheet { href: NAVBAR_CSS }
        nav {
            id: "navbar",
            class: "glass nav-blur",
            style: "position: fixed; top: 0; left: 0; right: 0; height: var(--nav-height); padding: 0.5rem 0; z-index: 1000; display: flex; align-items: center; border-bottom: 1px solid var(--border-color);",

            div {
                class: "nav-content",
                style: "display: flex; align-items: center; justify-content: space-between; width: 100%; height: 100%; padding: 0 40px;",

                div {
                    class: "nav-left",
                    style: "flex: 1; display: flex; align-items: center;",
                    Link {
                        class: "logo",
                        to: props.home_route.clone(),
                        style: "display: flex; align-items: center; text-decoration: none; padding: 4px 0; font-size: 24px; font-weight: 900; letter-spacing: -0.04em; color: white;",
                        img {
                            src: LOGO_PNG,
                            alt: "Wallr Logo",
                            style: "height: 52px; opacity: 0.9;"
                        }
                    }
                }

                div {
                    class: "nav-center",
                    style: "flex: 0; display: flex; justify-content: center; min-width: 400px; margin: 0 40px; position: relative;",
                    if props.show_search {
                        div {
                            class: "nav-search-container fade-in",
                            style: "display: flex; align-items: center; width: 100%; position: relative; transition: all var(--transition-smooth);",
                            input {
                                id: "nav-search-input",
                                r#type: "text",
                                class: "glass search-input",
                                style: "width: 100%; padding: 12px 20px 12px 48px; border-radius: 14px; border: 1px solid rgba(255,255,255,0.1); background: rgba(255,255,255,0.05); color: white; outline: none;",
                                placeholder: i18n.t("search_placeholder"),
                                value: "{search_query}",
                                oninput: move |e| search_query.set(e.value()),
                                onkeydown: move |e| {
                                    if e.key() == Key::Enter {
                                        let q = search_query();
                                        if !q.trim().is_empty() {
                                            nav.push(format!("/search/{}", q.trim()));
                                        }
                                    }
                                }
                            }
                            span {
                                style: "position: absolute; left: 16px; top: 50%; transform: translateY(-50%); opacity: 0.5; display: flex; align-items: center;",
                                svg {
                                    width: "18",
                                    height: "18",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2.5",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    path { d: "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" }
                                }
                            }
                            if !search_query().is_empty() {
                                button {
                                    style: "position: absolute; right: 16px; top: 50%; transform: translateY(-50%); opacity: 0.5; display: flex; align-items: center; background: none; border: none; color: white; cursor: pointer; padding: 4px; border-radius: 50%; transition: opacity 0.2s, background 0.2s;",
                                    class: "search-clear-btn glow-hover",
                                    onclick: move |_| {
                                        search_query.set(String::new());
                                    },
                                    X { size: 16 }
                                }
                            }

                            if !search_query().is_empty() && suggestions.read().as_ref().map(|s| !s.is_empty()).unwrap_or(false) {
                                div {
                                    class: "glass",
                                    style: "position: absolute; top: calc(100% + 8px); left: 0; right: 0; border-radius: 12px; padding: 8px; display: flex; flex-direction: column; gap: 4px; z-index: 1000; border: 1px solid rgba(255,255,255,0.1);",
                                    for user in suggestions.read().as_ref().unwrap_or(&vec![]).iter() {
                                        a {
                                            href: "/user/{user.name.replace(\" \", \"-\")}",
                                            style: "display: flex; align-items: center; gap: 12px; padding: 8px 12px; border-radius: 8px; text-decoration: none; color: white; transition: background 0.2s;",
                                            class: "menu-item-hover",
                                            onclick: move |_| search_query.set(String::new()),
                                            img {
                                                src: "{crate::resolve_asset_url(&user.pfp_url)}",
                                                style: "width: 28px; height: 28px; border-radius: 50%; object-fit: cover; border: 1px solid rgba(255,255,255,0.1);"
                                            }
                                            span { style: "font-size: 14px; font-weight: 600;", "{user.name}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div {
                    class: "nav-right",
                    style: "flex: 1; display: flex; align-items: center; justify-content: flex-end; gap: 20px;",

                    div {
                        style: "display: flex; align-items: center; gap: 12px; border-right: 1px solid rgba(255,255,255,0.1); padding-right: 20px; margin-right: 4px;",
                        {props.children}
                    }

                    if let Some(user) = props.user.clone() {
                        if let Some(upload_route) = props.upload_route.clone() {
                            Link {
                                class: "glow-hover",
                                to: upload_route,
                                style: "padding: 8px 18px; border-radius: 20px; background: rgba(255, 255, 255, 0.05); border: 1px solid rgba(255, 255, 255, 0.1); color: var(--text-primary); font-weight: 600; text-decoration: none; font-size: 13px; transition: all 0.2s ease;",
                                "{i18n.t(\"upload\")}"
                            }
                        }

                        div {
                            class: "notifications-container",
                            style: "position: relative;",
                            button {
                                style: "background: none; border: none; color: white; cursor: pointer; padding: 8px; border-radius: 50%; transition: background 0.2s; display: flex; align-items: center; justify-content: center; position: relative;",
                                class: "glow-hover",
                                onclick: move |_| notif_menu_open.toggle(),
                                Bell { size: 20 }
                                if let Some(notifs) = notifications_res.read().as_ref() {
                                    if notifs.iter().any(|n| !n.is_read) {
                                        div {
                                            style: "position: absolute; top: 8px; right: 8px; width: 8px; height: 8px; background: #ef4444; border-radius: 50%; border: 2px solid #1a1a1a;"
                                        }
                                    }
                                }
                            }
                            if notif_menu_open() {
                                div {
                                    style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 1000;",
                                    onclick: move |e| { e.stop_propagation(); notif_menu_open.set(false); }
                                }
                                div {
                                    class: "glass user-dropdown",
                                    style: "position: absolute; top: 50px; right: -20px; width: 320px; max-height: 400px; overflow-y: auto; padding: 12px; border-radius: 16px; border: 1px solid rgba(255,255,255,0.1); z-index: 1001; display: flex; flex-direction: column; gap: 8px;",
                                    onclick: move |e| e.stop_propagation(),
                                    div {
                                        style: "display: flex; align-items: center; justify-content: space-between; padding: 0 4px 8px; border-bottom: 1px solid rgba(255,255,255,0.1);",
                                        span { style: "font-weight: 700; color: white;", "{i18n.t(\"notifications\")}" }
                                        if let Some(notifs) = notifications_res.read().as_ref() {
                                            if notifs.iter().any(|n| !n.is_read) {
                                                button {
                                                    style: "background: none; border: none; color: var(--accent-primary); cursor: pointer; font-size: 12px; font-weight: 600;",
                                                    onclick: move |_| {
                                                        spawn(async move {
                                                            let _ = api::mark_all_notifications_read().await;
                                                            notifications_res.restart();
                                                        });
                                                    },
                                                    "{i18n.t(\"mark_all_read\")}"
                                                }
                                            }
                                        }
                                    }
                                    if let Some(notifs) = notifications_res.read().as_ref() {
                                        if notifs.is_empty() {
                                            div {
                                                style: "padding: 24px 0; text-align: center; color: var(--text-muted); font-size: 14px;",
                                                "{i18n.t(\"no_notifications\")}"
                                            }
                                        } else {
                                            for notif in notifs {
                                                NotificationItem {
                                                    notif: notif.clone(),
                                                    on_read: move |nid: String| {
                                                        spawn(async move {
                                                            let _ = api::mark_notification_read(nid).await;
                                                            notifications_res.restart();
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        div { style: "padding: 20px; text-align: center; color: var(--text-muted);", "{i18n.t(\"loading\")}" }
                                    }
                                }
                            }
                        }

                        div {
                            class: "user-menu-container",
                            style: "position: relative; display: flex; align-items: center; gap: 12px; cursor: pointer; padding: 4px 8px; border-radius: 12px; transition: background 0.2s;",
                            onclick: move |_| user_menu_open.toggle(),

                            div {
                                class: "user-info",
                                style: "text-align: right; display: flex; flex-direction: column;",
                                span { style: "font-size: 14px; font-weight: 700; color: white;", "{user.name}" }
                                span { style: "font-size: 11px; color: var(--text-muted);", "{i18n.t(\"pro_member\")}" }
                            }
                            img {
                                src: "{crate::resolve_asset_url(&user.pfp_url)}",
                                style: "width: 40px; height: 40px; border-radius: 50%; object-fit: cover; border: 2px solid rgba(255,255,255,0.1);"
                            }

                            if user_menu_open() {
                                div {
                                    style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 1000;",
                                    onclick: move |e| { e.stop_propagation(); user_menu_open.set(false); }
                                }
                                div {
                                    class: "glass user-dropdown",
                                    style: "position: absolute; top: 50px; right: 0; width: 220px; padding: 12px; border-radius: 16px; border: 1px solid rgba(255,255,255,0.1); z-index: 1001; display: flex; flex-direction: column; gap: 4px;",
                                    onclick: move |e| e.stop_propagation(),

                                    if let Some(profile) = props.profile_route.clone() {
                                        Link {
                                            to: profile,
                                            onclick: move |_| user_menu_open.set(false),
                                            style: "padding: 10px 16px; border-radius: 10px; color: white; text-decoration: none; font-size: 14px; font-weight: 600; transition: background 0.2s; display: flex; align-items: center; gap: 10px;",
                                            class: "menu-item-hover",
                                            User { size: 16 }
                                            "{i18n.t(\"view_profile\")}"
                                        }
                                    }

                                    if let Some(settings) = props.settings_route.clone() {
                                        Link {
                                            to: settings,
                                            onclick: move |_| user_menu_open.set(false),
                                            style: "padding: 10px 16px; border-radius: 10px; color: white; text-decoration: none; font-size: 14px; font-weight: 600; transition: background 0.2s; display: flex; align-items: center; gap: 10px;",
                                            class: "menu-item-hover",
                                            Settings { size: 16 }
                                            "{i18n.t(\"settings\")}"
                                        }
                                    }

                                    div { style: "height: 1px; background: rgba(255,255,255,0.1); margin: 4px 0;" }

                                    button {
                                        style: "padding: 10px 16px; border-radius: 10px; color: #ff4d4d; background: none; border: none; text-align: left; font-size: 14px; font-weight: 600; cursor: pointer; transition: background 0.2s; display: flex; align-items: center; gap: 10px;",
                                        class: "menu-item-hover-danger",
                                        onclick: move |_| {
                                            user_menu_open.set(false);
                                            if let Some(onlogout) = props.onlogout {
                                                onlogout.call(());
                                            }
                                        },
                                        LogOut { size: 16 }
                                        "{i18n.t(\"logout\")}"
                                    }
                                }
                            }
                        }
                    } else {
                        if let Some(login) = props.login_action {
                            {login}
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ExploreDropdown(sections: Element) -> Element {
    let mut is_open = use_signal(|| false);
    let i18n = crate::i18n::use_i18n();
    #[allow(unused_mut)]
    let mut menu_offset = use_signal(|| "-200px".to_string());

    use_effect(move || {
        if is_open() {
            spawn(async move {
                #[allow(unused_mut, unused_variables)]
                let mut menu_offset = menu_offset;
                #[cfg(target_arch = "wasm32")]
                {
                    gloo_timers::future::TimeoutFuture::new(10).await;
                    if let Some(window) = web_sys::window() {
                        let win_width = window.inner_width().unwrap().as_f64().unwrap_or(1200.0);
                        if let Some(trigger) = window
                            .document()
                            .unwrap()
                            .get_element_by_id("explore-trigger")
                        {
                            let rect = trigger.get_bounding_client_rect();
                            let trigger_left = rect.left();

                            if trigger_left - 200.0 + 900.0 > win_width - 20.0 {
                                let overflow = (trigger_left - 200.0 + 900.0) - (win_width - 20.0);
                                menu_offset.set(format!("{}px", -200.0 - overflow));
                            } else if trigger_left - 200.0 < 20.0 {
                                menu_offset.set(format!("{}px", -trigger_left + 20.0));
                            } else {
                                menu_offset.set("-200px".to_string());
                            }
                        }
                    }
                }
            });
        }
    });

    rsx! {
        div {
            class: "explore-container",

            span {
                id: "explore-trigger",
                class: "explore-trigger",
                style: "color: var(--text-secondary); font-weight: 600; font-size: 14px; cursor: pointer; display: flex; align-items: center; gap: 4px;",
                onclick: move |_| is_open.toggle(),
                "{i18n.t(\"explore\")}"
                ChevronDown { size: 16 }
            }

            if is_open() {
                div {
                    style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 1000; background: transparent;",
                    onclick: move |_| is_open.set(false),
                }
                div {
                    class: "explore-mega-menu",
                    style: "position: absolute; top: calc(100% + 12px); width: 900px; border-radius: 24px; padding: 0; z-index: 1001; overflow: hidden; left: {menu_offset};",
                    onclick: move |_| is_open.set(false),
                    div {
                        class: "mega-row",
                        style: "display: flex;",
                        {sections}
                    }
                }
            }
        }
    }
}

#[component]
pub fn DropdownSection(
    title: String,
    children: Element,
    #[props(default = false)] separator: bool,
) -> Element {
    let border_style = if separator {
        "border-left: 1px solid rgba(255, 255, 255, 0.08);"
    } else {
        ""
    };
    rsx! {
        div {
            class: if separator { "mega-col separator" } else { "mega-col" },
            style: "flex: 1; padding: 32px; display: flex; flex-direction: column; gap: 12px; {border_style}",
            h6 {
                style: "font-size: 12px; font-weight: 800; text-transform: uppercase; letter-spacing: 0.15em; color: var(--accent-primary); margin-bottom: 12px;",
                "{title}"
            }
            {children}
        }
    }
}

#[component]
fn NotificationItem(notif: api::Notification, on_read: EventHandler<String>) -> Element {
    let is_read = notif.is_read;
    let nid = notif.id.clone();

    rsx! {
        div {
            style: format!("padding: 12px; border-radius: 12px; background: {}; border: 1px solid rgba(255,255,255,0.05); display: flex; flex-direction: column; gap: 4px; cursor: pointer;", if is_read { "rgba(0,0,0,0.1)" } else { "rgba(96, 165, 250, 0.1)" }),
            onclick: move |e| {
                e.stop_propagation();
                if !is_read {
                    on_read.call(nid.clone());
                }
            },
            span { style: "font-weight: 600; font-size: 14px; color: white; display: flex; align-items: center; justify-content: space-between;",
                "{notif.title}"
                if !is_read {
                    div { style: "width: 6px; height: 6px; border-radius: 50%; background: #60a5fa;" }
                }
            }
            span { style: "font-size: 13px; color: var(--text-secondary); line-height: 1.4;", "{notif.message}" }
            span { style: "font-size: 11px; color: var(--text-muted); margin-top: 4px;", {notif.created_at.format("%b %d, %H:%M").to_string()} }
        }
    }
}
