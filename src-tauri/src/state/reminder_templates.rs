use std::sync::Mutex;

use crate::db::reminder_template_repository::InMemoryReminderTemplateRepository;

pub struct ReminderTemplateState {
    pub repository: Mutex<InMemoryReminderTemplateRepository>,
}

impl ReminderTemplateState {
    pub fn new(repository: InMemoryReminderTemplateRepository) -> Self {
        Self {
            repository: Mutex::new(repository),
        }
    }
}
