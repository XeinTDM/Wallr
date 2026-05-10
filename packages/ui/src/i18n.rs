use dioxus::prelude::*;

mod en;
mod es;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Language {
    English,
    Spanish,
}

impl Language {
    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Spanish => "Español",
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

pub fn use_i18n() -> I18nContext {
    let lang = use_context::<Signal<Language>>();
    I18nContext { lang }
}

#[derive(Clone, Copy)]
pub struct I18nContext {
    pub lang: Signal<Language>,
}

impl I18nContext {
    pub fn set_lang(&mut self, l: Language) {
        self.lang.set(l);
    }

    pub fn get_lang(&self) -> Language {
        *self.lang.read()
    }

    pub fn t<'a>(&self, key: &'a str) -> &'a str {
        match self.get_lang() {
            Language::English => en::translate(key).unwrap_or(key),
            Language::Spanish => es::translate(key).unwrap_or(key),
        }
    }
}

pub fn init_i18n() {
    use_context_provider(|| Signal::new(Language::English));
}
