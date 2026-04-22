use std::sync::Mutex;

use crate::models::reminder_occurrence::ReminderOccurrence;
use crate::settings::app_settings::AppSettings;

pub struct AppRuntimeState {
    pub occurrences: Mutex<Vec<ReminderOccurrence>>,
    pub settings: Mutex<AppSettings>,
}

impl AppRuntimeState {
    pub fn new(occurrences: Vec<ReminderOccurrence>, settings: AppSettings) -> Self {
        Self {
            occurrences: Mutex::new(occurrences),
            settings: Mutex::new(settings),
        }
    }
}
