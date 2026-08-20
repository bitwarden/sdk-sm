//! Single-pass line detection: provider [`RegexSet`] plus an entropy
//! fallback for assignments that don't match a known provider shape.
//!
//! Nothing in this module ever returns, logs, or panics with the matched
//! text itself — callers only ever see a masked [`mask_preview`] output.

use std::{collections::HashMap, sync::OnceLock};

use regex::{Regex, RegexSet};

use crate::rules::{HIGH_ENTROPY_RULE_ID, RULES, Severity};

/// Longer lines are almost always minified/bundled assets, not
/// hand-written config. Skipping them keeps worst-case per-line cost
/// bounded and avoids reporting an unhelpful masked preview into a wall of
/// text.
pub const MAX_LINE_LEN: usize = 2000;

/// Minimum length of the quoted literal considered for the entropy
/// fallback (plan §1.2 / task spec: "quoted string literal >= 20 chars").
const ENTROPY_MIN_LITERAL_LEN: usize = 20;

/// Shannon-entropy threshold (bits/char) above which a key-like assignment
/// is treated as a likely secret.
const ENTROPY_THRESHOLD: f64 = 3.7;

/// Total length, in characters, of a masked preview.
const PREVIEW_MAX_LEN: usize = 20;

/// Number of leading characters of a match kept visible in a preview.
const PREVIEW_VISIBLE_LEN: usize = 4;

struct CompiledRules {
    set: RegexSet,
    regexes: Vec<Regex>,
    entropy_assignment: Regex,
}

fn compiled() -> &'static CompiledRules {
    static COMPILED: OnceLock<CompiledRules> = OnceLock::new();
    COMPILED.get_or_init(|| {
        let patterns: Vec<&str> = RULES.iter().map(|r| r.pattern).collect();
        let set = RegexSet::new(&patterns)
            .expect("RULES patterns are a fixed, test-covered set (see rules.rs) and always compile");
        let regexes = RULES
            .iter()
            .map(|r| {
                Regex::new(r.pattern).unwrap_or_else(|e| {
                    panic!("rule `{}` pattern failed to compile: {e}", r.id);
                })
            })
            .collect();
        // Matches `<identifier containing key|token|secret|password|credential>
        // <opt suffix> [:=] "<literal, no whitespace>"` or the single-quoted
        // equivalent. Only the identifier and the literal's *shape* are used;
        // the literal's content is scored for entropy, never logged.
        let entropy_assignment = Regex::new(
            r#"(?i)(?:key|token|secret|passw(?:or)?d|credential)[A-Za-z0-9_-]*\s*[:=]\s*(?:"(?P<dq>[^"\s]{20,})"|'(?P<sq>[^'\s]{20,})')"#,
        )
        .expect("entropy assignment pattern is a fixed literal and always compiles");
        CompiledRules { set, regexes, entropy_assignment }
    })
}

/// One detection on a single line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedMatch {
    pub rule_id: &'static str,
    pub severity: Severity,
    /// 1-based column, counted in `char`s (not bytes) from the start of the
    /// line, respecting UTF-8 boundaries.
    pub column: u64,
    /// Masked preview of the matched text. Never the full value.
    pub preview: String,
}

/// Scan a single line of text against every provider rule, then (only if no
/// provider rule fired) against the entropy fallback.
///
/// Lines longer than [`MAX_LINE_LEN`] are treated as minified/bundled
/// content and skipped entirely.
pub fn scan_line(line: &str) -> Vec<DetectedMatch> {
    if line.chars().count() > MAX_LINE_LEN {
        return Vec::new();
    }

    let compiled = compiled();
    let mut matches = Vec::new();

    let hit_indices: Vec<usize> = compiled.set.matches(line).into_iter().collect();
    for idx in &hit_indices {
        let Some(rule) = RULES.get(*idx) else {
            continue;
        };
        let Some(regex) = compiled.regexes.get(*idx) else {
            continue;
        };
        let Some(m) = regex.find(line) else {
            continue;
        };
        matches.push(DetectedMatch {
            rule_id: rule.id,
            severity: rule.severity,
            column: char_column(line, m.start()),
            preview: mask_preview(m.as_str()),
        });
    }

    if hit_indices.is_empty()
        && let Some(caps) = compiled.entropy_assignment.captures(line)
        && let Some(literal) = caps.name("dq").or_else(|| caps.name("sq"))
    {
        let text = literal.as_str();
        if text.chars().count() >= ENTROPY_MIN_LITERAL_LEN
            && shannon_entropy(text) >= ENTROPY_THRESHOLD
        {
            matches.push(DetectedMatch {
                rule_id: HIGH_ENTROPY_RULE_ID,
                severity: Severity::Medium,
                column: char_column(line, literal.start()),
                preview: mask_preview(text),
            });
        }
    }

    matches
}

/// Convert a byte offset (as returned by `regex`, always on a char
/// boundary) into a 1-based character column.
fn char_column(line: &str, byte_offset: usize) -> u64 {
    let prefix_chars = line
        .get(..byte_offset)
        .map(|s| s.chars().count())
        .unwrap_or(0);
    (prefix_chars + 1) as u64
}

/// Mask a matched value for safe inclusion in a findings artifact: the
/// first [`PREVIEW_VISIBLE_LEN`] characters, then asterisks for the rest,
/// the whole thing capped at [`PREVIEW_MAX_LEN`] characters. Always
/// respects UTF-8 character boundaries — never the full value.
pub fn mask_preview(matched: &str) -> String {
    let total_chars = matched.chars().count();
    let visible_len = PREVIEW_VISIBLE_LEN.min(total_chars);
    let visible: String = matched.chars().take(visible_len).collect();
    let remaining = total_chars - visible_len;
    let asterisks_len = remaining.min(PREVIEW_MAX_LEN.saturating_sub(visible_len));

    let mut preview = visible;
    for _ in 0..asterisks_len {
        preview.push('*');
    }
    preview
}

/// Shannon entropy of `s`, in bits per character.
fn shannon_entropy(s: &str) -> f64 {
    let len = s.chars().count();
    if len == 0 {
        return 0.0;
    }
    let mut freq: HashMap<char, usize> = HashMap::new();
    for c in s.chars() {
        *freq.entry(c).or_insert(0) += 1;
    }
    freq.values().fold(0.0, |acc, &count| {
        let p = f64::from(count as u32) / f64::from(len as u32);
        acc - p * p.log2()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_never_returns_full_short_value() {
        let preview = mask_preview("AKIA1234");
        assert_eq!(preview, "AKIA****");
        assert!(!preview.contains("1234"));
    }

    #[test]
    fn mask_caps_total_length_at_twenty() {
        let long = "A".repeat(100);
        let preview = mask_preview(&long);
        let chars: Vec<char> = preview.chars().collect();
        assert_eq!(chars.len(), PREVIEW_MAX_LEN);
        assert_eq!(&chars[..4], &['A', 'A', 'A', 'A']);
        assert!(chars[4..].iter().all(|&c| c == '*'));
    }

    #[test]
    fn mask_respects_multibyte_boundaries() {
        // 4-byte emoji characters; slicing by byte index would panic here.
        // 8 emoji + 19 ASCII chars = 27 chars total, comfortably over
        // PREVIEW_MAX_LEN so the cap (not the input length) is exercised.
        let value = "🔑🔑🔑🔑🔑🔑🔑🔑secretvalue1234567";
        let preview = mask_preview(value);
        assert_eq!(preview.chars().count(), PREVIEW_MAX_LEN);
        assert!(preview.starts_with("🔑🔑🔑🔑"));
    }

    #[test]
    fn short_match_has_no_asterisks() {
        assert_eq!(mask_preview("abc"), "abc");
    }

    #[test]
    fn char_column_counts_chars_not_bytes() {
        let line = "🔑🔑key=\"AKIAIOSFODNN7EXAMPLE\"";
        let byte_offset = line
            .find("AKIA")
            .unwrap_or_else(|| panic!("fixture must contain AKIA"));
        // Two 4-byte emoji + `key="` = 2 chars + 5 chars = 7 chars before the match.
        assert_eq!(char_column(line, byte_offset), 8);
    }

    #[test]
    fn entropy_fallback_does_not_fire_when_a_provider_rule_already_matched() {
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        let line = format!("aws_secret_token = \"{aws_example_key}\"");
        let matches = scan_line(&line);
        assert!(matches.iter().any(|m| m.rule_id == "aws-access-key-id"));
        assert!(!matches.iter().any(|m| m.rule_id == HIGH_ENTROPY_RULE_ID));
    }

    #[test]
    fn entropy_fallback_fires_on_high_entropy_quoted_assignment() {
        let literal: String = "Zx9!qLp2Kv7mWs4Rt8Nb1Yc6".chars().collect();
        let line = format!("api_secret = \"{literal}\"");
        let matches = scan_line(&line);
        assert!(matches.iter().any(|m| m.rule_id == HIGH_ENTROPY_RULE_ID));
    }

    #[test]
    fn entropy_fallback_silent_on_low_entropy_placeholder() {
        let line = "api_key = \"your-key-here-your-key-here\"";
        let matches = scan_line(line);
        assert!(
            matches.is_empty(),
            "placeholder text should not trip the entropy fallback: {matches:?}"
        );
    }

    #[test]
    fn lines_over_max_len_are_skipped() {
        let aws_example_key = "AKIA".to_string() + "IOSFODNN7EXAMPLE";
        let line = " ".repeat(MAX_LINE_LEN + 1) + &aws_example_key;
        assert!(scan_line(&line).is_empty());
    }
}

/// True-positive corpus: one synthetic, constructed-not-literal sample per
/// rule, each asserted to fire exactly its own rule and nothing else.
///
/// Samples are built by concatenation/repetition at runtime rather than
/// checked in as fixture literals, so nothing here resembles a real
/// checked-in credential (the AWS sample is the exception: it's the
/// official, publicly documented `AKIAIOSFODNN7EXAMPLE` example value).
#[cfg(test)]
mod true_positive_corpus {
    use super::*;
    use crate::rules::RULES;

    /// Repeat `charset` to build a string of exactly `len` characters.
    fn cycled(charset: &str, len: usize) -> String {
        charset.chars().cycle().take(len).collect()
    }

    fn sample_for_rule(id: &str) -> String {
        const ALNUM: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        const HEX: &str = "0123456789abcdef";
        const B64URL: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_";
        const B64: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/";

        match id {
            "aws-access-key-id" => {
                // Officially documented AWS example value, not a real key.
                format!("aws_key = \"{}\"", "AKIA".to_string() + "IOSFODNN7EXAMPLE")
            }
            "github-token-classic" => format!("token = \"ghp_{}\"", cycled(ALNUM, 36)),
            "github-fine-grained-pat" => format!("token = \"github_pat_{}\"", cycled(ALNUM, 24)),
            "gitlab-pat" => format!("token = \"glpat-{}\"", cycled(ALNUM, 20)),
            "slack-token" => format!("token = \"xoxb-{}\"", cycled("0123456789-", 12)),
            "slack-webhook-url" => {
                format!(
                    "webhook = \"https://hooks.slack.com/services/{}/{}/{}\"",
                    cycled(ALNUM, 9),
                    cycled(ALNUM, 9),
                    cycled(ALNUM, 24)
                )
            }
            "stripe-live-secret-key" => format!("stripe = \"sk_live_{}\"", cycled(ALNUM, 24)),
            "stripe-live-restricted-key" => format!("stripe = \"rk_live_{}\"", cycled(ALNUM, 24)),
            "google-api-key" => format!("google = \"AIza{}\"", cycled(B64URL, 35)),
            "gcp-service-account-private-key" => "-----BEGIN PRIVATE KEY-----".to_string(),
            "pem-rsa-private-key" => "-----BEGIN RSA PRIVATE KEY-----".to_string(),
            "pem-ec-private-key" => "-----BEGIN EC PRIVATE KEY-----".to_string(),
            "pem-openssh-private-key" => "-----BEGIN OPENSSH PRIVATE KEY-----".to_string(),
            "pem-pgp-private-key" => "-----BEGIN PGP PRIVATE KEY BLOCK-----".to_string(),
            "openai-api-key" => format!("openai = \"sk-proj-{}\"", cycled(ALNUM, 40)),
            "anthropic-api-key" => format!("anthropic = \"sk-ant-{}\"", cycled(ALNUM, 30)),
            "npm-access-token" => format!("npm = \"npm_{}\"", cycled(ALNUM, 36)),
            "pypi-api-token" => format!("pypi = \"pypi-AgEIcHlwaS5vcmc{}\"", cycled(ALNUM, 24)),
            "sendgrid-api-key" => {
                format!(
                    "sendgrid = \"SG.{}.{}\"",
                    cycled(ALNUM, 22),
                    cycled(ALNUM, 43)
                )
            }
            "twilio-api-key" => format!("twilio = \"SK{}\"", cycled(HEX, 32)),
            "digitalocean-token" => format!("do_token = \"dop_v1_{}\"", cycled(HEX, 64)),
            "shopify-token" => format!("shopify = \"shpat_{}\"", cycled(HEX, 32)),
            "discord-bot-token" => {
                format!(
                    "discord = \"M{}.{}.{}\"",
                    cycled(ALNUM, 24),
                    cycled(ALNUM, 6),
                    cycled(ALNUM, 27)
                )
            }
            "telegram-bot-token" => format!("telegram = \"123456789:AA{}\"", cycled(ALNUM, 33)),
            "huggingface-token" => format!("hf = \"hf_{}\"", cycled(ALNUM, 34)),
            "databricks-token" => format!("databricks = \"dapi{}\"", cycled(HEX, 32)),
            "postman-api-key" => format!("postman = \"PMAK-{}\"", cycled(HEX, 24)),
            "jwt" => {
                format!(
                    "jwt = \"eyJ{}.eyJ{}.{}\"",
                    cycled(B64URL, 20),
                    cycled(B64URL, 20),
                    cycled(B64URL, 20)
                )
            }
            "bitwarden-sm-access-token" => {
                format!(
                    "token = \"0.ec2c1d46-6a4b-4751-a310-af9601317f2d.{}:{}==\"",
                    cycled(ALNUM, 30),
                    cycled(B64, 22)
                )
            }
            "azure-storage-account-key" => {
                format!("conn = \"AccountKey={}==\"", cycled(B64, 86))
            }
            "generic-bearer-token" => format!("Authorization: Bearer {}", cycled(ALNUM, 30)),
            other => panic!("no true-positive sample authored for rule `{other}` — add one above"),
        }
    }

    #[test]
    fn every_rule_fires_on_its_own_synthetic_sample() {
        for rule in RULES {
            let sample = sample_for_rule(rule.id);
            let matches = scan_line(&sample);
            assert!(
                matches.iter().any(|m| m.rule_id == rule.id),
                "rule `{}` did not fire on its own synthetic sample: {sample:?} (got {matches:?})",
                rule.id
            );
            // Masking correctness itself is covered by the `mask_*` tests
            // above; here we only need every preview to be shorter than the
            // sample line, i.e. not simply "the whole matched text".
            for m in &matches {
                assert!(
                    m.preview.chars().count() <= PREVIEW_MAX_LEN,
                    "preview for rule `{}` exceeds the {} char cap: {:?}",
                    m.rule_id,
                    PREVIEW_MAX_LEN,
                    m.preview
                );
            }
        }
    }

    #[test]
    fn every_rule_fires_exactly_once_on_its_own_sample() {
        // A stronger check than `every_rule_fires_on_its_own_synthetic_sample`:
        // no *other* provider rule should also fire on the same line. A
        // handful of rules are intentionally permissive (e.g. the generic
        // bearer-token and JWT shapes can nest inside other tokens), so this
        // is a guardrail, not an absolute invariant — failures here are worth
        // a look, not an automatic bug.
        for rule in RULES {
            let sample = sample_for_rule(rule.id);
            let matches = scan_line(&sample);
            let rule_ids: Vec<&str> = matches.iter().map(|m| m.rule_id).collect();
            assert_eq!(
                rule_ids,
                vec![rule.id],
                "expected only `{}` to fire on its own sample, got {rule_ids:?}",
                rule.id
            );
        }
    }
}

/// False-positive corpus: content that looks credential-adjacent but must
/// stay silent across every rule, including the entropy fallback.
#[cfg(test)]
mod false_positive_corpus {
    use super::*;

    fn assert_silent(line: &str) {
        let matches = scan_line(line);
        assert!(
            matches.is_empty(),
            "expected {line:?} to be silent, got {matches:?}"
        );
    }

    #[test]
    fn random_uuid_is_silent() {
        assert_silent("resource_id = \"e6e6e6e6-1234-4abc-8abc-abcdefabcdef\"");
    }

    #[test]
    fn forty_char_git_sha_is_silent() {
        let sha: String = "0123456789abcdef".chars().cycle().take(40).collect();
        assert_silent(&format!("commit = \"{sha}\""));
    }

    #[test]
    fn generic_base64_blob_in_prose_is_silent() {
        assert_silent(
            "See the encoded blob VGhpcyBpcyBqdXN0IGV4YW1wbGUgdGV4dCBub3QgYSBzZWNyZXQ= in the appendix.",
        );
    }

    #[test]
    fn package_lock_integrity_hash_line_is_silent() {
        let hash: String = "abcdef1234567890ABCDEFabcdefABCDEFabcdefABCDEFabcdefABCDEFabcdef"
            .chars()
            .collect();
        assert_silent(&format!("      \"integrity\": \"sha512-{hash}==\","));
    }

    #[test]
    fn env_example_placeholder_is_silent() {
        assert_silent("API_KEY=your-key-here");
        assert_silent("API_KEY=\"your-key-here\"");
    }

    #[test]
    fn plain_url_is_silent() {
        assert_silent("See https://example.com/docs/getting-started for details.");
    }
}
