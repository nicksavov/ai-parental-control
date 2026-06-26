//! On-device content-safety pipeline.
//!
//! This is the Stage-0 gate: a cheap, deterministic, rule-based pass that runs on
//! every message and image. It drops obviously benign content, flags clear cases
//! (profanity), and escalates ambiguous or high-risk candidates to Stage-1 (an
//! on-device LLM, wired per platform via packages/ai runtimes). It produces an
//! [`AlertDraft`] that becomes an [`apc_alert::Alert`].
//!
//! Nothing here uploads or stores content. Drafts carry only a short, capped
//! snippet for text, never any part of an image.

use apc_alert::{Alert, Category, Modality, Severity};

const SNIPPET_CAP: usize = 280;

/// A candidate alert produced by the pipeline, before it is addressed and sealed.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertDraft {
    pub category: Category,
    pub severity: Severity,
    pub modality: Modality,
    pub rationale: String,
    pub confidence: f64,
    /// Short context snippet. Text only; never set for images.
    pub snippet: Option<String>,
}

impl AlertDraft {
    /// Address the draft and turn it into a validated alert.
    pub fn into_alert(
        self,
        id: impl Into<String>,
        child_device_id: impl Into<String>,
        source: impl Into<String>,
        created_at: impl Into<String>,
    ) -> Alert {
        Alert {
            v: apc_alert::ALERT_SCHEMA_VERSION,
            id: id.into(),
            child_device_id: child_device_id.into(),
            category: self.category,
            severity: self.severity,
            created_at: created_at.into(),
            source: source.into(),
            modality: Some(self.modality),
            confidence: Some(self.confidence),
            rationale: Some(self.rationale),
            snippet: self.snippet,
        }
    }
}

/// What the Stage-0 gate decided for a message.
#[derive(Debug, Clone, PartialEq)]
pub enum GateDecision {
    /// Benign. Drop it; nothing is stored.
    Pass,
    /// Confident enough to alert now (for example clear profanity).
    Flag(AlertDraft),
    /// A candidate that Stage-1 (the on-device LLM) should confirm.
    Escalate(AlertDraft),
}

fn snippet_of(message: &str) -> Option<String> {
    Some(message.chars().take(SNIPPET_CAP).collect())
}

fn contains_any(haystack_lower: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack_lower.contains(n))
}

// Phrase patterns are deliberately specific so idioms do not false-positive. For
// example "kill myself" matches, but "this test is killing me" does not, because
// we never key on the bare word "kill".
const SUICIDE_PATTERNS: &[&str] = &[
    "kill myself",
    "killing myself",
    "kill my self",
    "end my life",
    "want to die",
    "wanna die",
    "i want to die",
    "better off dead",
    "suicidal",
    "suicide",
];

const SELF_HARM_PATTERNS: &[&str] = &[
    "cut myself",
    "cutting myself",
    "hurt myself",
    "harm myself",
    "burn myself",
];

const DRUG_TERMS: &[&str] = &["cocaine", "molly", "meth", "heroin", "xanax", "weed", "vape"];

const PROFANITY_TERMS: &[&str] = &["fuck", "shit", "bitch", "asshole"];

// Grooming is longitudinal. These are cues; one is weak, several together is not.
const GROOMING_CUES: &[&str] = &[
    "don't tell your",
    "dont tell your",
    "don't tell anyone",
    "our secret",
    "keep it secret",
    "delete these messages",
    "delete this chat",
    "send me a pic",
    "send me a picture",
    "how old are you",
    "you're so mature",
    "youre so mature",
    "meet up",
    "don't tell your parents",
];

/// The Stage-0 rule gate.
#[derive(Default)]
pub struct TextGate;

impl TextGate {
    pub fn new() -> TextGate {
        TextGate
    }

    /// Evaluate a single message.
    pub fn evaluate(&self, message: &str) -> GateDecision {
        let lower = message.to_lowercase();

        if contains_any(&lower, SUICIDE_PATTERNS) {
            return GateDecision::Escalate(AlertDraft {
                category: Category::SuicidalIdeation,
                severity: Severity::Severe,
                modality: Modality::Text,
                rationale: "language indicating suicidal ideation".to_string(),
                confidence: 0.6,
                snippet: snippet_of(message),
            });
        }
        if contains_any(&lower, SELF_HARM_PATTERNS) {
            return GateDecision::Escalate(AlertDraft {
                category: Category::SelfHarm,
                severity: Severity::Severe,
                modality: Modality::Text,
                rationale: "language indicating self-harm".to_string(),
                confidence: 0.6,
                snippet: snippet_of(message),
            });
        }
        if contains_any(&lower, DRUG_TERMS) {
            return GateDecision::Escalate(AlertDraft {
                category: Category::Drugs,
                severity: Severity::All,
                modality: Modality::Text,
                rationale: "possible drug reference".to_string(),
                confidence: 0.4,
                snippet: snippet_of(message),
            });
        }
        if contains_any(&lower, PROFANITY_TERMS) {
            return GateDecision::Flag(AlertDraft {
                category: Category::Profanity,
                severity: Severity::All,
                modality: Modality::Text,
                rationale: "profanity".to_string(),
                confidence: 0.9,
                snippet: snippet_of(message),
            });
        }
        GateDecision::Pass
    }

    /// Evaluate a sliding window of recent messages for grooming, which only
    /// shows up across a conversation. Escalates when two or more distinct cues
    /// appear in the window.
    pub fn evaluate_conversation(&self, window: &[&str]) -> Option<AlertDraft> {
        let mut hits = 0usize;
        for cue in GROOMING_CUES {
            if window.iter().any(|m| m.to_lowercase().contains(cue)) {
                hits += 1;
            }
        }
        if hits >= 2 {
            let last = window.last().copied().unwrap_or_default();
            return Some(AlertDraft {
                category: Category::Grooming,
                severity: Severity::Severe,
                modality: Modality::Text,
                rationale: format!("{hits} grooming cues across the conversation"),
                confidence: 0.5,
                snippet: snippet_of(last),
            });
        }
        None
    }
}

/// Scores an image for nudity. Implemented per platform by an on-device model
/// (NudeNet/ViT via ONNX, Core ML, or ExecuTorch). The pipeline only sees a
/// score; the image never leaves the device and is never carried in an alert.
pub trait NudityDetector {
    /// Returns a likelihood in 0.0..=1.0 that the image contains nudity.
    fn score(&self, image: &[u8]) -> f64;
}

/// The image track: score on-device, alert if over threshold, never attach the
/// image.
pub struct ImagePipeline<D: NudityDetector> {
    detector: D,
    threshold: f64,
}

impl<D: NudityDetector> ImagePipeline<D> {
    pub fn new(detector: D, threshold: f64) -> ImagePipeline<D> {
        ImagePipeline { detector, threshold }
    }

    pub fn analyze(&self, image: &[u8]) -> Option<AlertDraft> {
        let score = self.detector.score(image);
        if score < self.threshold {
            return None;
        }
        Some(AlertDraft {
            category: Category::NudityImage,
            severity: Severity::Severe,
            modality: Modality::Image,
            rationale: "on-device nudity detection".to_string(),
            confidence: score,
            // Never any part of the image.
            snippet: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benign_message_passes() {
        assert_eq!(TextGate::new().evaluate("want to grab lunch later?"), GateDecision::Pass);
    }

    #[test]
    fn suicidal_phrasing_escalates() {
        match TextGate::new().evaluate("i don't want to be here, i want to kill myself") {
            GateDecision::Escalate(d) => {
                assert_eq!(d.category, Category::SuicidalIdeation);
                assert_eq!(d.severity, Severity::Severe);
            }
            other => panic!("expected escalate, got {other:?}"),
        }
    }

    #[test]
    fn kill_idiom_does_not_false_positive() {
        // The classic disambiguation: hyperbole must not alert.
        assert_eq!(TextGate::new().evaluate("ugh this test is killing me"), GateDecision::Pass);
        assert_eq!(TextGate::new().evaluate("that movie was to die for"), GateDecision::Pass);
    }

    #[test]
    fn profanity_flags_immediately() {
        match TextGate::new().evaluate("this is such bullshit") {
            GateDecision::Flag(d) => assert_eq!(d.category, Category::Profanity),
            other => panic!("expected flag, got {other:?}"),
        }
    }

    #[test]
    fn grooming_needs_multiple_cues() {
        let gate = TextGate::new();
        // One cue alone is not enough.
        assert!(gate.evaluate_conversation(&["how old are you?"]).is_none());
        // Several cues across the conversation escalate.
        let convo = ["hey", "how old are you", "this is our secret, don't tell your parents"];
        let draft = gate.evaluate_conversation(&convo).expect("should escalate");
        assert_eq!(draft.category, Category::Grooming);
    }

    struct FakeDetector(f64);
    impl NudityDetector for FakeDetector {
        fn score(&self, _image: &[u8]) -> f64 {
            self.0
        }
    }

    #[test]
    fn image_over_threshold_alerts_without_a_snippet() {
        let pipeline = ImagePipeline::new(FakeDetector(0.95), 0.8);
        let draft = pipeline.analyze(b"fake-bytes").expect("should alert");
        assert_eq!(draft.category, Category::NudityImage);
        assert_eq!(draft.snippet, None); // never any part of the image
    }

    #[test]
    fn image_under_threshold_passes() {
        let pipeline = ImagePipeline::new(FakeDetector(0.1), 0.8);
        assert!(pipeline.analyze(b"fake-bytes").is_none());
    }

    #[test]
    fn drafts_become_valid_alerts() {
        // A text draft and an image draft must both produce schema-valid alerts.
        let text = TextGate::new().evaluate("i want to kill myself");
        if let GateDecision::Escalate(d) = text {
            let alert = d.into_alert("a1", "dev", "sms", "2026-06-26T00:00:00Z");
            alert.validate().expect("text alert should be valid");
        } else {
            panic!("expected escalate");
        }

        let img = ImagePipeline::new(FakeDetector(0.99), 0.8)
            .analyze(b"x")
            .unwrap()
            .into_alert("a2", "dev", "photos", "2026-06-26T00:00:00Z");
        // The image alert must validate, which requires snippet to be absent.
        img.validate().expect("image alert should be valid");
        assert_eq!(img.snippet, None);
    }
}
