use schedule_reminder::commands::app::{
    create_calendar_event_command, default_app_settings, duplicate_reminder_template_command,
    get_calendar_overview_command, get_settings_command, get_today_dashboard_command, greet,
    list_reminder_templates_command, mark_next_reminder_completed_command, seed_occurrences,
    seed_reminder_templates, toggle_reminder_template_command, update_settings_command,
};
use schedule_reminder::commands::app::{
    CalendarOverviewData, ReminderTemplateCommandError, ReminderTemplateListItem, SettingsViewData,
    TodayDashboardData,
};
use schedule_reminder::state::app_runtime::AppRuntimeState;
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
    id: String,
    enabled: bool,
) -> Result<ReminderTemplateListItem, ReminderTemplateCommandError> {
    toggle_reminder_template_command(state, id, enabled)
}

#[tauri::command]
fn duplicate_reminder_template(
    state: tauri::State<'_, ReminderTemplateState>,
    id: String,
) -> Result<ReminderTemplateListItem, ReminderTemplateCommandError> {
    duplicate_reminder_template_command(state, id)
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
) -> Result<TodayDashboardData, ReminderTemplateCommandError> {
    mark_next_reminder_completed_command(runtime, templates)
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
    title: String,
    selected_date: String,
    time: String,
) -> Result<CalendarOverviewData, ReminderTemplateCommandError> {
    create_calendar_event_command(runtime, templates, title, selected_date, time)
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
    default_grace_minutes: i32,
    startup_with_windows: bool,
    close_to_tray_on_close: bool,
) -> Result<SettingsViewData, ReminderTemplateCommandError> {
    update_settings_command(
        runtime,
        default_grace_minutes,
        startup_with_windows,
        close_to_tray_on_close,
    )
}

fn main() {
    tauri::Builder::default()
        .manage(ReminderTemplateState::new(seed_reminder_templates()))
        .manage(AppRuntimeState::new(
            seed_occurrences(),
            default_app_settings(),
        ))
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
            get_today_dashboard,
            mark_next_reminder_completed,
            get_calendar_overview,
            create_calendar_event,
            get_settings,
            update_settings
        ])
        .run(tauri::generate_context!())
        .expect("failed to run ScheduleReminder")
}
