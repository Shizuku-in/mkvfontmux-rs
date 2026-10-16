use serde::{Deserialize, Serialize};

/// Keyword rules mapping subtitle file names to track language tags.
///
/// Rules are tried in order; the first whose keyword matches wins. ASCII
/// keywords must equal a whole token of the file stem (split on spaces and
/// `.-_[](){}+&`); other keywords, such as `简体`, match as substrings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LanguageRules {
    pub rules: Vec<LanguageRule>,
    /// Tag used when no rule matches.
    pub fallback: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageRule {
    /// BCP 47 language tag, for example `zh-Hans`.
    pub code: String,
    pub keywords: Vec<String>,
}

impl Default for LanguageRules {
    fn default() -> Self {
        let rule = |code: &str, keywords: &[&str]| LanguageRule {
            code: code.into(),
            keywords: keywords.iter().map(|k| (*k).into()).collect(),
        };
        Self {
            // Traditional first: names like "zh-tw" would otherwise never be reached
            // when they also contain a simplified keyword.
            rules: vec![
                rule(
                    "zh-Hant",
                    &[
                        "tc", "cht", "zht", "zh-tw", "zh-hk", "big5", "繁体", "繁中", "繁體",
                    ],
                ),
                rule(
                    "zh-Hans",
                    &[
                        "sc", "chs", "zhs", "zh-cn", "gb", "gbk", "gb2312", "简体", "简中",
                    ],
                ),
                rule("en", &["en", "eng", "english", "英文", "英字"]),
                rule(
                    "ja",
                    &["jp", "ja", "jpn", "japanese", "日文", "日语", "日字"],
                ),
            ],
            fallback: "zh-Hans".into(),
        }
    }
}

impl LanguageRules {
    /// Language tag for a subtitle file name.
    pub fn detect(&self, file_name: &str) -> &str {
        let stem = file_name
            .rsplit_once('.')
            .map_or(file_name, |(stem, _)| stem)
            .to_lowercase();
        let tokens: Vec<&str> = stem
            .split([' ', '.', '-', '_', '[', ']', '(', ')', '{', '}', '+', '&'])
            .filter(|t| !t.is_empty())
            .collect();
        for rule in &self.rules {
            let hit = rule.keywords.iter().any(|keyword| {
                let keyword = keyword.to_lowercase();
                if keyword.is_empty() {
                    false
                } else if keyword
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                {
                    // Hyphenated keywords such as "zh-cn" span two tokens.
                    tokens.contains(&keyword.as_str()) || hyphenated_match(&tokens, &keyword)
                } else {
                    stem.contains(&keyword)
                }
            });
            if hit {
                return &rule.code;
            }
        }
        &self.fallback
    }
}

fn hyphenated_match(tokens: &[&str], keyword: &str) -> bool {
    let parts: Vec<&str> = keyword.split(['-', '_']).collect();
    parts.len() > 1 && tokens.windows(parts.len()).any(|w| w == parts.as_slice())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_common_names() {
        let rules = LanguageRules::default();
        assert_eq!(rules.detect("[VCB] Show [01].sc.ass"), "zh-Hans");
        assert_eq!(rules.detect("[VCB] Show [01].tc.ass"), "zh-Hant");
        assert_eq!(rules.detect("Show 01.zh-tw.ass"), "zh-Hant");
        assert_eq!(rules.detect("Show 01.简体中文.ass"), "zh-Hans");
        assert_eq!(rules.detect("Show 01.jpn.ass"), "ja");
        assert_eq!(rules.detect("Show 01.ass"), "zh-Hans");
        // "Scene" must not match the "sc" keyword.
        assert_eq!(rules.detect("Scene 01.eng.ass"), "en");
    }
}
