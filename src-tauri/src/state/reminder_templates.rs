use std::sync::{Arc, Mutex};

use crate::db::reminder_template_repository::InMemoryReminderTemplateRepository;

#[derive(Clone)]
pub struct ReminderTemplateState {
    pub repository: Arc<Mutex<InMemoryReminderTemplateRepository>>,
}

impl ReminderTemplateState {
    pub fn new(repository: InMemoryReminderTemplateRepository) -> Self {
        Self {
            repository: Arc::new(Mutex::new(repository)),
        }
    }
}
