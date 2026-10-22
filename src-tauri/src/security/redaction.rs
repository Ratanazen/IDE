use regex::Regex;
use std::sync::LazyLock;

static BEARER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)bearer\s+[a-zA-Z0-9\-_.~+/]+=*").unwrap()
});

static KV_SECRET_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b([a-zA-Z0-9_\-]*(?:password|passwd|secret|pass|token|api[_-]?key|auth|credential))\s*[:=]\s*([^\s,;'\x22]+)").unwrap()
});

static PRIVATE_KEY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)-----BEGIN [A-Z ]+ PRIVATE KEY-----.*?-----END [A-Z ]+ PRIVATE KEY-----").unwrap()
});

/// Redacts sensitive information from text strings (such as logs or process arguments)
/// before they are stored on disk, emitted to the UI, or passed to AI models.
pub fn redact_secrets(input: &str) -> String {
    // 1. Redact full private key blocks
    let step1 = PRIVATE_KEY_REGEX.replace_all(input, "[REDACTED_PRIVATE_KEY]");

    // 2. Redact Bearer tokens
    let step2 = BEARER_REGEX.replace_all(&step1, "Bearer [REDACTED_TOKEN]");

    // 3. Redact key-value secrets (e.g. keyPassword=..., storePassword=..., token: ...)
    let step3 = KV_SECRET_REGEX.replace_all(&step2, "$1=[REDACTED]");

    step3.into_owned()
}
