use api::{
    EditorialCollection, add_wallpaper_to_editorial_collection, create_editorial_collection,
    get_all_editorial_collections, get_editorial_collection_wallpapers,
    remove_wallpaper_from_editorial_collection, update_editorial_collection,
};
use dioxus::prelude::*;
use lucide_dioxus::{ArrowLeft, Image as ImageIcon, Pencil, Plus, Trash2};

const EDITORIAL_CSS: Asset = asset!("/assets/styling/admin_editorial.css");

#[component]
pub fn AdminEditorial() -> Element {
    let mut selected_collection = use_signal(|| None::<EditorialCollection>);
    let mut is_creating = use_signal(|| false);

    rsx! {
        document::Stylesheet { href: EDITORIAL_CSS }
        div { class: "admin-editorial",
            if is_creating() {
                EditorialEditForm {
                    collection: None,
                    on_close: move |_| is_creating.set(false),
                    on_saved: move |_| is_creating.set(false),
                }
            } else if let Some(collection) = selected_collection() {
                EditorialEditForm {
                    collection: Some(collection.clone()),
                    on_close: move |_| selected_collection.set(None),
                    on_saved: move |_| {
                        selected_collection.set(None);
                    },
                }
            } else {
                EditorialList {
                    on_create: move |_| is_creating.set(true),
                    on_select: move |c| selected_collection.set(Some(c)),
                }
            }
        }
    }
}

#[component]
fn EditorialList(
    on_create: EventHandler<()>,
    on_select: EventHandler<EditorialCollection>,
) -> Element {
    let collections_res =
        use_server_future(move || async move { get_all_editorial_collections().await })?;

    rsx! {
        div { class: "editorial-header",
            h1 { class: "editorial-title",
                lucide_dioxus::Library { size: 32, color: "var(--accent-primary)" }
                "Editorial Collections"
            }
            button { class: "create-btn", onclick: move |_| on_create.call(()),
                Plus { size: 18 }
                "Create New"
            }
        }

        match collections_res() {
            Some(Ok(collections)) => rsx! {
                div { class: "collection-grid",
                    for collection in collections {
                        div {
                            key: "{collection.id}",
                            class: "collection-card",
                            onclick: {
                                let c = collection.clone();
                                move |_| on_select.call(c.clone())
                            },
                            if let Some(cover) = &collection.cover_url {
                                div {
                                    class: "collection-cover",
                                    style: "background-image: url('{cover}');",
                                }
                            } else {
                                div {
                                    class: "collection-cover",
                                    style: "display: flex; align-items: center; justify-content: center;",
                                    ImageIcon { size: 48, color: "rgba(255,255,255,0.2)" }
                                }
                            }
                            div { class: "collection-info",
                                h3 { "{collection.title}" }
                                p { "{collection.description}" }
                                {
                                    let status_class = if collection.is_published { "published" } else { "draft" };
                                    let status_text = if collection.is_published { "Published" } else { "Draft" };
                                    rsx! {
                                        div { class: "status-badge {status_class}", "{status_text}" }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Some(Err(e)) => rsx! {
                p { style: "color: #ef4444;", "Error loading collections: {e}" }
            },
            None => rsx! {
                p { "Loading..." }
            },
        }
    }
}

#[component]
fn EditorialEditForm(
    collection: Option<EditorialCollection>,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let mut title = use_signal(|| {
        collection
            .as_ref()
            .map(|c| c.title.clone())
            .unwrap_or_default()
    });
    let mut description = use_signal(|| {
        collection
            .as_ref()
            .map(|c| c.description.clone())
            .unwrap_or_default()
    });
    let mut cover_url = use_signal(|| {
        collection
            .as_ref()
            .and_then(|c| c.cover_url.clone())
            .unwrap_or_default()
    });
    let mut is_published =
        use_signal(|| collection.as_ref().map(|c| c.is_published).unwrap_or(false));
    let mut error_msg = use_signal(|| String::new());

    let is_editing = collection.is_some();
    let col_id = collection
        .as_ref()
        .map(|c| c.id.clone())
        .unwrap_or_default();

    rsx! {
        div {
            button { class: "back-btn", onclick: move |_| on_close.call(()),
                ArrowLeft { size: 18 }
                "Back to List"
            }

            h2 { style: "margin-top: 0; margin-bottom: 24px; display: flex; align-items: center; gap: 8px;",
                Pencil { size: 24, color: "var(--accent-primary)" }
                if is_editing {
                    "Edit Collection"
                } else {
                    "Create Collection"
                }
            }

            if !error_msg().is_empty() {
                p { style: "color: #ef4444; background: rgba(239,68,68,0.1); padding: 12px; border-radius: 8px;",
                    "{error_msg}"
                }
            }

            div { class: "edit-form-group",
                label { "Title" }
                input {
                    r#type: "text",
                    value: "{title}",
                    oninput: move |e| title.set(e.value()),
                    placeholder: "e.g. Best of Cyberpunk",
                }
            }

            div { class: "edit-form-group",
                label { "Description" }
                textarea {
                    value: "{description}",
                    oninput: move |e| description.set(e.value()),
                    placeholder: "A short description of this collection.",
                }
            }

            div { class: "edit-form-group",
                label { "Cover URL (Optional)" }
                input {
                    r#type: "text",
                    value: "{cover_url}",
                    oninput: move |e| cover_url.set(e.value()),
                    placeholder: "https://example.com/cover.jpg",
                }
            }

            div {
                class: "edit-form-group",
                style: "flex-direction: row; align-items: center;",
                label { class: "checkbox-group",
                    input {
                        r#type: "checkbox",
                        checked: "{is_published}",
                        onchange: move |e| is_published.set(e.value() == "true"),
                    }
                    "Published (Visible to users)"
                }
            }

            button {
                class: "save-btn",
                onclick: move |_| {
                    let col_id = col_id.clone();
                    let t = title();
                    let d = description();
                    let c_url = if cover_url().is_empty() { None } else { Some(cover_url()) };
                    let publ = is_published();

                    spawn(async move {
                        if is_editing {
                            match update_editorial_collection(
                                    col_id.clone(),
                                    t.clone(),
                                    d.clone(),
                                    c_url.clone(),
                                    publ,
                                )
                                .await
                            {
                                Ok(_) => {
                                    on_saved.call(());
                                }
                                Err(e) => error_msg.set(e.to_string()),
                            }
                        } else {
                            match create_editorial_collection(
                                    t.clone(),
                                    d.clone(),
                                    c_url.clone(),
                                    publ,
                                )
                                .await
                            {
                                Ok(_) => {
                                    on_saved.call(());
                                }
                                Err(e) => error_msg.set(e.to_string()),
                            }
                        }
                    });
                },
                "Save Collection"
            }

            if is_editing {
                WallpaperManager { collection_id: col_id.clone() }
            }
        }
    }
}

#[component]
fn WallpaperManager(collection_id: String) -> Element {
    let mut new_wallpaper_id = use_signal(|| String::new());
    let mut new_sort_order = use_signal(|| 0i32);
    let mut error_msg = use_signal(|| String::new());

    let col_id_clone = collection_id.clone();
    let mut wallpapers_res = use_server_future(move || {
        let cid = col_id_clone.clone();
        async move { get_editorial_collection_wallpapers(cid, 1).await }
    })?;

    rsx! {
        div { class: "wallpaper-management",
            h2 { style: "margin-top: 0; margin-bottom: 24px;", "Manage Wallpapers" }

            if !error_msg().is_empty() {
                p { style: "color: #ef4444;", "{error_msg}" }
            }

            div { class: "add-wallpaper-row",
                input {
                    r#type: "text",
                    placeholder: "Wallpaper ID",
                    value: "{new_wallpaper_id}",
                    oninput: move |e| new_wallpaper_id.set(e.value()),
                }
                input {
                    r#type: "number",
                    placeholder: "Sort Order",
                    value: "{new_sort_order}",
                    style: "max-width: 120px;",
                    oninput: move |e| {
                        if let Ok(v) = e.value().parse::<i32>() {
                            new_sort_order.set(v);
                        }
                    },
                }
                button {
                    onclick: move |_| {
                        let cid = collection_id.clone();
                        let wid = new_wallpaper_id();
                        let order = new_sort_order();
                        spawn(async move {
                            match add_wallpaper_to_editorial_collection(cid, wid, order).await {
                                Ok(_) => {
                                    new_wallpaper_id.set(String::new());
                                    error_msg.set(String::new());
                                    wallpapers_res.restart();
                                }
                                Err(e) => error_msg.set(e.to_string()),
                            }
                        });
                    },
                    Plus { size: 16, style: "margin-right: 4px;" }
                    "Add"
                }
            }

            match wallpapers_res() {
                Some(Ok(wallpapers)) => {
                    if wallpapers.is_empty() {
                        rsx! {
                            p { style: "color: var(--text-muted);", "No wallpapers in this collection yet." }
                        }
                    } else {
                        rsx! {
                            div { class: "wallpaper-grid",
                                for wp in wallpapers {
                                    {
                                        let short_id = wp.id.chars().take(8).collect::<String>();
                                        rsx! {
                                            div { key: "{wp.id}", class: "wallpaper-item",
                                                img { src: "{wp.thumbnail_url}" }
                                                div { class: "wallpaper-overlay",
                                                    span { style: "color: white; font-size: 12px; font-weight: 600;", "ID: {short_id}..." }
                                                    button {
                                                        class: "remove-btn",
                                                        onclick: {
                                                            let cid = collection_id.clone();
                                                            let wid = wp.id.clone();
                                                            move |_| {
                                                                let cid = cid.clone();
                                                                let wid = wid.clone();
                                                                spawn(async move {
                                                                    if let Ok(_) = remove_wallpaper_from_editorial_collection(cid, wid).await
                                                                    {
                                                                        wallpapers_res.restart();
                                                                    }
                                                                });
                                                            }
                                                        },
                                                        Trash2 { size: 16 }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! {
                    p { style: "color: #ef4444;", "Error: {e}" }
                },
                None => rsx! {
                    p { "Loading wallpapers..." }
                },
            }
        }
    }
}
