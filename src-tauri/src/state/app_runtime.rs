use std::sync::Mutex;

use crate::models::reminder_action_log::ReminderActionLog;
use crate::models::reminder_occurrence::ReminderOccurrence;
use crate::settings::app_settings::AppSettings;

pub struct AppRuntimeState {
    pub occurrences: Mutex<Vec<ReminderOccurrence>>,
    pub action_logs: Mutex<Vec<ReminderActionLog>>,
    pub settings: Mutex<AppSettings>,
}

impl AppRuntimeState {
    pub fn new(
        occurrences: Vec<ReminderOccurrence>,
        action_logs: Vec<ReminderActionLog>,
        settings: AppSettings,
    ) -> Self {
        Self {
            occurrences: Mutex::new(occurrences),
            action_logs: Mutex::new(action_logs),
            settings: Mutex::new(settings),
        }
    }
}
