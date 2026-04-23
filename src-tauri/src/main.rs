#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use schedule_reminder::commands::app::{
    create_calendar_event_command, create_reminder_template_command, default_app_settings, duplicate_reminder_template_command,
    get_calendar_overview_command, get_settings_command, get_today_dashboard_command,
    grace_next_reminder_ten_minutes_command, greet, list_reminder_templates_command,
    mark_next_reminder_completed_command, seed_occurrences, seed_reminder_templates,
    skip_next_reminder_command, snooze_next_reminder_command, toggle_reminder_template_command,
    update_reminder_template_command, update_settings_command,
};
use schedule_reminder::commands::app::{
    CalendarOverviewData, ReminderTemplateCommandError, ReminderTemplateListItem, SettingsViewData,
    TodayDashboardData,
};
use schedule_reminder::db::config::load_db_config;
use schedule_reminder::db::migration::initialize_database;
use schedule_reminder::db::persistence::{
    bootstrap_defaults, load_action_logs, load_occurrences, load_settings, load_template_repository,
};
use schedule_reminder::state::app_runtime::AppRuntimeState;
use schedule_reminder::state::database::DatabaseState;
use schedule_reminder::state::reminder_templates::ReminderTemplateState;
use tauri::Manager;

#[tauri::command]
fn greet_command(name: &str) -> String {
    greet(name)
}

#[tauri::command]
fn list_reminder_templates(
    state: tauri::State<'_, ReminderTemplateState>,
) -> Result<Vec<ReminderTemplateListItem>, ReminderTemplateCommandError> {
    list_reminder_templates_command(state)
}

#[tauri::command]
fn toggle_reminder_template(
    state: tauri::State<'_, ReminderTemplateState>,
    database: tauri::State<'_, DatabaseState>,
    id: String,
    enabled: bool,
) -> Result<ReminderTemplateListItem, ReminderTemplateCommandError> {
    toggle_reminder_template_command(state, database, id, enabled)
}

#[tauri::command]
fn duplicate_reminder_template(
    state: tauri::State<'_, ReminderTemplateState>,
    database: tauri::State<'_, DatabaseState>,
    id: String,
) -> Result<ReminderTemplateListItem, ReminderTemplateCommandError> {
    duplicate_reminder_template_command(state, database, id)
}

#[tauri::command]
fn create_reminder_template(
    state: tauri::State<'_, ReminderTemplateState>,
    database: tauri::State<'_, DatabaseState>,
    title: String,
    message: String,
    category: Option<String>,
    repeat_rule_json: String,
    default_grace_minutes: i32,
    note: Option<String>,
) -> Result<ReminderTemplateListItem, ReminderTemplateCommandError> {
    create_reminder_template_command(
        state,
        database,
        title,
        message,
        category,
        repeat_rule_json,
        default_grace_minutes,
        note,
    )
}

#[tauri::command]
fn update_reminder_template(
    state: tauri::State<'_, ReminderTemplateState>,
    database: tauri::State<'_, DatabaseState>,
    id: String,
    title: String,
    message: String,
    category: Option<String>,
    repeat_rule_json: String,
    default_grace_minutes: i32,
    note: Option<String>,
    enabled: bool,
) -> Result<ReminderTemplateListItem, ReminderTemplateCommandError> {
    update_reminder_template_command(
        state,
        database,
        id,
        title,
        message,
        category,
        repeat_rule_json,
        default_grace_minutes,
        note,
        enabled,
    )
}

#[tauri::command]
fn get_today_dashboard(
    runtime: tauri::State<'_, AppRuntimeState>,
    templates: tauri::State<'_, ReminderTemplateState>,
) -> Result<TodayDashboardData, ReminderTemplateCommandError> {
    get_today_dashboard_command(runtime, templates)
}

#[tauri::command]
fn mark_next_reminder_completed(
    runtime: tauri::State<'_, AppRuntimeState>,
    templates: tauri::State<'_, ReminderTemplateState>,
    database: tauri::State<'_, DatabaseState>,
) -> Result<TodayDashboardData, ReminderTemplateCommandError> {
    mark_next_reminder_completed_command(runtime, templates, database)
}

#[tauri::command]
fn grace_next_reminder_ten_minutes(
    runtime: tauri::State<'_, AppRuntimeState>,
    templates: tauri::State<'_, ReminderTemplateState>,
    database: tauri::State<'_, DatabaseState>,
) -> Result<TodayDashboardData, ReminderTemplateCommandError> {
    grace_next_reminder_ten_minutes_command(runtime, templates, database)
}

#[tauri::command]
fn snooze_next_reminder(
    runtime: tauri::State<'_, AppRuntimeState>,
    templates: tauri::State<'_, ReminderTemplateState>,
    database: tauri::State<'_, DatabaseState>,
    minutes: u32,
) -> Result<TodayDashboardData, ReminderTemplateCommandError> {
    snooze_next_reminder_command(runtime, templates, database, minutes)
}

#[tauri::command]
fn skip_next_reminder(
    runtime: tauri::State<'_, AppRuntimeState>,
    templates: tauri::State<'_, ReminderTemplateState>,
    database: tauri::State<'_, DatabaseState>,
) -> Result<TodayDashboardData, ReminderTemplateCommandError> {
    skip_next_reminder_command(runtime, templates, database)
}

#[tauri::command]
fn get_calendar_overview(
    runtime: tauri::State<'_, AppRuntimeState>,
    templates: tauri::State<'_, ReminderTemplateState>,
    selected_date: String,
) -> Result<CalendarOverviewData, ReminderTemplateCommandError> {
    get_calendar_overview_command(runtime, templates, selected_date)
}

#[tauri::command]
fn create_calendar_event(
    runtime: tauri::State<'_, AppRuntimeState>,
    templates: tauri::State<'_, ReminderTemplateState>,
    database: tauri::State<'_, DatabaseState>,
    title: String,
    message: String,
    selected_date: String,
    time: String,
) -> Result<CalendarOverviewData, ReminderTemplateCommandError> {
    create_calendar_event_command(runtime, templates, database, title, message, selected_date, time)
}

#[tauri::command]
fn get_settings(
    runtime: tauri::State<'_, AppRuntimeState>,
) -> Result<SettingsViewData, ReminderTemplateCommandError> {
    get_settings_command(runtime)
}

#[tauri::command]
fn update_settings(
    runtime: tauri::State<'_, AppRuntimeState>,
    database: tauri::State<'_, DatabaseState>,
    default_grace_minutes: i32,
    startup_with_windows: bool,
    close_to_tray_on_close: bool,
) -> Result<SettingsViewData, ReminderTemplateCommandError> {
    update_settings_command(
        runtime,
        database,
        default_grace_minutes,
        startup_with_windows,
        close_to_tray_on_close,
    )
}

fn main() {
    let db_config = load_db_config().expect("database config should load");
    let pool = initialize_database(&db_config.database_url).expect("database should initialize");

    let seed_templates = seed_reminder_templates().list();
    let seed_occurrence_items = seed_occurrences();
    let seed_settings = default_app_settings();

    bootstrap_defaults(&pool, &seed_templates, &seed_occurrence_items, &seed_settings)
        .expect("database bootstrap should succeed");

    let template_repository = load_template_repository(&pool).expect("templates should load from sqlite");
    let occurrences = load_occurrences(&pool).expect("occurrences should load from sqlite");
    let action_logs = load_action_logs(&pool).expect("action logs should load from sqlite");
    let settings = load_settings(&pool).expect("settings should load from sqlite");

    tauri::Builder::default()
        .manage(DatabaseState::new(pool))
        .manage(ReminderTemplateState::new(template_repository))
        .manage(AppRuntimeState::new(occurrences, action_logs, settings))
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let runtime = window.state::<AppRuntimeState>();
                let should_hide = runtime
                    .settings
                    .lock()
                    .map(|settings| settings.close_to_tray_on_close)
                    .unwrap_or(false);

                if should_hide {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            greet_command,
            list_reminder_templates,
            toggle_reminder_template,
            duplicate_reminder_template,
            create_reminder_template,
            update_reminder_template,
            get_today_dashboard,
            mark_next_reminder_completed,
            grace_next_reminder_ten_minutes,
            snooze_next_reminder,
            skip_next_reminder,
            get_calendar_overview,
            create_calendar_event,
            get_settings,
            update_settings
        ])
        .run(tauri::generate_context!())
        .expect("failed to run ScheduleReminder")
}
