#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReminderEventType {
    Text,
    SystemAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderTemplate {
    pub id: String,
    pub title: String,
    pub category: Option<String>,
    pub event_type: ReminderEventType,
    pub event_payload_json: String,
    pub repeat_rule_json: String,
    pub default_grace_minutes: i32,
    pub notify_sound: bool,
    pub note: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateReminderTemplateInput {
    pub title: String,
    pub category: Option<String>,
    pub event_type: ReminderEventType,
    pub event_payload_json: String,
    pub repeat_rule_json: String,
    pub default_grace_minutes: i32,
    pub notify_sound: bool,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateReminderTemplateInput {
    pub id: String,
    pub title: String,
    pub category: Option<String>,
    pub event_type: ReminderEventType,
    pub event_payload_json: String,
    pub repeat_rule_json: String,
    pub default_grace_minutes: i32,
    pub notify_sound: bool,
    pub note: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReminderTemplateError {
    EmptyTitle,
    InvalidEventPayload,
    InvalidRepeatRule,
    NegativeGraceMinutes,
    NotFound,
}
