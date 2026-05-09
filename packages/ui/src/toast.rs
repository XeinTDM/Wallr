use dioxus::prelude::*;
use lucide_dioxus::{CircleCheck, CircleX, Info, X};

#[derive(Clone, PartialEq, Debug)]
pub enum ToastType {
    Success,
    Error,
    Info,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub kind: ToastType,
}

#[component]
pub fn ToastContainer() -> Element {
    let mut toasts = use_context::<Signal<Vec<Toast>>>();

    rsx! {
        div {
            id: "toast-container",
            style: "position: fixed; bottom: 32px; right: 32px; z-index: 9999;",
            
            for (i, toast) in toasts.read().clone().into_iter().enumerate() {
                {
                    let offset = toasts.read().len() - 1 - i;
                    let translate_y = offset as i32 * -16;
                    let scale = 1.0 - (offset as f64 * 0.05);
                    let opacity = if offset > 3 { 0.0 } else { 1.0 - (offset as f64 * 0.2) };
                    let z_index = 100 - offset;
                    
                    rsx! {
                        div {
                            key: "{toast.id}",
                            class: "toast glass fade-in",
                            style: format!(
                                "position: absolute; bottom: 0; right: 0; padding: 16px 24px; border-radius: 16px; min-width: 280px; display: flex; align-items: center; gap: 12px; border: 1px solid {}; box-shadow: 0 10px 30px rgba(0,0,0,0.2); transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275); transform: translateY({}px) scale({}); opacity: {}; z-index: {}; pointer-events: {}; transform-origin: bottom right;",
                                match toast.kind {
                                    ToastType::Success => "rgba(34, 197, 94, 0.3)",
                                    ToastType::Error => "rgba(239, 68, 68, 0.3)",
                                    ToastType::Info => "var(--accent-primary)",
                                },
                                translate_y, scale, opacity, z_index,
                                if offset > 0 { "none" } else { "auto" }
                            ),
                            span { 
                                display: "flex",
                                align_items: "center",
                                match toast.kind {
                                    ToastType::Success => rsx! { CircleCheck { color: "#22c55e", size: 24 } },
                                    ToastType::Error => rsx! { CircleX { color: "#ef4444", size: 24 } },
                                    ToastType::Info => rsx! { Info { color: "var(--accent-primary)", size: 24 } },
                                }
                            }
                            div {
                                style: "flex: 1;",
                                p { style: "font-weight: 700; font-size: 14px; margin: 0;", "{toast.message}" }
                            }
                            button {
                                style: "background: none; border: none; color: white; opacity: 0.5; cursor: pointer; display: flex; align_items: center; justify-content: center; padding: 4px; border-radius: 50%; transition: opacity 0.2s;",
                                onclick: move |_| {
                                    let mut list = toasts.write();
                                    list.retain(|t| t.id != toast.id);
                                },
                                X { size: 16 }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn use_toaster() -> Toaster {
    let toasts = use_context::<Signal<Vec<Toast>>>();
    
    Toaster { toasts }
}

#[derive(Clone, Copy)]
pub struct Toaster {
    toasts: Signal<Vec<Toast>>,
}

impl Toaster {
    pub fn success(&mut self, message: impl Into<String>) {
        self.push(message.into(), ToastType::Success);
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.push(message.into(), ToastType::Error);
    }

    pub fn info(&mut self, message: impl Into<String>) {
        self.push(message.into(), ToastType::Info);
    }

    fn push(&mut self, message: String, kind: ToastType) {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        
        self.toasts.write().push(Toast { id, message, kind });
        
        let mut toasts = self.toasts;
        spawn(async move {
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::TimeoutFuture::new(5000).await;
            #[cfg(not(target_arch = "wasm32"))]
            tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
            
            toasts.with_mut(|list| list.retain(|t| t.id != id));
        });
    }
}
