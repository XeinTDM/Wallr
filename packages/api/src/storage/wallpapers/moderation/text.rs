use decancer::cure;
use regex::Regex;
use std::sync::OnceLock;
use strsim::normalized_levenshtein;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

static AGE_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_age_regex() -> &'static Regex {
    AGE_REGEX.get_or_init(|| {
        Regex::new(r"\b([1-9]|1[0-7])\s*(yo|yr|years?|old|歳|才)\b|\b\d{1,2}\s*(months?|mos?|mths?|ヶ月|カ月)\b").unwrap()
    })
}

pub fn check_predatory_metadata(title: &str, tags: &[String]) -> Option<&'static str> {
    let combined = format!("{} {}", title, tags.join(" "));
    let normalized = normalize_text(&combined);
    let collapsed = collapse_repeats(&normalized);

    let words: Vec<String> = normalized
        .unicode_words()
        .map(|w| w.to_lowercase())
        .collect();

    let high_confidence = [
        "cheesepizza",
        "childporn",
        "jailbait",
        "nudeteen",
        "nakedteen",
        "underageporn",
        "lolicon",
        "shotacon",
        "rorikon",
        "ロリコン",
    ];

    for kw in high_confidence {
        if collapsed.contains(kw) {
            return Some(kw);
        }
    }

    let mut score = 0i32;
    let mut triggers = vec![];

    let suspicious = [
        ("loli", 8),
        ("shota", 8),
        ("pedo", 7),
        ("csem", 9),
        ("cp", 3),
        ("teen", 3),
        ("young", 3),
        ("schoolgirl", 6),
        ("tiny", 4),
        ("fresh", 4),
        ("lolicon", 9),
        ("shotacon", 9),
        ("ロリ", 9),
        ("ショタ", 9),
        ("少女", 5),
        ("lolikon", 8),
    ];

    for (term, weight) in &suspicious {
        if collapsed.contains(term) || words.iter().any(|w| w == term) {
            score += weight;
            triggers.push(*term);
        }
    }

    let fuzzy_terms = ["loli", "shota", "pedo", "teen", "minor", "lolicon"];
    for term in fuzzy_terms {
        for word in &words {
            if normalized_levenshtein(word, term) > 0.82 {
                score += 6;
                triggers.push(term);
                break;
            }
        }
    }

    if contains_cp_with_context(&words, &collapsed) {
        score += 8;
    }

    if get_age_regex().is_match(&normalized) {
        score += 7;
    }

    let separator_patterns = ["l o l i", "s h o t a", "p e d o", "c p"];
    for pat in separator_patterns {
        if collapsed.contains(pat) {
            score += 7;
        }
    }

    if score >= 14 {
        return Some("high risk combination");
    }
    if score >= 9 && !triggers.is_empty() {
        return Some(triggers[0]);
    }

    None
}

fn normalize_text(text: &str) -> String {
    let cured = cure(text, decancer::Options::default())
        .map(|c| c.to_string())
        .unwrap_or_else(|_| text.to_string())
        .to_lowercase();

    let mut cleaned = String::with_capacity(cured.len());
    for c in cured.nfkc() {
        if c.is_control()
            || ['\u{200B}', '\u{200C}', '\u{200D}', '\u{2060}', '\u{FEFF}'].contains(&c)
        {
            continue;
        }

        let mapped = match c {
            '0' => 'o',
            '1' | '!' => 'i',
            '3' => 'e',
            '4' | '@' => 'a',
            '5' | '$' => 's',
            '+' => 't',
            _ => c,
        };

        if mapped.is_whitespace() {
            cleaned.push(' ');
        } else if "!@#$%^&*()_+-=[]{}|;':\",./<>?~`".contains(mapped) {
        } else {
            cleaned.push(mapped);
        }
    }

    cleaned
}

fn collapse_repeats(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev = '\0';
    let mut count = 0u8;

    for c in s.chars() {
        if c == prev {
            count = count.saturating_add(1);
            if count <= 2 {
                result.push(c);
            }
        } else {
            result.push(c);
            prev = c;
            count = 1;
        }
    }
    result
}

fn contains_cp_with_context(words: &[String], collapsed: &str) -> bool {
    let has_cp = words.iter().any(|w| w == "cp") || collapsed.contains("c p");
    if !has_cp {
        return false;
    }
    words.iter().any(|w| {
        matches!(
            w.as_str(),
            "teen"
                | "young"
                | "girl"
                | "child"
                | "nude"
                | "naked"
                | "loli"
                | "shota"
                | "school"
                | "minor"
        )
    })
}
