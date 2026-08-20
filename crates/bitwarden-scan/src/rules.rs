//! Embedded, hand-authored provider rules.
//!
//! Every rule is a single high-precision regex matched against one line of
//! text at a time (see [`crate::detect`]). Rules never see, log, or return
//! the value they matched — only the caller-provided line is used to compute
//! a location and a masked preview.

use serde::{Deserialize, Serialize};

/// How confident a finding is, roughly proportional to how much damage a
/// leaked value of this shape can do and how unlikely the pattern is to
/// appear outside a real credential.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Medium,
    High,
}

/// One provider rule: a stable id, a human description, a severity, and the
/// regex pattern that identifies it.
pub struct Rule {
    /// Stable, kebab-case identifier. Part of the fingerprint, so it must
    /// never change once shipped — renaming a rule silently invalidates
    /// every existing `.bitwardenignore` entry that suppresses it.
    pub id: &'static str,
    pub description: &'static str,
    pub severity: Severity,
    pub pattern: &'static str,
}

/// Rule id for the entropy fallback (see [`crate::detect::scan_line`]). Not
/// part of [`RULES`] because it is not a fixed-pattern regex rule — it is
/// driven by an assignment regex plus a Shannon-entropy threshold.
pub const HIGH_ENTROPY_RULE_ID: &str = "high-entropy-string";

/// The provider ruleset, compiled once into a single [`regex::RegexSet`] by
/// [`crate::detect`].
pub static RULES: &[Rule] = &[
    Rule {
        id: "aws-access-key-id",
        description: "AWS access key ID",
        severity: Severity::High,
        pattern: r"\b(?:AKIA|ASIA)[0-9A-Z]{16}\b",
    },
    Rule {
        id: "github-token-classic",
        description: "GitHub classic personal/app/OAuth token",
        severity: Severity::High,
        pattern: r"\bgh[oprsu]_[A-Za-z0-9]{36,}\b",
    },
    Rule {
        id: "github-fine-grained-pat",
        description: "GitHub fine-grained personal access token",
        severity: Severity::High,
        pattern: r"\bgithub_pat_[A-Za-z0-9_]{20,}\b",
    },
    Rule {
        id: "gitlab-pat",
        description: "GitLab personal access token",
        severity: Severity::High,
        pattern: r"\bglpat-[A-Za-z0-9_-]{20,}\b",
    },
    Rule {
        id: "slack-token",
        description: "Slack API token",
        severity: Severity::High,
        pattern: r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b",
    },
    Rule {
        id: "slack-webhook-url",
        description: "Slack incoming webhook URL",
        severity: Severity::Medium,
        pattern: r"https://hooks\.slack\.com/services/[A-Za-z0-9]+/[A-Za-z0-9]+/[A-Za-z0-9]+",
    },
    Rule {
        id: "stripe-live-secret-key",
        description: "Stripe live secret key",
        severity: Severity::High,
        pattern: r"\bsk_live_[A-Za-z0-9]{16,}\b",
    },
    Rule {
        id: "stripe-live-restricted-key",
        description: "Stripe live restricted key",
        severity: Severity::High,
        pattern: r"\brk_live_[A-Za-z0-9]{16,}\b",
    },
    Rule {
        id: "google-api-key",
        description: "Google API key",
        severity: Severity::High,
        pattern: r"\bAIza[0-9A-Za-z\-_]{35}\b",
    },
    Rule {
        id: "gcp-service-account-private-key",
        description: "GCP service-account JSON private key block",
        severity: Severity::High,
        pattern: r"-----BEGIN PRIVATE KEY-----",
    },
    Rule {
        id: "pem-rsa-private-key",
        description: "PEM RSA private key block",
        severity: Severity::High,
        pattern: r"-----BEGIN RSA PRIVATE KEY-----",
    },
    Rule {
        id: "pem-ec-private-key",
        description: "PEM EC private key block",
        severity: Severity::High,
        pattern: r"-----BEGIN EC PRIVATE KEY-----",
    },
    Rule {
        id: "pem-openssh-private-key",
        description: "OpenSSH private key block",
        severity: Severity::High,
        pattern: r"-----BEGIN OPENSSH PRIVATE KEY-----",
    },
    Rule {
        id: "pem-pgp-private-key",
        description: "PGP private key block",
        severity: Severity::High,
        pattern: r"-----BEGIN PGP PRIVATE KEY BLOCK-----",
    },
    Rule {
        id: "openai-api-key",
        description: "OpenAI API key",
        severity: Severity::High,
        pattern: r"\bsk-(?:proj-[A-Za-z0-9_-]{20,74}|[A-Za-z0-9]{20,64})\b",
    },
    Rule {
        id: "anthropic-api-key",
        description: "Anthropic API key",
        severity: Severity::High,
        pattern: r"\bsk-ant-[A-Za-z0-9_-]{20,}\b",
    },
    Rule {
        id: "npm-access-token",
        description: "npm access token",
        severity: Severity::High,
        pattern: r"\bnpm_[A-Za-z0-9]{36}\b",
    },
    Rule {
        id: "pypi-api-token",
        description: "PyPI API token",
        severity: Severity::High,
        pattern: r"\bpypi-AgEIcHlwaS5vcmc[A-Za-z0-9_-]{20,}\b",
    },
    Rule {
        id: "sendgrid-api-key",
        description: "SendGrid API key",
        severity: Severity::High,
        pattern: r"\bSG\.[A-Za-z0-9_-]{16,32}\.[A-Za-z0-9_-]{16,64}\b",
    },
    Rule {
        id: "twilio-api-key",
        description: "Twilio API key",
        severity: Severity::High,
        pattern: r"\bSK[0-9a-fA-F]{32}\b",
    },
    Rule {
        id: "digitalocean-token",
        description: "DigitalOcean personal access / OAuth token",
        severity: Severity::High,
        pattern: r"\bdo[opr]_v1_[0-9a-f]{64}\b",
    },
    Rule {
        id: "shopify-token",
        description: "Shopify access token",
        severity: Severity::High,
        pattern: r"\bshp(?:at|ss|ca)_[0-9a-f]{32,}\b",
    },
    Rule {
        id: "discord-bot-token",
        description: "Discord bot token",
        severity: Severity::High,
        pattern: r"\b[MN][A-Za-z0-9_-]{23,25}\.[A-Za-z0-9_-]{6}\.[A-Za-z0-9_-]{27,}\b",
    },
    Rule {
        id: "telegram-bot-token",
        description: "Telegram bot token",
        severity: Severity::High,
        pattern: r"\b\d{8,10}:AA[0-9A-Za-z\-_]{33}\b",
    },
    Rule {
        id: "huggingface-token",
        description: "Hugging Face access token",
        severity: Severity::High,
        pattern: r"\bhf_[A-Za-z0-9]{34,}\b",
    },
    Rule {
        id: "databricks-token",
        description: "Databricks personal access token",
        severity: Severity::High,
        pattern: r"\bdapi[0-9a-f]{32}\b",
    },
    Rule {
        id: "postman-api-key",
        description: "Postman API key",
        severity: Severity::High,
        pattern: r"\bPMAK-[0-9a-fA-F]{20,}\b",
    },
    Rule {
        id: "jwt",
        description: "JSON Web Token (three-segment, base64url header/payload/signature)",
        severity: Severity::Medium,
        pattern: r"\beyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b",
    },
    Rule {
        id: "bitwarden-sm-access-token",
        description: "Bitwarden Secrets Manager machine-account access token",
        severity: Severity::High,
        pattern: concat!(
            r"\b0\.[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}",
            r"\.[A-Za-z0-9_-]{22,}:[A-Za-z0-9+/]{20,}={0,2}",
        ),
    },
    Rule {
        id: "azure-storage-account-key",
        description: "Azure storage account key",
        severity: Severity::High,
        pattern: r"AccountKey=[A-Za-z0-9+/]{86,}={0,2}",
    },
    Rule {
        id: "generic-bearer-token",
        description: "Generic long-lived `Authorization: Bearer` token",
        severity: Severity::Medium,
        pattern: r"(?i)Authorization:\s*Bearer\s+[A-Za-z0-9\-_.]{20,}",
    },
];

#[cfg(test)]
mod tests {
    use regex::{Regex, RegexSet};

    use super::*;

    #[test]
    fn all_rule_ids_are_unique() {
        let mut ids: Vec<&str> = RULES.iter().map(|r| r.id).collect();
        ids.push(HIGH_ENTROPY_RULE_ID);
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(ids.len(), sorted.len(), "duplicate rule id in RULES");
    }

    #[test]
    fn all_rule_ids_are_kebab_case() {
        for rule in RULES {
            assert!(
                rule.id
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "rule id {} is not kebab-case",
                rule.id
            );
        }
    }

    #[test]
    fn every_pattern_compiles_individually_and_as_a_set() {
        let patterns: Vec<&str> = RULES.iter().map(|r| r.pattern).collect();
        for pattern in &patterns {
            Regex::new(pattern)
                .unwrap_or_else(|e| panic!("rule pattern {pattern} failed to compile: {e}"));
        }
        RegexSet::new(&patterns).expect("full rule set must compile as one RegexSet");
    }

    #[test]
    fn ruleset_is_approximately_thirty_rules() {
        // Not a hard requirement, just a guardrail against silent regressions
        // in the hand-authored provider ruleset (see plan §1.2).
        assert!(
            RULES.len() >= 28,
            "expected ~30 provider rules, found {}",
            RULES.len()
        );
    }
}
