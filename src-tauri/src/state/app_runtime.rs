use std::sync::{Arc, Mutex};

use crate::models::reminder_action_log::ReminderActionLog;
use crate::models::reminder_occurrence::ReminderOccurrence;
use crate::settings::app_settings::AppSettings;

#[derive(Clone)]
pub struct AppRuntimeState {
    pub occurrences: Arc<Mutex<Vec<ReminderOccurrence>>>,
    pub action_logs: Arc<Mutex<Vec<ReminderActionLog>>>,
    pub settings: Arc<Mutex<AppSettings>>,
}

impl AppRuntimeState {
    pub fn new(
        occurrences: Vec<ReminderOccurrence>,
        action_logs: Vec<ReminderActionLog>,
        settings: AppSettings,
    ) -> Self {
        Self {
            occurrences: Arc::new(Mutex::new(occurrences)),
            action_logs: Arc::new(Mutex::new(action_logs)),
            settings: Arc::new(Mutex::new(settings)),
        }
    }
}
