//! The alert type and the no-raw-content invariant.
//!
//! An alert carries structured metadata only. The single exception is
//! `snippet`, a short context string for text alerts that is capped and never
//! present for images. `#[serde(deny_unknown_fields)]` mirrors the JSON Schema
//! `additionalProperties: false`, so a payload that smuggles in a forbidden key
//! (rawText, image, mediaPath, ...) fails to deserialize. See README.md and
//! alert.schema.json in this directory.

use serde::{Deserialize, Serialize};

/// Current alert schema version.
pub const ALERT_SCHEMA_VERSION: u32 = 1;
/// Max characters for the optional text snippet.
pub const MAX_SNIPPET_LEN: usize = 280;
/// Max characters for the optional rationale.
pub const MAX_RATIONALE_LEN: usize = 280;
/// Max characters for the source label.
pub const MAX_SOURCE_LEN: usize = 64;

/// The risk category that fired.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Cyberbullying,
    Depression,
    Anxiety,
    SelfHarm,
    SuicidalIdeation,
    Grooming,
    SexualContentText,
    NudityImage,
    Drugs,
    Alcohol,
    SmokingVaping,
    Violence,
    Weapons,
    HateSpeech,
    Profanity,
    Gambling,
    BodyImage,
    ContactConcern,
    FilterBlock,
    LimitReached,
    Tamper,
}

/// Alert tier. `severe` is high priority, `all` is moderate, `info` is for
/// non-content events (filter blocks, limits, tamper).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    All,
    Severe,
}

/// What kind of signal produced the alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    Text,
    Image,
    Web,
    Usage,
    Location,
    System,
}

/// The plaintext payload of an alert, produced on the child device. This is
/// encrypted end to end before it leaves the device; the backend only ever
/// sees ciphertext.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Alert {
    pub v: u32,
    pub id: String,
    pub child_device_id: String,
    pub category: Category,
    pub severity: Severity,
    pub created_at: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub modality: Option<Modality>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rationale: Option<String>,
    /// Short context snippet for text alerts only. Never set for images.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub snippet: Option<String>,
}

/// Why an alert is invalid.
#[derive(Debug, Clone, PartialEq)]
pub enum AlertError {
    WrongVersion(u32),
    MissingField(&'static str),
    SourceTooLong(usize),
    SnippetTooLong(usize),
    RationaleTooLong(usize),
    /// A snippet was present on an image alert. We never include any part of a
    /// flagged image.
    SnippetOnImage,
    /// Confidence must be within 0.0 to 1.0.
    ConfidenceOutOfRange(f64),
}

impl std::fmt::Display for AlertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertError::WrongVersion(v) => write!(f, "unsupported alert version {v}"),
            AlertError::MissingField(name) => write!(f, "missing required field {name}"),
            AlertError::SourceTooLong(n) => write!(f, "source too long ({n} > {MAX_SOURCE_LEN})"),
            AlertError::SnippetTooLong(n) => write!(f, "snippet too long ({n} > {MAX_SNIPPET_LEN})"),
            AlertError::RationaleTooLong(n) => {
                write!(f, "rationale too long ({n} > {MAX_RATIONALE_LEN})")
            }
            AlertError::SnippetOnImage => {
                write!(f, "snippet must not be present on an image alert")
            }
            AlertError::ConfidenceOutOfRange(c) => write!(f, "confidence {c} out of range 0.0..=1.0"),
        }
    }
}

impl std::error::Error for AlertError {}

impl Alert {
    /// Validate the invariants the JSON Schema cannot fully express in code.
    pub fn validate(&self) -> Result<(), AlertError> {
        if self.v != ALERT_SCHEMA_VERSION {
            return Err(AlertError::WrongVersion(self.v));
        }
        if self.id.is_empty() {
            return Err(AlertError::MissingField("id"));
        }
        if self.child_device_id.is_empty() {
            return Err(AlertError::MissingField("childDeviceId"));
        }
        if self.created_at.is_empty() {
            return Err(AlertError::MissingField("createdAt"));
        }
        if self.source.is_empty() {
            return Err(AlertError::MissingField("source"));
        }
        if self.source.chars().count() > MAX_SOURCE_LEN {
            return Err(AlertError::SourceTooLong(self.source.chars().count()));
        }
        if let Some(c) = self.confidence {
            if !(0.0..=1.0).contains(&c) {
                return Err(AlertError::ConfidenceOutOfRange(c));
            }
        }
        if let Some(r) = &self.rationale {
            let n = r.chars().count();
            if n > MAX_RATIONALE_LEN {
                return Err(AlertError::RationaleTooLong(n));
            }
        }
        if let Some(s) = &self.snippet {
            let n = s.chars().count();
            if n > MAX_SNIPPET_LEN {
                return Err(AlertError::SnippetTooLong(n));
            }
            if self.modality == Some(Modality::Image) {
                return Err(AlertError::SnippetOnImage);
            }
        }
        Ok(())
    }

    /// Parse JSON and validate. Forbidden keys fail at the parse step because of
    /// `deny_unknown_fields`; the rest is checked by `validate`.
    pub fn from_json(input: &str) -> Result<Alert, Box<dyn std::error::Error>> {
        let alert: Alert = serde_json::from_str(input)?;
        alert.validate()?;
        Ok(alert)
    }

    /// Serialize to JSON. Caller encrypts the result before it leaves the device.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_text_alert() -> Alert {
        Alert {
            v: ALERT_SCHEMA_VERSION,
            id: "11111111-1111-4111-8111-111111111111".to_string(),
            child_device_id: "device-abc".to_string(),
            category: Category::SuicidalIdeation,
            severity: Severity::Severe,
            created_at: "2026-06-26T00:00:00Z".to_string(),
            source: "sms".to_string(),
            modality: Some(Modality::Text),
            confidence: Some(0.92),
            rationale: Some("language indicating self-harm".to_string()),
            snippet: Some("i want to ...".to_string()),
        }
    }

    #[test]
    fn valid_alert_round_trips() {
        let a = sample_text_alert();
        a.validate().expect("sample should be valid");
        let json = a.to_json().unwrap();
        let back = Alert::from_json(&json).unwrap();
        assert_eq!(a, back);
    }

    #[test]
    fn forbidden_keys_are_rejected_at_parse() {
        // Every one of these keys would turn an alert into a content store.
        // deny_unknown_fields must reject all of them.
        let forbidden = [
            "rawText",
            "body",
            "fullMessage",
            "image",
            "imageData",
            "base64",
            "mediaPath",
            "filePath",
            "mediaUrl",
            "contentUrl",
            "attachment",
            "thumbnail",
            "screenshot",
        ];
        for key in forbidden {
            let json = format!(
                r#"{{"v":1,"id":"x","childDeviceId":"d","category":"profanity","severity":"all","createdAt":"2026-06-26T00:00:00Z","source":"sms","{key}":"smuggled"}}"#
            );
            let parsed: Result<Alert, _> = serde_json::from_str(&json);
            assert!(
                parsed.is_err(),
                "forbidden key {key} was accepted but must be rejected"
            );
        }
    }

    #[test]
    fn image_alert_must_not_carry_a_snippet() {
        let mut a = sample_text_alert();
        a.category = Category::NudityImage;
        a.modality = Some(Modality::Image);
        a.snippet = Some("anything".to_string());
        assert_eq!(a.validate(), Err(AlertError::SnippetOnImage));
    }

    #[test]
    fn snippet_over_cap_is_rejected() {
        let mut a = sample_text_alert();
        a.snippet = Some("x".repeat(MAX_SNIPPET_LEN + 1));
        assert_eq!(
            a.validate(),
            Err(AlertError::SnippetTooLong(MAX_SNIPPET_LEN + 1))
        );
    }

    #[test]
    fn rationale_over_cap_is_rejected() {
        let mut a = sample_text_alert();
        a.rationale = Some("x".repeat(MAX_RATIONALE_LEN + 1));
        assert_eq!(
            a.validate(),
            Err(AlertError::RationaleTooLong(MAX_RATIONALE_LEN + 1))
        );
    }

    #[test]
    fn source_over_cap_is_rejected() {
        let mut a = sample_text_alert();
        a.source = "x".repeat(MAX_SOURCE_LEN + 1);
        assert_eq!(a.validate(), Err(AlertError::SourceTooLong(MAX_SOURCE_LEN + 1)));
    }

    #[test]
    fn confidence_out_of_range_is_rejected() {
        let mut a = sample_text_alert();
        a.confidence = Some(1.5);
        assert_eq!(a.validate(), Err(AlertError::ConfidenceOutOfRange(1.5)));
    }

    #[test]
    fn wrong_version_is_rejected() {
        let mut a = sample_text_alert();
        a.v = 2;
        assert_eq!(a.validate(), Err(AlertError::WrongVersion(2)));
    }

    #[test]
    fn enum_values_match_schema_strings() {
        // Guard against accidental rename drift from alert.schema.json.
        assert_eq!(
            serde_json::to_string(&Category::SuicidalIdeation).unwrap(),
            "\"suicidal_ideation\""
        );
        assert_eq!(serde_json::to_string(&Category::NudityImage).unwrap(), "\"nudity_image\"");
        assert_eq!(serde_json::to_string(&Severity::Severe).unwrap(), "\"severe\"");
        assert_eq!(serde_json::to_string(&Modality::Image).unwrap(), "\"image\"");
    }
}
