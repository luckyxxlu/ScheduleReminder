#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderOccurrence {
    pub id: String,
    pub template_id: String,
    pub scheduled_at: String,
    pub grace_deadline_at: String,
    pub snoozed_until: Option<String>,
    pub status: String,
    pub handled_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReminderOccurrenceError {
    DuplicateOccurrence,
    InvalidRepeatRule,
}
