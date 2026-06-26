//! The policy a parent sets for one child, distributed from the backend to that
//! child's agents. One model for every platform; each agent enforces what its
//! OS allows and reports the rest as not enforceable. Mirrors policy.schema.json.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const POLICY_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChildPolicy {
    pub v: u32,
    pub child_id: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub filtering: Option<Filtering>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub screen_time: Option<ScreenTime>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub monitoring: Option<Monitoring>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub location: Option<Location>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterCategory {
    Adult,
    Drugs,
    Alcohol,
    Gambling,
    Weapons,
    Violence,
    Hate,
    Illegal,
    Malware,
    Streaming,
    OnlineGaming,
    SocialMedia,
    Shopping,
    FileSharing,
    ProxyVpn,
    Chat,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Filtering {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub safe_search: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub blocked_categories: Option<Vec<FilterCategory>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub blocked_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub allowed_domains: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScreenTime {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub daily_total_minutes: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub per_app: Option<Vec<AppLimit>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub windows: Option<Vec<ScheduleWindow>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppLimit {
    pub app_id: String,
    pub daily_minutes: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowMode {
    BlockAll,
    AllowOnly,
    BlockInternet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Day {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScheduleWindow {
    pub name: String,
    pub mode: WindowMode,
    pub days: Vec<Day>,
    /// Local time HH:MM.
    pub start: String,
    pub end: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub allowed_apps: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sensitivity {
    None,
    All,
    Severe,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Monitoring {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub image_nudity: Option<bool>,
    /// Off by default. Only effective where the OS and build allow reading text.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub text_analysis: Option<bool>,
    /// Per-category alert tier, keyed by the category names in alert.schema.json.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub category_sensitivity: Option<HashMap<String, Sensitivity>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Location {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub enabled: Option<bool>,
    /// Capped at 90, short default keeps retention low (a COPPA expectation).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub history_days: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub geofences: Option<Vec<Geofence>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Geofence {
    pub name: String,
    pub lat: f64,
    pub lng: f64,
    pub radius_meters: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "v": 1,
        "childId": "child-1",
        "updatedAt": "2026-06-26T00:00:00Z",
        "filtering": { "enabled": true, "safeSearch": true, "blockedCategories": ["adult", "gambling"] },
        "screenTime": {
            "dailyTotalMinutes": 120,
            "perApp": [{ "appId": "com.example.game", "dailyMinutes": 30 }],
            "windows": [{ "name": "bedtime", "mode": "block_all", "days": ["mon", "tue"], "start": "21:00", "end": "07:00" }]
        },
        "monitoring": { "imageNudity": true, "textAnalysis": false, "categorySensitivity": { "profanity": "all", "suicidal_ideation": "severe" } },
        "location": { "enabled": true, "historyDays": 30, "geofences": [{ "name": "home", "lat": 1.5, "lng": -2.5, "radiusMeters": 150 }] }
    }"#;

    #[test]
    fn sample_policy_round_trips() {
        let policy: ChildPolicy = serde_json::from_str(SAMPLE).unwrap();
        assert_eq!(policy.v, POLICY_VERSION);
        assert_eq!(policy.child_id, "child-1");
        let json = serde_json::to_string(&policy).unwrap();
        let back: ChildPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(policy, back);
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let json = r#"{"v":1,"childId":"c","updatedAt":"t","surpriseField":true}"#;
        let parsed: Result<ChildPolicy, _> = serde_json::from_str(json);
        assert!(parsed.is_err(), "unknown policy field must be rejected");
    }

    #[test]
    fn enum_values_match_schema_strings() {
        assert_eq!(serde_json::to_string(&WindowMode::BlockInternet).unwrap(), "\"block_internet\"");
        assert_eq!(serde_json::to_string(&FilterCategory::ProxyVpn).unwrap(), "\"proxy_vpn\"");
        assert_eq!(serde_json::to_string(&Sensitivity::Severe).unwrap(), "\"severe\"");
        assert_eq!(serde_json::to_string(&Day::Sun).unwrap(), "\"sun\"");
    }
}
