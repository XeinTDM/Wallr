use dioxus::prelude::*;

#[component]
pub fn TermsOfService() -> Element {
    rsx! {
        div {
            class: "container",
            style: "padding: 120px 20px 80px;",
            div {
                class: "legal-content reveal-up show",
                style: "max-width: 800px; margin: 0 auto;",

                h1 {
                    style: "font-size: 48px; font-weight: 900; margin-bottom: 32px;",
                    "Terms & Conditions"
                }

                div {
                    class: "glass",
                    style: "padding: 40px; border-radius: 24px; color: var(--text-secondary); line-height: 1.6;",

                    p {
                        style: "margin-bottom: 24px; font-weight: bold;",
                        "Last Updated: May 9, 2026"
                    }

                    p {
                        style: "margin-bottom: 32px;",
                        "Welcome to Wallr. By accessing or using our platform, you agree to be bound by these Terms & Conditions. If you do not agree to all of these terms, do not use the service."
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "1. Usage & Account" }
                    p { style: "margin-bottom: 24px;", "Wallr is provided for discovering, sharing, and downloading high-quality wallpapers. You must be at least 13 years of age to create an account. You are strictly responsible for maintaining the security of your account and for all activities that occur under your username." }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "2. Content Uploads & Automated Processing" }
                    p { "By uploading images to Wallr, you grant us the necessary rights to host, process, and display your content. Because our platform enforces strict architectural standards for performance, you acknowledge and agree that:" }
                    ul {
                        style: "margin-left: 20px; margin-bottom: 24px; list-style-type: disc;",
                        li { style: "margin-bottom: 8px;", strong { "Format Conversion: " } "Original uploads are permanently converted to AVIF format. Original files (and their metadata/EXIF data) are discarded upon successful ingestion." }
                        li { style: "margin-bottom: 8px;", strong { "Automated AI Tagging: " } "Your uploads will be scanned by our local, bare-metal AI models to automatically assign categories and extract color profiles. We reserve the right to override or remove inaccurate auto-generated tags." }
                        li { style: "margin-bottom: 8px;", strong { "On-Demand Delivery: " } "We may dynamically generate, resize, and cache legacy formats (JPEG/PNG) from your original upload to serve to other users." }
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "3. Copyright & Intellectual Property" }
                    p { style: "margin-bottom: 24px;", "You must own or hold the explicit right to distribute any content you upload. Wallr strictly respects intellectual property rights and will immediately remove content upon receiving valid takedown notices. Repeat infringers will have their accounts permanently terminated." }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "4. Prohibited Conduct & Community Standards" }
                    p { "To maintain the safety and stability of our community and backend systems, you agree not to:" }
                    ul {
                        style: "margin-left: 20px; margin-bottom: 24px; list-style-type: disc;",
                        li { style: "margin-bottom: 8px;", "Upload illicit, illegal, explicit, or inherently abusive material." }
                        li { style: "margin-bottom: 8px;", "Post comments containing racial slurs, extreme profanity, or encouragement of self-harm. We employ a strict, automated local moderation engine and operate a zero-tolerance policy for such behavior." }
                        li { style: "margin-bottom: 8px;", "Spam comments or maliciously hammer our endpoints (e.g., to exhaust server CPU or bypass rate limits)." }
                        li { style: "margin-bottom: 8px;", "Attempt to reverse-engineer, exploit, or bypass any of our platform's security, moderation, or rate-limiting measures." }
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "5. Disclaimer of Warranties" }
                    p { style: "margin-bottom: 24px;", "While Wallr is engineered for maximum performance and stability, the service is provided on an \"AS IS\" and \"AS AVAILABLE\" basis. We do not guarantee uninterrupted uptime, flawless automated tagging, or that your uploads will be preserved indefinitely. We strongly recommend keeping backups of your original images." }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "6. Termination" }
                    p { "We reserve the absolute right to suspend or terminate your account, and to remove any content you have uploaded, at our sole discretion and without prior notice, for any conduct that violates these Terms of Service." }
                }
            }
        }
    }
}

#[component]
pub fn PrivacyPolicy() -> Element {
    rsx! {
        div {
            class: "container",
            style: "padding: 120px 20px 80px;",
            div {
                class: "legal-content reveal-up show",
                style: "max-width: 800px; margin: 0 auto;",

                h1 {
                    style: "font-size: 48px; font-weight: 900; margin-bottom: 32px;",
                    "Privacy Policy"
                }

                div {
                    class: "glass",
                    style: "padding: 40px; border-radius: 24px; color: var(--text-secondary); line-height: 1.6;",

                    p {
                        style: "margin-bottom: 24px; font-weight: bold;",
                        "Last Updated: May 9, 2026"
                    }

                    p {
                        style: "margin-bottom: 32px;",
                        "Welcome to our platform. Your privacy is paramount. Because our platform is built from the ground up on a highly optimized, single-server architecture, we have complete control over how your data is processed. This policy explains our data practices."
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "1. Information We Collect" }
                    p { "We collect the absolute minimum amount of data required to provide a blazing-fast wallpaper experience:" }
                    ul {
                        style: "margin-left: 20px; margin-bottom: 24px; list-style-type: disc;",
                        li { style: "margin-bottom: 8px;", strong { "Account Profile: " } "If you create an account, we collect your username, email, and credentials. Optionally, you may provide a public bio and external social links." }
                        li { style: "margin-bottom: 8px;", strong { "User Content: " } "The wallpapers, public collections, and comments you voluntarily post on our platform." }
                        li { style: "margin-bottom: 8px;", strong { "Usage Data: " } "Standard server connection logs (such as IP address and browser type) to detect abuse and maintain security." }
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "2. AI Processing & Utmost Privacy" }
                    p { "Our platform relies heavily on AI for auto-tagging and color extraction, but we do it differently than the rest of the industry:" }
                    ul {
                        style: "margin-left: 20px; margin-bottom: 24px; list-style-type: disc;",
                        li {
                            style: "margin-bottom: 8px;",
                            strong { "100% Local AI & Moderation: " }
                            "When you upload an image or post a comment, the auto-tagging process and text moderation are done entirely on bare metal using localized models and algorithms. ",
                            span { style: "color: white; font-weight: bold;", "Your images and private thoughts are NEVER sent to third-party AI APIs, cloud providers, or data scrapers." }
                        }
                        li { style: "margin-bottom: 8px;", strong { "Mathematical Extraction: " } "Color palettes and text tokens are analyzed locally, ensuring your data never leaves our servers." }
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "3. Data Storage & Public Visibility" }
                    p { "To ensure fast delivery and clarity on what is public, here is how we handle your data:" }
                    ul {
                        style: "margin-left: 20px; margin-bottom: 24px; list-style-type: disc;",
                        li { style: "margin-bottom: 8px;", "Original uploads are permanently converted to AVIF format for secure, low-footprint object storage. Legacy formats are generated on-demand." }
                        li { style: "margin-bottom: 8px;", "Any item explicitly marked as 'Public' (such as public wallpapers, collections, or comments) is visible to anyone on the internet and may be indexed by search engines." }
                        li { style: "margin-bottom: 8px;", "Your public profile, including your bio and social links, is visible to all visitors." }
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "4. Third-Party Sharing" }
                    p { style: "margin-bottom: 24px;", "We do not sell your personal data or uploaded images to third parties. Data is only shared with infrastructure partners (e.g., our server hosts or database providers) strictly to run the service, or when required by legal mandates." }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "5. Security Measures" }
                    p { style: "margin-bottom: 24px;", "By designing our backend in pure Rust with zero external microservices, we dramatically reduce our attack surface. All traffic is encrypted in transit via SSL/TLS, and your passwords are cryptographically hashed before hitting our database." }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "6. Your Rights & Control" }
                    p { style: "margin-bottom: 24px;", "You retain full ownership of your data. At any point, you can:" }
                    ul {
                        style: "margin-left: 20px; margin-bottom: 24px; list-style-type: disc;",
                        li { style: "margin-bottom: 8px;", "Delete any wallpaper you have uploaded." }
                        li { style: "margin-bottom: 8px;", "Permanently delete your account and all associated metadata." }
                        li { style: "margin-bottom: 8px;", "Request an export of the data we hold regarding your account." }
                    }

                    h2 { style: "color: white; margin: 32px 0 16px; font-size: 24px;", "7. Contact Us" }
                    p { "If you have any questions or concerns regarding this Privacy Policy, our use of local AI, or our data practices, please reach out to our support team." }
                }
            }
        }
    }
}
