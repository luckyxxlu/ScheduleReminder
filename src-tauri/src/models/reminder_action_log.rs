#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderActionLog {
    pub id: String,
    pub occurrence_id: String,
    pub action: String,
    pub action_at: String,
    pub payload_json: Option<String>,
}
