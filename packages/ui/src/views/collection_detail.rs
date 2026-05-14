use crate::app::Route;
use crate::{LoadingScreen, WallpaperCard, use_toaster};
use api::{
    add_wallpaper_to_collection, get_collection_wallpapers, get_user_uploads,
    get_collection, get_public_profile_by_id, update_collection, delete_collection, remove_wallpaper_from_collection
};
use dioxus::prelude::*;

#[component]
pub fn CollectionDetail(id: String) -> Element {
    let i18n = crate::i18n::use_i18n();
    let toaster = use_toaster();
    let nav = use_navigator();
    let auth = use_context::<Signal<crate::app::AuthState>>();
    
    let mut page = use_signal(|| 0_u32);
    let mut current_id = use_signal(|| id.clone());
    
    let mut is_add_modal_open = use_signal(|| false);
    let mut is_sync_modal_open = use_signal(|| false);
    let mut is_edit_modal_open = use_signal(|| false);
    let mut is_delete_modal_open = use_signal(|| false);

    if *current_id.peek() != id {
        current_id.set(id.clone());
        page.set(0);
    }

    let mut collection = use_resource(move || {
        let cid = current_id();
        async move { get_collection(cid).await }
    });

    let owner = use_resource(move || {
        let col = collection();
        async move {
            if let Some(Ok(c)) = col {
                get_public_profile_by_id(c.user_id).await
            } else {
                Ok(None)
            }
        }
    });

    let mut wallpapers = use_resource(move || {
        let cid = current_id();
        let p = page();
        async move { get_collection_wallpapers(cid, p, 50).await }
    });

    let existing_ids = match wallpapers() {
        Some(Ok(list)) => list.iter().map(|w| w.id.clone()).collect::<Vec<_>>(),
        _ => vec![],
    };

    let copy_link = {
        let mut toaster = toaster;
        let id_val = current_id();
        move |_| {
            #[cfg(target_arch = "wasm32")]
            {
                let window = web_sys::window().unwrap();
                let url = format!("{}/collection/{}", window.location().origin().unwrap(), id_val);
                let nav = window.navigator();
                let clip = nav.clipboard();
                let _ = clip.write_text(&url);
                toaster.success("Link copied to clipboard!");
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                toaster.success("Copy link not supported in this environment");
            }
        }
    };

    rsx! {
        div {
            div {
                class: "container",
                style: "padding-top: var(--nav-height, 68px); padding-bottom: 80px;",

                div {
                    style: "margin-bottom: 40px; margin-top: 40px;",
                    Link {
                        to: Route::Collections {},
                        style: "color: var(--text-muted); text-decoration: none; display: inline-block; margin-bottom: 16px; font-weight: 600;",
                        "← {i18n.t(\"col_detail_back\")}"
                    }
                    
                    match collection() {
                        Some(Ok(col)) => rsx! {
                            div {
                                style: "display: flex; justify-content: space-between; align-items: flex-start;",
                                div {
                                    h1 { style: "font-size: 36px; font-weight: 900; margin: 0 0 8px 0;", "{col.name}" }
                                    if let Some(desc) = &col.description {
                                        if !desc.trim().is_empty() {
                                            p { style: "color: var(--text-secondary); margin: 0 0 16px 0; max-width: 600px; line-height: 1.5;", "{desc}" }
                                        }
                                    }
                                    div {
                                        style: "display: flex; gap: 16px; align-items: center; color: var(--text-muted); font-size: 14px;",
                                        match owner() {
                                            Some(Ok(Some(u))) => rsx! {
                                                Link {
                                                    to: Route::PublicProfile { username: u.name.clone() },
                                                    class: "glow-hover",
                                                    style: "color: inherit; text-decoration: none; display: flex; align-items: center; gap: 8px;",
                                                    img {
                                                        src: "{crate::resolve_asset_url(&u.pfp_url)}",
                                                        style: "width: 24px; height: 24px; border-radius: 50%; object-fit: cover;"
                                                    }
                                                    span { style: "font-weight: 600; color: white;", "{u.name}" }
                                                }
                                            },
                                            _ => rsx! { span { "Unknown Owner" } }
                                        }
                                        span { "•" }
                                        span { "{col.item_count} items" }
                                        span { "•" }
                                        if col.is_private {
                                            span { style: "display: flex; align-items: center; gap: 4px;", lucide_dioxus::Lock { size: 14 } "Private" }
                                        } else {
                                            span { style: "display: flex; align-items: center; gap: 4px;", lucide_dioxus::Globe { size: 14 } "Public" }
                                        }
                                    }
                                }
                                
                                div {
                                    style: "display: flex; gap: 12px; align-items: center;",
                                    button {
                                        class: "glow-hover",
                                        style: "padding: 10px; background: rgba(255,255,255,0.1); border-radius: 12px; color: white; border: 1px solid rgba(255,255,255,0.2); cursor: pointer;",
                                        title: "Copy Link",
                                        onclick: copy_link,
                                        lucide_dioxus::Link2 { size: 18 }
                                    }
                                    {
                                        let is_owner = match auth() {
                                            crate::app::AuthState::Authenticated(u) => u.id == col.user_id || u.role == "admin" || u.role == "super_admin",
                                            _ => false,
                                        };
                                        if is_owner {
                                            rsx! {
                                                button {
                                                    class: "glow-hover",
                                                    style: "padding: 10px; background: rgba(255,255,255,0.1); border-radius: 12px; color: white; border: 1px solid rgba(255,255,255,0.2); cursor: pointer;",
                                                    title: "Edit Collection",
                                                    onclick: move |_| is_edit_modal_open.set(true),
                                                    lucide_dioxus::Pen { size: 18 }
                                                }
                                                button {
                                                    class: "glow-hover",
                                                    style: "padding: 10px; background: rgba(239, 68, 68, 0.2); border-radius: 12px; color: #ef4444; border: 1px solid rgba(239, 68, 68, 0.3); cursor: pointer;",
                                                    title: "Delete Collection",
                                                    onclick: move |_| is_delete_modal_open.set(true),
                                                    lucide_dioxus::Trash2 { size: 18 }
                                                }
                                                button {
                                                    class: "glow-hover",
                                                    style: "padding: 10px 20px; background: var(--accent-primary); border-radius: 12px; color: white; font-weight: 600; border: none; cursor: pointer; display: flex; align-items: center; gap: 8px;",
                                                    onclick: move |_| is_add_modal_open.set(true),
                                                    lucide_dioxus::Plus { size: 18 }
                                                    "{i18n.t(\"col_detail_add_btn\")}"
                                                }
                                            }
                                        } else {
                                            rsx! {}
                                        }
                                    }
                                    button {
                                        class: "glow-hover",
                                        style: "padding: 10px 20px; background: rgba(255,255,255,0.1); border-radius: 12px; color: white; font-weight: 600; border: 1px solid rgba(255,255,255,0.2); cursor: pointer; display: flex; align-items: center; gap: 8px;",
                                        onclick: move |_| is_sync_modal_open.set(true),
                                        lucide_dioxus::Monitor { size: 18 }
                                        "Sync to Desktop"
                                    }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! { div { "Error loading collection metadata: {e}" } },
                        None => rsx! { div { "Loading..." } }
                    }
                }

                match wallpapers() {
                    Some(Ok(list)) => {
                        if list.is_empty() {
                            rsx! {
                                div {
                                    style: "display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 80px 20px; text-align: center;",
                                    lucide_dioxus::Image { size: 48, color: "rgba(255,255,255,0.2)", class: "mb-4" }
                                    h3 { style: "font-size: 20px; font-weight: 700; margin-bottom: 8px;", "{i18n.t(\"col_detail_empty_title\")}" }
                                    p { style: "color: var(--text-muted); margin-bottom: 24px;", "{i18n.t(\"col_detail_empty_desc\")}" }
                                    Link {
                                        to: Route::Explore { tag: "all".to_string() },
                                        class: "glow-hover",
                                        style: "padding: 10px 24px; background: rgba(255,255,255,0.1); border-radius: 12px; color: white; font-weight: 600; text-decoration: none;",
                                        "{i18n.t(\"col_detail_discover\")}"
                                    }
                                }
                            }
                        } else {
                            rsx! {
                                div {
                                    class: "wallpaper-grid",
                                    style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 32px;",
                                    for wp in list.iter() {
                                        WallpaperCard {
                                            key: "{wp.id}",
                                            wallpaper: wp.clone(),
                                            on_remove: {
                                                let w_id = wp.id.clone();
                                                let c_id = current_id();
                                                let mut toaster = toaster;
                                                move |_| {
                                                    let w_id = w_id.clone();
                                                    let c_id = c_id.clone();
                                                    let mut toaster = toaster;
                                                    spawn(async move {
                                                        if let Ok(_) = remove_wallpaper_from_collection(c_id, w_id).await {
                                                            wallpapers.restart();
                                                            collection.restart();
                                                        } else {
                                                            toaster.error("Failed to remove item.");
                                                        }
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Some(Err(e)) => rsx! { div { class: "error", "{i18n.t(\"col_detail_error\")}{e}" } },
                    None => rsx! { LoadingScreen {} }
                }
            }
            AddFromUploadsModal {
                is_open: is_add_modal_open,
                collection_id: current_id(),
                existing_ids,
                on_added: move |_| {
                    wallpapers.restart();
                    collection.restart();
                }
            }
            SyncModal {
                is_open: is_sync_modal_open,
                collection_id: current_id(),
            }
            if let Some(Ok(col)) = collection() {
                EditCollectionModal {
                    is_open: is_edit_modal_open,
                    collection: col,
                    on_saved: move |_| {
                        collection.restart();
                    }
                }
                DeleteCollectionModal {
                    is_open: is_delete_modal_open,
                    collection_id: current_id(),
                    on_deleted: move |_| {
                        nav.push(Route::Collections {});
                    }
                }
            }
        }
    }
}

#[component]
fn EditCollectionModal(
    is_open: Signal<bool>,
    collection: api::UserCollection,
    on_saved: EventHandler<()>,
) -> Element {
    let mut is_open_sig = is_open;
    let mut name = use_signal(|| collection.name.clone());
    let mut desc = use_signal(|| collection.description.clone().unwrap_or_default());
    let mut is_private = use_signal(|| collection.is_private);
    let mut is_submitting = use_signal(|| false);
    let toaster = use_toaster();

    if !is_open_sig() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "modal-overlay fade-in",
            style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.7); backdrop-filter: blur(8px); z-index: 1000; display: flex; align-items: center; justify-content: center;",
            onclick: move |e| { e.stop_propagation(); is_open_sig.set(false); },
            div {
                class: "glass slide-up",
                style: "width: 90%; max-width: 400px; border-radius: 24px; padding: 32px; display: flex; flex-direction: column; gap: 24px; border: 1px solid rgba(255,255,255,0.1); box-shadow: 0 25px 50px -12px rgba(0,0,0,0.5);",
                onclick: move |e| e.stop_propagation(),

                h2 { style: "font-size: 24px; font-weight: 800; margin: 0; color: white;", "Edit Collection" }

                div {
                    style: "display: flex; flex-direction: column; gap: 8px;",
                    label { style: "font-weight: 600; font-size: 14px;", "Name" }
                    input {
                        r#type: "text",
                        value: "{name}",
                        style: "width: 100%; background: rgba(0,0,0,0.2); border: 1px solid rgba(255,255,255,0.1); padding: 12px; border-radius: 12px; color: white; font-size: 14px; outline: none;",
                        oninput: move |e| name.set(e.value().clone()),
                    }
                }

                div {
                    style: "display: flex; flex-direction: column; gap: 8px;",
                    label { style: "font-weight: 600; font-size: 14px;", "Description" }
                    textarea {
                        value: "{desc}",
                        style: "width: 100%; background: rgba(0,0,0,0.2); border: 1px solid rgba(255,255,255,0.1); padding: 12px; border-radius: 12px; color: white; font-size: 14px; outline: none; min-height: 80px; resize: vertical;",
                        oninput: move |e| desc.set(e.value().clone()),
                    }
                }

                div {
                    style: "display: flex; align-items: center; gap: 12px; cursor: pointer;",
                    onclick: move |_| is_private.set(!is_private()),
                    div {
                        style: format!("width: 48px; height: 24px; border-radius: 12px; background: {}; position: relative; transition: all 0.3s;", if is_private() { "var(--accent-primary)" } else { "rgba(255,255,255,0.1)" }),
                        div {
                            style: format!("width: 20px; height: 20px; border-radius: 50%; background: white; position: absolute; top: 2px; transition: all 0.3s; left: {};", if is_private() { "26px" } else { "2px" }),
                        }
                    }
                    span { style: "font-weight: 600; font-size: 14px;", "Private Collection" }
                }

                div { style: "display: flex; gap: 12px; justify-content: flex-end; margin-top: 8px;",
                    button {
                        class: "glow-hover",
                        style: "padding: 10px 20px; border-radius: 12px; background: transparent; color: white; font-weight: 600; border: 1px solid rgba(255,255,255,0.1); cursor: pointer;",
                        onclick: move |_| is_open_sig.set(false),
                        "Cancel"
                    }
                    button {
                        class: "glow-hover",
                        style: "padding: 10px 20px; border-radius: 12px; background: var(--accent-primary); color: white; font-weight: 600; border: none; cursor: pointer;",
                        disabled: name().trim().is_empty() || is_submitting(),
                        onclick: {
                            let cid = collection.id.clone();
                            let mut toaster = toaster;
                            move |_| {
                                let name_val = name().clone();
                                let desc_val = desc().clone();
                                let priv_val = is_private();
                                let cid = cid.clone();
                                let mut toaster = toaster;
                                is_submitting.set(true);
                                spawn(async move {
                                    let desc_opt = if desc_val.trim().is_empty() { None } else { Some(desc_val) };
                                    if let Ok(_) = update_collection(cid, name_val, desc_opt, priv_val).await {
                                        toaster.success("Collection updated");
                                        on_saved.call(());
                                        is_open_sig.set(false);
                                    } else {
                                        toaster.error("Failed to update collection");
                                    }
                                    is_submitting.set(false);
                                });
                            }
                        },
                        if is_submitting() { "Saving..." } else { "Save Changes" }
                    }
                }
            }
        }
    }
}

#[component]
fn DeleteCollectionModal(
    is_open: Signal<bool>,
    collection_id: String,
    on_deleted: EventHandler<()>,
) -> Element {
    let mut is_open_sig = is_open;
    let mut is_submitting = use_signal(|| false);
    let toaster = use_toaster();

    if !is_open_sig() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "modal-overlay fade-in",
            style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.7); backdrop-filter: blur(8px); z-index: 1000; display: flex; align-items: center; justify-content: center;",
            onclick: move |e| { e.stop_propagation(); is_open_sig.set(false); },
            div {
                class: "glass slide-up",
                style: "width: 90%; max-width: 400px; border-radius: 24px; padding: 32px; display: flex; flex-direction: column; gap: 24px; border: 1px solid rgba(255,255,255,0.1); box-shadow: 0 25px 50px -12px rgba(0,0,0,0.5);",
                onclick: move |e| e.stop_propagation(),

                h2 { style: "font-size: 24px; font-weight: 800; margin: 0; color: #ef4444;", "Delete Collection" }
                p { style: "color: var(--text-secondary); margin: 0; line-height: 1.5;",
                    "Are you sure you want to delete this collection? The wallpapers themselves will not be deleted, but the collection will be permanently removed."
                }

                div { style: "display: flex; gap: 12px; justify-content: flex-end; margin-top: 8px;",
                    button {
                        class: "glow-hover",
                        style: "padding: 10px 20px; border-radius: 12px; background: transparent; color: white; font-weight: 600; border: 1px solid rgba(255,255,255,0.1); cursor: pointer;",
                        onclick: move |_| is_open_sig.set(false),
                        "Cancel"
                    }
                    button {
                        class: "glow-hover",
                        style: "padding: 10px 20px; border-radius: 12px; background: rgba(239, 68, 68, 0.2); color: #ef4444; font-weight: 600; border: 1px solid rgba(239, 68, 68, 0.3); cursor: pointer;",
                        disabled: is_submitting(),
                        onclick: {
                            let cid = collection_id.clone();
                            let mut toaster = toaster;
                            move |_| {
                                let cid = cid.clone();
                                let mut toaster = toaster;
                                is_submitting.set(true);
                                spawn(async move {
                                    if let Ok(_) = delete_collection(cid).await {
                                        toaster.success("Collection deleted");
                                        on_deleted.call(());
                                        is_open_sig.set(false);
                                    } else {
                                        toaster.error("Failed to delete collection");
                                    }
                                    is_submitting.set(false);
                                });
                            }
                        },
                        if is_submitting() { "Deleting..." } else { "Delete Forever" }
                    }
                }
            }
        }
    }
}


#[component]
fn AddFromUploadsModal(
    is_open: Signal<bool>,
    collection_id: String,
    existing_ids: Vec<String>,
    on_added: EventHandler<()>,
) -> Element {
    let i18n = crate::i18n::use_i18n();
    let mut just_added = use_signal(std::collections::HashSet::<String>::new);

    use_effect(move || {
        if !is_open() {
            just_added.set(std::collections::HashSet::new());
        }
    });
    let uploads = use_resource(move || {
        let open = is_open();
        async move {
            if open {
                get_user_uploads(0, 100).await
            } else {
                Err(dioxus::prelude::ServerFnError::new("Not open"))
            }
        }
    });

    use_effect(move || {
        #[allow(unused_variables)]
        let current_is_open = is_open();
        #[cfg(target_arch = "wasm32")]
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(body) = document.body() {
                    if current_is_open {
                        let _ = body.set_attribute("style", "overflow: hidden;");
                    } else {
                        let _ = body.remove_attribute("style");
                    }
                }
            }
        }
    });

    if !is_open() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "modal-overlay fade-in",
            style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.7); backdrop-filter: blur(8px); z-index: 1000; display: flex; align-items: center; justify-content: center;",
            onclick: move |e| {
                e.stop_propagation();
                is_open.set(false);
            },

            div {
                class: "glass slide-up",
                style: "width: 90%; max-width: 800px; max-height: 80vh; border-radius: 24px; padding: 32px; display: flex; flex-direction: column; gap: 24px; border: 1px solid rgba(255,255,255,0.1); box-shadow: 0 25px 50px -12px rgba(0,0,0,0.5);",
                onclick: move |e| e.stop_propagation(),

                div {
                    style: "display: flex; justify-content: space-between; align-items: center;",
                    h2 {
                        style: "font-size: 24px; font-weight: 800; margin: 0; color: white;",
                        "{i18n.t(\"col_detail_modal_title\")}"
                    }
                    button {
                        style: "background: none; border: none; cursor: pointer; padding: 8px;",
                        onclick: move |_| is_open.set(false),
                        lucide_dioxus::X { size: 24, color: "var(--text-muted)" }
                    }
                }

                div {
                    style: "overflow-y: auto; flex-grow: 1; padding-right: 8px;",
                    match uploads() {
                        Some(Ok(list)) => {
                            let filtered_list: Vec<_> = list.iter()
                                .filter(|wp| !existing_ids.contains(&wp.id) && !just_added().contains(&wp.id))
                                .collect();

                            if filtered_list.is_empty() {
                                rsx! {
                                    div {
                                        style: "text-align: center; padding: 40px; color: var(--text-muted);",
                                        "{i18n.t(\"col_detail_modal_empty\")}"
                                    }
                                }
                            } else {
                                rsx! {
                                    div {
                                        style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); gap: 16px;",
                                        for wp in filtered_list {
                                            div {
                                                key: "{wp.id}",
                                                style: "position: relative; border-radius: 12px; overflow: hidden; cursor: pointer; aspect-ratio: 16/9;",
                                                class: "glow-hover",
                                                onclick: {
                                                    let w_id = wp.id.clone();
                                                    let c_id = collection_id.clone();
                                                    let on_added = on_added;
                                                    let mut just_added_sig = just_added;
                                                    move |_| {
                                                        let w_id = w_id.clone();
                                                        let c_id = c_id.clone();
                                                        let on_added = on_added;
                                                        just_added_sig.write().insert(w_id.clone());
                                                        spawn(async move {
                                                            if let Ok(_) = add_wallpaper_to_collection(c_id, w_id).await {
                                                                on_added.call(());
                                                            }
                                                        });
                                                    }
                                                },
                                                img {
                                                    src: "{crate::resolve_asset_url(&wp.thumbnail_url)}",
                                                    style: "width: 100%; height: 100%; object-fit: cover;"
                                                }
                                                div {
                                                    style: "position: absolute; inset: 0; background: linear-gradient(transparent, rgba(0,0,0,0.8)); display: flex; align-items: flex-end; padding: 12px;",
                                                    span {
                                                        style: "color: white; font-size: 12px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                                                        "{wp.title}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        _ => rsx! {
                            div {
                                style: "display: flex; justify-content: center; padding: 40px;",
                                crate::LoadingScreen {}
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SyncModal(is_open: Signal<bool>, collection_id: String) -> Element {
    let mut interval = use_signal(|| 3600);
    let mut is_submitting = use_signal(|| false);

    if !is_open() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "modal-overlay fade-in",
            style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.7); backdrop-filter: blur(8px); z-index: 1000; display: flex; align-items: center; justify-content: center;",
            onclick: move |e| {
                e.stop_propagation();
                is_open.set(false);
            },
            div {
                class: "glass slide-up",
                style: "width: 90%; max-width: 400px; border-radius: 24px; padding: 32px; display: flex; flex-direction: column; gap: 24px; border: 1px solid rgba(255,255,255,0.1); box-shadow: 0 25px 50px -12px rgba(0,0,0,0.5);",
                onclick: move |e| e.stop_propagation(),

                h2 { style: "margin: 0; font-size: 24px; font-weight: 800;", "Sync to Desktop" }
                p { style: "color: var(--text-muted); margin: 0;", "Your desktop app will automatically rotate wallpapers from this collection." }

                div {
                    style: "display: flex; flex-direction: column; gap: 8px;",
                    label { style: "font-weight: 600; font-size: 14px;", "Rotation Interval" }
                    select {
                        style: "padding: 12px; background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); border-radius: 8px; color: white; outline: none;",
                        onchange: move |e| {
                            if let Ok(val) = e.value().parse() {
                                interval.set(val);
                            }
                        },
                        option { value: "300", "5 Minutes" }
                        option { value: "900", "15 Minutes" }
                        option { value: "3600", selected: true, "1 Hour" }
                        option { value: "86400", "24 Hours" }
                    }
                }

                div {
                    style: "display: flex; gap: 12px; justify-content: flex-end; margin-top: 16px;",
                    button {
                        style: "padding: 10px 20px; background: transparent; border: 1px solid rgba(255,255,255,0.2); border-radius: 12px; color: white; cursor: pointer;",
                        onclick: move |_| is_open.set(false),
                        "Cancel"
                    }
                    button {
                        class: "glow-hover",
                        style: "padding: 10px 20px; background: var(--accent-primary); border: none; border-radius: 12px; color: white; font-weight: 600; cursor: pointer;",
                        disabled: is_submitting(),
                        onclick: move |_| {
                            let cid = collection_id.clone();
                            is_submitting.set(true);
                            spawn(async move {
                                let _ = api::set_active_playlist(Some(cid), Some(interval())).await;
                                is_submitting.set(false);
                                is_open.set(false);
                            });
                        },
                        if is_submitting() { "Saving..." } else { "Start Sync" }
                    }
                }
            }
        }
    }
}
