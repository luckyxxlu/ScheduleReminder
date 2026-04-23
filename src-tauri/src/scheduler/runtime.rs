use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::db::persistence::{save_action_log, save_all_occurrences};
use crate::notification::event_executor::build_notification_payload;
use crate::scheduler::dispatcher::scan_occurrences;
use crate::state::app_runtime::AppRuntimeState;
use crate::state::database::DatabaseState;
use crate::state::reminder_templates::ReminderTemplateState;

const REMINDER_TRIGGERED_EVENT: &str = "reminder-triggered";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReminderTriggeredPayload {
    occurrence_id: String,
    title: String,
    message: String,
    scheduled_time: String,
    grace_deadline: String,
}

pub fn start_scheduler(
    app_handle: AppHandle,
    runtime: AppRuntimeState,
    templates: ReminderTemplateState,
    database: DatabaseState,
) {
    thread::spawn(move || loop {
        tick(&app_handle, &runtime, &templates, &database);
        thread::sleep(Duration::from_secs(15));
    });
}

fn tick(
    app_handle: &AppHandle,
    runtime: &AppRuntimeState,
    templates: &ReminderTemplateState,
    database: &DatabaseState,
) {
    let now = chrono::Local::now()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let result = {
        let mut occurrences = match runtime.occurrences.lock() {
            Ok(occurrences) => occurrences,
            Err(_) => return,
        };

        let result = scan_occurrences(&now, &mut occurrences);

        if result.triggered_ids.is_empty() && result.logs.is_empty() && result.missed_ids.is_empty() {
            return;
        }

        if save_all_occurrences(&database.pool, &occurrences).is_err() {
            return;
        }

        result
    };

    {
        let mut action_logs = match runtime.action_logs.lock() {
            Ok(action_logs) => action_logs,
            Err(_) => return,
        };

        for log in result.logs.iter().rev() {
            if save_action_log(&database.pool, log).is_err() {
                return;
            }

            action_logs.insert(0, log.clone());
        }
    }

    if result.triggered_ids.is_empty() {
        return;
    }

    let repository = match templates.repository.lock() {
        Ok(repository) => repository,
        Err(_) => return,
    };
    let occurrences = match runtime.occurrences.lock() {
        Ok(occurrences) => occurrences,
        Err(_) => return,
    };

    for occurrence_id in result.triggered_ids {
        let Some(occurrence) = occurrences.iter().find(|item| item.id == occurrence_id) else {
            continue;
        };
        let Some(template) = repository.get(&occurrence.template_id) else {
            continue;
        };
        let Ok(notification_payload) = build_notification_payload(template, occurrence) else {
            continue;
        };

        let notification_body = format!(
            "{}\n宽容截止：{}\n打开应用可执行完成、宽容 10 分钟、稍后提醒、跳过今天。",
            notification_payload.body,
            time_part(&occurrence.grace_deadline_at)
        );

        let _ = app_handle
            .notification()
            .builder()
            .title(&notification_payload.title)
            .body(&notification_body)
            .show();

        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.show();
            let _ = window.set_focus();
        }

        let _ = app_handle.emit(
            REMINDER_TRIGGERED_EVENT,
            ReminderTriggeredPayload {
                occurrence_id: occurrence.id.clone(),
                title: template.title.clone(),
                message: notification_payload.body,
                scheduled_time: time_part(&occurrence.scheduled_at).to_string(),
                grace_deadline: time_part(&occurrence.grace_deadline_at).to_string(),
            },
        );
    }
}

fn time_part(timestamp: &str) -> &str {
    timestamp
        .split_once(' ')
        .map(|(_, time)| &time[..5])
        .unwrap_or(timestamp)
}
