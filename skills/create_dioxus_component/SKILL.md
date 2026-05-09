# Create Dioxus Component

Use this skill when tasked to create a new UI component for the Wallr Dioxus frontend.

## Gotchas

- **Dioxus 0.7 API ONLY**: Do NOT use `cx`, `Scope`, or `use_state`. Dioxus 0.7 relies on `#[component]`, `Signal`, and `use_signal`.
- **Props**: Props must implement `PartialEq` and `Clone`. Use `String` and `Vec<T>` instead of `&str` or `&[T]`.
- **Aesthetics**: Wallr requires a "premium, glassmorphic" aesthetic. Do not use generic ad-hoc styles. Ensure the component has responsive styles and micro-animations for hover states defined in its CSS file.
- **Asset Linking**: The stylesheet must be injected in the `.rs` file. Define the CSS path relative to the `assets` folder using the `asset!()` macro: `const STYLES: Asset = asset!("/assets/styling/component_name.css");`. Note: although the physical file is at `packages/ui/assets/styling/component_name.css`, the asset path must use `/assets/...` as configured in the Web package.
- **Icons**: Use `lucide-dioxus` for icons. Do NOT use emojis or SVGs directly.

## Workflow

Follow this checklist to create a new component:

Progress:
- [ ] Step 1: Create the CSS file in `packages/ui/assets/styling/<name>.css`.
- [ ] Step 2: Create the Rust file in `packages/ui/src/<name>.rs`.
- [ ] Step 3: Inject the CSS asset into the component using `document::Stylesheet` inside the `rsx!`.
- [ ] Step 4: Export the component by adding `pub mod <name>;` to `packages/ui/src/lib.rs` (or `mod.rs`).
- [ ] Step 5: Verify the component uses `lucide-dioxus` for icons and has smooth `:hover` transitions in the CSS.

## Template

### Rust (`packages/ui/src/my_component.rs`)

```rust
use dioxus::prelude::*;
use lucide_dioxus::Home; // Example icon

const MY_COMPONENT_CSS: Asset = asset!("/assets/styling/my_component.css");

#[derive(PartialEq, Clone, Props)]
pub struct MyComponentProps {
    pub title: String,
}

#[component]
pub fn MyComponent(props: MyComponentProps) -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        document::Stylesheet { href: MY_COMPONENT_CSS }
        
        div { class: "my-component glass",
            h2 { "{props.title}" }
            button {
                class: "glass-button",
                onclick: move |_| count += 1,
                Home { size: 18 }
                "Clicked {count} times"
            }
        }
    }
}
```

### CSS (`packages/ui/assets/styling/my_component.css`)

```css
.my-component {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 24px;
    background: rgba(255, 255, 255, 0.05);
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 16px;
    transition: all 0.3s ease;
}

.my-component:hover {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
}

.glass-button {
    display: flex;
    align-items: center;
    gap: 8px;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    border-radius: 8px;
    color: white;
    padding: 10px 16px;
    cursor: pointer;
    transition: all 0.2s ease;
}

.glass-button:hover {
    background: rgba(255, 255, 255, 0.2);
    transform: translateY(-2px);
}
```
