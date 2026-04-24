use rusqlite::params;

use crate::db::migration::DbPool;
use crate::db::reminder_template_repository::InMemoryReminderTemplateRepository;
use crate::models::reminder_action_log::ReminderActionLog;
use crate::models::reminder_occurrence::ReminderOccurrence;
use crate::models::reminder_template::{ReminderEventType, ReminderTemplate};
use crate::settings::app_settings::AppSettings;

#[derive(Debug, PartialEq, Eq)]
pub enum PersistenceError {
    DatabaseUnavailable,
    StatementExecutionFailed,
}

pub fn bootstrap_defaults(
    pool: &DbPool,
    templates: &[ReminderTemplate],
    occurrences: &[ReminderOccurrence],
    settings: &AppSettings,
) -> Result<(), PersistenceError> {
    let connection = pool.lock().map_err(|_| PersistenceError::DatabaseUnavailable)?;

    if count_rows(&connection, "reminder_templates")? == 0 {
        insert_templates(&connection, templates)?;
    }

    if count_rows(&connection, "reminder_occurrences")? == 0 {
        insert_occurrences(&connection, occurrences)?;
    }

    if count_rows(&connection, "app_settings")? == 0 {
        save_settings_with_connection(&connection, settings)?;
    }

    Ok(())
}

pub fn load_template_repository(pool: &DbPool) -> Result<InMemoryReminderTemplateRepository, PersistenceError> {
    let connection = pool.lock().map_err(|_| PersistenceError::DatabaseUnavailable)?;
    let mut statement = connection
        .prepare(
            "SELECT id, title, category, event_type, event_payload_json, repeat_rule_json, default_grace_minutes, notify_sound, note, enabled FROM reminder_templates ORDER BY id",
        )
        .map_err(|_| PersistenceError::StatementExecutionFailed)?;

    let rows = statement
        .query_map([], |row| {
            Ok(ReminderTemplate {
                id: row.get(0)?,
                title: row.get(1)?,
                category: row.get(2)?,
                event_type: map_event_type(&row.get::<_, String>(3)?),
                event_payload_json: row.get(4)?,
                repeat_rule_json: row.get(5)?,
                default_grace_minutes: row.get(6)?,
                notify_sound: row.get::<_, i64>(7)? > 0,
                note: row.get(8)?,
                enabled: row.get::<_, i64>(9)? > 0,
            })
        })
        .map_err(|_| PersistenceError::StatementExecutionFailed)?;

    let items = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| PersistenceError::StatementExecutionFailed)?;

    Ok(InMemoryReminderTemplateRepository::from_items(items))
}

pub fn load_occurrences(pool: &DbPool) -> Result<Vec<ReminderOccurrence>, PersistenceError> {
    let connection = pool.lock().map_err(|_| PersistenceError::DatabaseUnavailable)?;
    let mut statement = connection
        .prepare(
            "SELECT id, template_id, scheduled_at, grace_deadline_at, snoozed_until, status, handled_at FROM reminder_occurrences ORDER BY scheduled_at, id",
        )
        .map_err(|_| PersistenceError::StatementExecutionFailed)?;

    let rows = statement
        .query_map([], |row| {
            Ok(ReminderOccurrence {
                id: row.get(0)?,
                template_id: row.get(1)?,
                scheduled_at: row.get(2)?,
                grace_deadline_at: row.get(3)?,
                snoozed_until: row.get(4)?,
                status: row.get(5)?,
                handled_at: row.get(6)?,
            })
        })
        .map_err(|_| PersistenceError::StatementExecutionFailed)?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|_| PersistenceError::StatementExecutionFailed)
}

pub fn load_action_logs(pool: &DbPool) -> Result<Vec<ReminderActionLog>, PersistenceError> {
    let connection = pool.lock().map_err(|_| PersistenceError::DatabaseUnavailable)?;
    let mut statement = connection
        .prepare(
            "SELECT id, occurrence_id, action, action_at, payload_json FROM reminder_action_logs ORDER BY action_at DESC, id DESC",
        )
        .map_err(|_| PersistenceError::StatementExecutionFailed)?;

    let rows = statement
        .query_map([], |row| {
            Ok(ReminderActionLog {
                id: row.get(0)?,
                occurrence_id: row.get(1)?,
                action: row.get(2)?,
                action_at: row.get(3)?,
                payload_json: row.get(4)?,
            })
        })
        .map_err(|_| PersistenceError::StatementExecutionFailed)?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|_| PersistenceError::StatementExecutionFailed)
}

pub fn load_settings(pool: &DbPool) -> Result<AppSettings, PersistenceError> {
    let connection = pool.lock().map_err(|_| PersistenceError::DatabaseUnavailable)?;

    connection
        .query_row(
            "SELECT default_grace_minutes, startup_with_windows, tray_enabled, close_to_tray_on_close, theme, quiet_hours_enabled, quiet_hours_start, quiet_hours_end FROM app_settings WHERE id = 1",
            [],
            |row| {
                Ok(AppSettings {
                    default_grace_minutes: row.get(0)?,
                    startup_with_windows: row.get::<_, i64>(1)? > 0,
                    tray_enabled: row.get::<_, i64>(2)? > 0,
                    close_to_tray_on_close: row.get::<_, i64>(3)? > 0,
                    theme: row.get(4)?,
                    quiet_hours_enabled: row.get::<_, i64>(5)? > 0,
                    quiet_hours_start: row.get(6)?,
                    quiet_hours_end: row.get(7)?,
                })
            },
        )
        .map_err(|_| PersistenceError::StatementExecutionFailed)
}

pub fn save_all_templates(pool: &DbPool, templates: &[ReminderTemplate]) -> Result<(), PersistenceError> {
    let connection = pool.lock().map_err(|_| PersistenceError::DatabaseUnavailable)?;
    insert_templates(&connection, templates)
}

pub fn save_all_occurrences(pool: &DbPool, occurrences: &[ReminderOccurrence]) -> Result<(), PersistenceError> {
    let connection = pool.lock().map_err(|_| PersistenceError::DatabaseUnavailable)?;
    insert_occurrences(&connection, occurrences)
}

pub fn save_settings(pool: &DbPool, settings: &AppSettings) -> Result<(), PersistenceError> {
    let connection = pool.lock().map_err(|_| PersistenceError::DatabaseUnavailable)?;
    save_settings_with_connection(&connection, settings)
}

pub fn save_action_log(pool: &DbPool, log: &ReminderActionLog) -> Result<(), PersistenceError> {
    let connection = pool.lock().map_err(|_| PersistenceError::DatabaseUnavailable)?;

    connection
        .execute(
            "INSERT INTO reminder_action_logs (id, occurrence_id, action, action_at, payload_json) VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(id) DO UPDATE SET action = excluded.action, action_at = excluded.action_at, payload_json = excluded.payload_json",
            params![log.id, log.occurrence_id, log.action, log.action_at, log.payload_json],
        )
        .map_err(|_| PersistenceError::StatementExecutionFailed)?;

    Ok(())
}

pub fn delete_occurrence_and_logs(
    pool: &DbPool,
    occurrence_id: &str,
) -> Result<(), PersistenceError> {
    let connection = pool.lock().map_err(|_| PersistenceError::DatabaseUnavailable)?;
    let transaction = connection
        .unchecked_transaction()
        .map_err(|_| PersistenceError::StatementExecutionFailed)?;

    transaction
        .execute(
            "DELETE FROM reminder_action_logs WHERE occurrence_id = ?1",
            params![occurrence_id],
        )
        .map_err(|_| PersistenceError::StatementExecutionFailed)?;
    transaction
        .execute(
            "DELETE FROM reminder_occurrences WHERE id = ?1",
            params![occurrence_id],
        )
        .map_err(|_| PersistenceError::StatementExecutionFailed)?;

    transaction
        .commit()
        .map_err(|_| PersistenceError::StatementExecutionFailed)
}

pub fn delete_template(pool: &DbPool, template_id: &str) -> Result<(), PersistenceError> {
    let connection = pool.lock().map_err(|_| PersistenceError::DatabaseUnavailable)?;

    connection
        .execute(
            "DELETE FROM reminder_templates WHERE id = ?1",
            params![template_id],
        )
        .map_err(|_| PersistenceError::StatementExecutionFailed)?;

    Ok(())
}

fn count_rows(connection: &rusqlite::Connection, table: &str) -> Result<i64, PersistenceError> {
    connection
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row.get(0))
        .map_err(|_| PersistenceError::StatementExecutionFailed)
}

fn insert_templates(
    connection: &rusqlite::Connection,
    templates: &[ReminderTemplate],
) -> Result<(), PersistenceError> {
    for template in templates {
        connection
            .execute(
                "INSERT INTO reminder_templates (id, title, category, event_type, event_payload_json, repeat_rule_json, default_grace_minutes, notify_sound, note, enabled, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) ON CONFLICT(id) DO UPDATE SET title = excluded.title, category = excluded.category, event_type = excluded.event_type, event_payload_json = excluded.event_payload_json, repeat_rule_json = excluded.repeat_rule_json, default_grace_minutes = excluded.default_grace_minutes, notify_sound = excluded.notify_sound, note = excluded.note, enabled = excluded.enabled, updated_at = CURRENT_TIMESTAMP",
                params![
                    template.id,
                    template.title,
                    template.category,
                    event_type_key(&template.event_type),
                    template.event_payload_json,
                    template.repeat_rule_json,
                    template.default_grace_minutes,
                    bool_to_flag(template.notify_sound),
                    template.note,
                    bool_to_flag(template.enabled),
                ],
            )
            .map_err(|_| PersistenceError::StatementExecutionFailed)?;
    }

    Ok(())
}

fn insert_occurrences(
    connection: &rusqlite::Connection,
    occurrences: &[ReminderOccurrence],
) -> Result<(), PersistenceError> {
    for occurrence in occurrences {
        connection
            .execute(
                "INSERT INTO reminder_occurrences (id, template_id, scheduled_at, grace_deadline_at, snoozed_until, status, handled_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) ON CONFLICT(id) DO UPDATE SET template_id = excluded.template_id, scheduled_at = excluded.scheduled_at, grace_deadline_at = excluded.grace_deadline_at, snoozed_until = excluded.snoozed_until, status = excluded.status, handled_at = excluded.handled_at, updated_at = CURRENT_TIMESTAMP",
                params![
                    occurrence.id,
                    occurrence.template_id,
                    occurrence.scheduled_at,
                    occurrence.grace_deadline_at,
                    occurrence.snoozed_until,
                    occurrence.status,
                    occurrence.handled_at,
                ],
            )
            .map_err(|_| PersistenceError::StatementExecutionFailed)?;
    }

    Ok(())
}

fn save_settings_with_connection(
    connection: &rusqlite::Connection,
    settings: &AppSettings,
) -> Result<(), PersistenceError> {
    connection
        .execute(
            "INSERT INTO app_settings (id, default_grace_minutes, startup_with_windows, tray_enabled, close_to_tray_on_close, theme, quiet_hours_enabled, quiet_hours_start, quiet_hours_end, updated_at) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CURRENT_TIMESTAMP) ON CONFLICT(id) DO UPDATE SET default_grace_minutes = excluded.default_grace_minutes, startup_with_windows = excluded.startup_with_windows, tray_enabled = excluded.tray_enabled, close_to_tray_on_close = excluded.close_to_tray_on_close, theme = excluded.theme, quiet_hours_enabled = excluded.quiet_hours_enabled, quiet_hours_start = excluded.quiet_hours_start, quiet_hours_end = excluded.quiet_hours_end, updated_at = CURRENT_TIMESTAMP",
            params![
                settings.default_grace_minutes,
                bool_to_flag(settings.startup_with_windows),
                bool_to_flag(settings.tray_enabled),
                bool_to_flag(settings.close_to_tray_on_close),
                settings.theme,
                bool_to_flag(settings.quiet_hours_enabled),
                settings.quiet_hours_start,
                settings.quiet_hours_end,
            ],
        )
        .map_err(|_| PersistenceError::StatementExecutionFailed)?;

    Ok(())
}

fn map_event_type(value: &str) -> ReminderEventType {
    match value {
        "system_action" => ReminderEventType::SystemAction,
        _ => ReminderEventType::Text,
    }
}

fn event_type_key(value: &ReminderEventType) -> &'static str {
    match value {
        ReminderEventType::Text => "text",
        ReminderEventType::SystemAction => "system_action",
    }
}

fn bool_to_flag(value: bool) -> i64 {
    if value { 1 } else { 0 }
}
