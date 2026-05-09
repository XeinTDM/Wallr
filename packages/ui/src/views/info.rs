use dioxus::prelude::*;

#[component]
pub fn About() -> Element {
    rsx! {
        div {
            class: "container",
            style: "padding: 120px 20px 80px;",
            div {
                class: "legal-content reveal-up show",
                style: "max-width: 800px; margin: 0 auto;",

                h1 {
                    style: "font-size: 48px; font-weight: 900; margin-bottom: 32px;",
                    "About Wallr"
                }

                div {
                    class: "glass",
                    style: "padding: 40px; border-radius: 24px; color: var(--text-secondary); line-height: 1.6;",

                    p {
                        style: "font-size: 20px; color: white; margin-bottom: 24px; font-weight: 600;",
                        "Wallr is a high-performance wallpaper platform built for the modern web."
                    }
                    p {
                        style: "margin-bottom: 24px;",
                        "We believe discovering beautiful backgrounds for your screens should be fast, seamless, and fiercely protective of your privacy. We grew tired of bloated, slow wallpaper sites running on heavy Python backends and relying on invasive, third-party AI APIs. So, we built something better."
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "The Optimal Philosophy" }
                    p { style: "margin-bottom: 16px;", "Our entire infrastructure is engineered from the ground up using pure Rust and Dioxus. We enforce strict architectural standards to ensure zero latency and a microscopic memory footprint:" }

                    ul {
                        style: "margin-left: 20px; margin-bottom: 24px; list-style-type: disc;",
                        li { style: "margin-bottom: 8px;", strong { "Pure Rust Architecture: " } "No Node.js. No Python microservices. Just maximum async throughput powered by Tokio and Axum." }
                        li { style: "margin-bottom: 8px;", strong { "AVIF-Native Storage: " } "We strictly store our master images in AVIF format using ravif, achieving next-generation compression without sacrificing fidelity." }
                        li { style: "margin-bottom: 8px;", strong { "On-Demand Delivery: " } "Need a JPEG or a specific 4K crop? Our system generates and caches legacy formats from the AVIF master in 20-40 milliseconds on-demand." }
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "Bare-Metal Intelligence" }
                    p { "Our auto-tagging and color extraction don't rely on expensive cloud APIs. We utilize zero-shot CLIP models powered by Hugging Face's Candle crate running natively on our bare metal. We combine this with lightning-fast K-Means mathematical clustering for precise color palette mapping. The result? Insanely fast search indexing with absolute data privacy." }
                }
            }
        }
    }
}

#[component]
pub fn FAQ() -> Element {
    rsx! {
        div {
            class: "container",
            style: "padding: 120px 20px 80px;",
            div {
                class: "legal-content reveal-up show",
                style: "max-width: 800px; margin: 0 auto;",

                h1 {
                    style: "font-size: 48px; font-weight: 900; margin-bottom: 32px;",
                    "Frequently Asked Questions"
                }

                div {
                    class: "glass",
                    style: "padding: 40px; border-radius: 24px; color: var(--text-secondary); line-height: 1.6;",

                    h3 { style: "color: white; margin: 0 0 12px; font-size: 20px;", "How do I upload a wallpaper?" }
                    p { style: "margin-bottom: 24px;", "Click the 'Upload' button in the navigation bar. Your image is immediately processed in memory, discarding any malformed EXIF data, before being securely encoded." }

                    h3 { style: "color: white; margin: 24px 0 12px; font-size: 20px;", "Why does Wallr convert images to AVIF?" }
                    p { style: "margin-bottom: 24px;", "AVIF provides insanely superior compression compared to traditional JPEGs or PNGs. This dramatically cuts down storage costs and gives you lightning-fast load times. Don't worry, if you need a standard JPEG or PNG, our backend generates it seamlessly in milliseconds when you hit download." }

                    h3 { style: "color: white; margin: 24px 0 12px; font-size: 20px;", "How does the AI auto-tagging work?" }
                    p { style: "margin-bottom: 24px;", "We run a quantized Vision Transformer (CLIP) natively on our servers. When you upload, we resize a temporary clone of the image to 224x224 and compare its visual embeddings against hundreds of predefined text categories using cosine similarity. Anything with a high enough match gets tagged instantly." }

                    h3 { style: "color: white; margin: 24px 0 12px; font-size: 20px;", "Are you sending my uploads to ChatGPT or Midjourney?" }
                    p { style: "margin-bottom: 24px;", "Absolutely not. One of our core philosophies is maintaining a 'Zero-Python, Zero-API' pipeline. Your uploads are processed locally on our bare-metal hardware. No third parties ever see your images." }

                    h3 { style: "color: white; margin: 24px 0 12px; font-size: 20px;", "Can I download an exact crop for my ultrawide monitor?" }
                    p { "Yes! Because of our deeply optimized fast_image_resize pipeline, when you request a specific resolution, we decode our AVIF master, resize it using SIMD acceleration, and deliver the exact pixels you need instantly." }
                }
            }
        }
    }
}

#[component]
pub fn ContactUs() -> Element {
    rsx! {
        div {
            class: "container",
            style: "padding: 120px 20px 80px;",
            div {
                class: "legal-content reveal-up show",
                style: "max-width: 800px; margin: 0 auto;",

                h1 {
                    style: "font-size: 48px; font-weight: 900; margin-bottom: 32px;",
                    "Contact Us"
                }

                div {
                    class: "glass",
                    style: "padding: 40px; border-radius: 24px; color: var(--text-secondary); line-height: 1.6;",

                    p {
                        style: "font-size: 18px; margin-bottom: 32px;",
                        "Have questions, feedback, or a business inquiry? Whether you found a bug in our Rust backend or just want to chat about next-gen image compression, we'd love to hear from you."
                    }

                    div {
                        style: "margin-top: 32px; padding-bottom: 24px; border-bottom: 1px solid rgba(255, 255, 255, 0.1);",
                        h4 { style: "color: white; margin-bottom: 8px; font-size: 20px;", "General Support & Inquiries" }
                        p { "For account help, API access, or general questions:" }
                        a {
                            href: "mailto:support@wallr.dev",
                            style: "color: #4a90e2; text-decoration: none; font-weight: bold;",
                            "support@wallr.dev"
                        }
                    }

                    div {
                        style: "margin-top: 24px; padding-bottom: 24px; border-bottom: 1px solid rgba(255, 255, 255, 0.1);",
                        h4 { style: "color: white; margin-bottom: 8px; font-size: 20px;", "Copyright & DMCA" }
                        p { "If you believe your copyrighted work has been uploaded without authorization, please email our legal team with a valid takedown notice:" }
                        a {
                            href: "mailto:legal@wallr.dev",
                            style: "color: #4a90e2; text-decoration: none; font-weight: bold;",
                            "legal@wallr.dev"
                        }
                    }

                    div {
                        style: "margin-top: 24px;",
                        h4 { style: "color: white; margin-bottom: 8px; font-size: 20px;", "Open Source Community" }
                        p { "Interested in how Wallr is built? Join our developer community to talk about Dioxus, Axum, and local AI." }
                        div {
                            style: "display: flex; gap: 16px; margin-top: 12px;",
                            a {
                                href: "#",
                                style: "color: white; text-decoration: underline;",
                                "GitHub"
                            }
                            a {
                                href: "#",
                                style: "color: white; text-decoration: underline;",
                                "Discord Server"
                            }
                        }
                    }
                }
            }
        }
    }
}
