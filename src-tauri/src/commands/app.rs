use serde::Serialize;

use crate::db::reminder_template_repository::InMemoryReminderTemplateRepository;
use crate::models::reminder_occurrence::ReminderOccurrence;
use crate::models::reminder_template::{
    CreateReminderTemplateInput, ReminderEventType, ReminderTemplate, ReminderTemplateError,
};
use crate::settings::app_settings::{set_launch_on_startup, validate_settings, AppSettings, SettingsError};
use crate::state::app_runtime::AppRuntimeState;
use crate::state::reminder_templates::ReminderTemplateState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderTemplateListItem {
    pub id: String,
    pub title: String,
    pub schedule_summary: String,
    pub event_type_label: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderTemplateCommandError {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayDashboardData {
    pub next_reminder_title: String,
    pub next_reminder_time: String,
    pub next_reminder_message: String,
    pub highlighted_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEntry {
    pub id: String,
    pub date: String,
    pub time: String,
    pub title: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarDaySummary {
    pub date: String,
    pub reminder_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarOverviewData {
    pub selected_date: String,
    pub month_key: String,
    pub month_entries: Vec<CalendarDaySummary>,
    pub entries: Vec<CalendarEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsViewData {
    pub default_grace_minutes: i32,
    pub startup_with_windows: bool,
    pub close_to_tray_on_close: bool,
    pub quiet_hours_enabled: bool,
    pub quiet_hours_start: Option<String>,
    pub quiet_hours_end: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateSettingsInput {
    pub default_grace_minutes: i32,
    pub startup_with_windows: bool,
    pub close_to_tray_on_close: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateCalendarEventInput {
    pub title: String,
    pub selected_date: String,
    pub time: String,
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("你好，{name}。欢迎使用 ScheduleReminder。")
}

pub fn seed_reminder_templates() -> InMemoryReminderTemplateRepository {
    let mut repository = InMemoryReminderTemplateRepository::default();

    repository
        .create(CreateReminderTemplateInput {
            title: "喝水提醒".to_string(),
            category: Some("health".to_string()),
            event_type: ReminderEventType::Text,
            event_payload_json: r#"{"message":"喝水时间到了"}"#.to_string(),
            repeat_rule_json: r#"{"type":"daily","interval":1,"time":"08:00"}"#.to_string(),
            default_grace_minutes: 10,
            notify_sound: true,
            note: Some("工作日上午补水".to_string()),
        })
        .expect("seed template should be valid");

    repository
        .create(CreateReminderTemplateInput {
            title: "准备休息".to_string(),
            category: Some("rest".to_string()),
            event_type: ReminderEventType::Text,
            event_payload_json: r#"{"message":"准备休息，放下屏幕"}"#.to_string(),
            repeat_rule_json: r#"{"type":"daily","interval":1,"time":"22:30"}"#.to_string(),
            default_grace_minutes: 15,
            notify_sound: true,
            note: Some("睡前整理".to_string()),
        })
        .expect("seed template should be valid");

    repository
}

pub fn seed_occurrences() -> Vec<ReminderOccurrence> {
    vec![
        ReminderOccurrence {
            id: "occ_1".to_string(),
            template_id: "tpl_1".to_string(),
            scheduled_at: "2026-04-22 08:00:00".to_string(),
            grace_deadline_at: "2026-04-22 08:10:00".to_string(),
            snoozed_until: Some("2026-04-22 08:10:00".to_string()),
            status: "grace".to_string(),
            handled_at: None,
        },
        ReminderOccurrence {
            id: "occ_2".to_string(),
            template_id: "tpl_2".to_string(),
            scheduled_at: "2026-04-22 22:30:00".to_string(),
            grace_deadline_at: "2026-04-22 22:45:00".to_string(),
            snoozed_until: None,
            status: "pending".to_string(),
            handled_at: None,
        },
        ReminderOccurrence {
            id: "occ_3".to_string(),
            template_id: "tpl_1".to_string(),
            scheduled_at: "2026-04-23 08:00:00".to_string(),
            grace_deadline_at: "2026-04-23 08:10:00".to_string(),
            snoozed_until: None,
            status: "pending".to_string(),
            handled_at: None,
        },
    ]
}

pub fn default_app_settings() -> AppSettings {
    AppSettings {
        default_grace_minutes: 10,
        startup_with_windows: false,
        tray_enabled: true,
        close_to_tray_on_close: true,
        theme: "system".to_string(),
        quiet_hours_enabled: true,
        quiet_hours_start: Some("22:00".to_string()),
        quiet_hours_end: Some("07:00".to_string()),
    }
}

pub fn list_reminder_templates(
    state: &ReminderTemplateState,
) -> Result<Vec<ReminderTemplateListItem>, ReminderTemplateCommandError> {
    let repository = state
        .repository
        .lock()
        .map_err(|_| command_error("提醒模板状态不可用"))?;

    Ok(repository
        .list()
        .into_iter()
        .map(map_template_list_item)
        .collect())
}

pub fn toggle_reminder_template(
    state: &ReminderTemplateState,
    id: String,
    enabled: bool,
) -> Result<ReminderTemplateListItem, ReminderTemplateCommandError> {
    let mut repository = state
        .repository
        .lock()
        .map_err(|_| command_error("提醒模板状态不可用"))?;

    repository
        .toggle_enabled(&id, enabled)
        .map(map_template_list_item)
        .map_err(map_template_error)
}

pub fn duplicate_reminder_template(
    state: &ReminderTemplateState,
    id: String,
) -> Result<ReminderTemplateListItem, ReminderTemplateCommandError> {
    let mut repository = state
        .repository
        .lock()
        .map_err(|_| command_error("提醒模板状态不可用"))?;

    repository
        .duplicate(&id)
        .map(map_template_list_item)
        .map_err(map_template_error)
}

pub fn get_today_dashboard(
    runtime: &AppRuntimeState,
    templates: &ReminderTemplateState,
) -> Result<TodayDashboardData, ReminderTemplateCommandError> {
    let occurrences = runtime
        .occurrences
        .lock()
        .map_err(|_| command_error("提醒实例状态不可用"))?;
    let repository = templates
        .repository
        .lock()
        .map_err(|_| command_error("提醒模板状态不可用"))?;

    let next_occurrence = occurrences
        .iter()
        .filter(|item| item.status == "pending")
        .min_by(|left, right| left.scheduled_at.cmp(&right.scheduled_at))
        .ok_or_else(|| command_error("暂无下一条提醒"))?;

    let template = repository
        .get(&next_occurrence.template_id)
        .ok_or_else(|| command_error("下一条提醒缺少模板信息"))?;

    let highlighted_status = occurrences
        .iter()
        .find(|item| item.status == "grace")
        .map(|item| status_label(&item.status))
        .unwrap_or("已完成")
        .to_string();

    Ok(TodayDashboardData {
        next_reminder_title: template.title.clone(),
        next_reminder_time: time_part(&next_occurrence.scheduled_at).to_string(),
        next_reminder_message: format!("宽容 {} 分钟，支持稍后提醒与跳过今天。", template.default_grace_minutes),
        highlighted_status,
    })
}

pub fn get_calendar_overview(
    runtime: &AppRuntimeState,
    templates: &ReminderTemplateState,
    selected_date: String,
) -> Result<CalendarOverviewData, ReminderTemplateCommandError> {
    let occurrences = runtime
        .occurrences
        .lock()
        .map_err(|_| command_error("提醒实例状态不可用"))?;
    let repository = templates
        .repository
        .lock()
        .map_err(|_| command_error("提醒模板状态不可用"))?;

    let month_key = month_key(&selected_date);
    let month_entries = summarize_month_entries(&occurrences, &month_key);

    let entries = occurrences
        .iter()
        .filter(|item| date_part(&item.scheduled_at) == selected_date)
        .filter_map(|item| {
            repository.get(&item.template_id).map(|template| CalendarEntry {
                id: item.id.clone(),
                date: item.scheduled_at.clone(),
                time: time_part(&item.scheduled_at).to_string(),
                title: template.title.clone(),
                status: status_label(&item.status).to_string(),
            })
        })
        .collect::<Vec<_>>();

    Ok(CalendarOverviewData {
        selected_date,
        month_key,
        month_entries,
        entries,
    })
}

pub fn create_calendar_event(
    runtime: &AppRuntimeState,
    templates: &ReminderTemplateState,
    input: CreateCalendarEventInput,
) -> Result<CalendarOverviewData, ReminderTemplateCommandError> {
    if input.title.trim().is_empty() {
        return Err(command_error("日历事件标题不能为空"));
    }

    let settings = runtime
        .settings
        .lock()
        .map_err(|_| command_error("应用设置状态不可用"))?
        .clone();

    let created_template = {
        let mut repository = templates
            .repository
            .lock()
            .map_err(|_| command_error("提醒模板状态不可用"))?;

        repository
            .create(CreateReminderTemplateInput {
                title: input.title.trim().to_string(),
                category: Some("calendar".to_string()),
                event_type: ReminderEventType::Text,
                event_payload_json: format!(r#"{{"message":"{}"}}"#, input.title.trim()),
                repeat_rule_json: format!(r#"{{"type":"none","time":"{}"}}"#, input.time),
                default_grace_minutes: settings.default_grace_minutes,
                notify_sound: true,
                note: Some(format!("日历事件 {} {}", input.selected_date, input.time)),
            })
            .map_err(map_template_error)?
    };

    let mut occurrences = runtime
        .occurrences
        .lock()
        .map_err(|_| command_error("提醒实例状态不可用"))?;

    let next_id = occurrences.len() + 1;
    let scheduled_at = format!("{} {}:00", input.selected_date, input.time);
    let grace_deadline_at = add_minutes_to_timestamp(&scheduled_at, settings.default_grace_minutes.max(0) as u32);

    occurrences.push(ReminderOccurrence {
        id: format!("occ_{next_id}"),
        template_id: created_template.id,
        scheduled_at,
        grace_deadline_at,
        snoozed_until: None,
        status: "pending".to_string(),
        handled_at: None,
    });

    drop(occurrences);

    get_calendar_overview(runtime, templates, input.selected_date)
}

pub fn get_settings(runtime: &AppRuntimeState) -> Result<SettingsViewData, ReminderTemplateCommandError> {
    let settings = runtime
        .settings
        .lock()
        .map_err(|_| command_error("应用设置状态不可用"))?;

    Ok(map_settings_view(&settings))
}

pub fn update_settings(
    runtime: &AppRuntimeState,
    input: UpdateSettingsInput,
) -> Result<SettingsViewData, ReminderTemplateCommandError> {
    let mut settings = runtime
        .settings
        .lock()
        .map_err(|_| command_error("应用设置状态不可用"))?;

    settings.default_grace_minutes = input.default_grace_minutes;
    set_launch_on_startup(&mut settings, input.startup_with_windows);
    settings.close_to_tray_on_close = input.close_to_tray_on_close;
    validate_settings(&settings).map_err(map_settings_error)?;

    Ok(map_settings_view(&settings))
}

fn map_template_list_item(template: ReminderTemplate) -> ReminderTemplateListItem {
    ReminderTemplateListItem {
        id: template.id,
        title: template.title,
        schedule_summary: schedule_summary(&template.repeat_rule_json),
        event_type_label: event_type_label(&template.event_type).to_string(),
        enabled: template.enabled,
    }
}

fn schedule_summary(repeat_rule_json: &str) -> String {
    if repeat_rule_json.contains("daily") {
        if let Some(time) = extract_json_string_field(repeat_rule_json, "time") {
            return format!("每天 {time}");
        }

        return "每天".to_string();
    }

    if repeat_rule_json.contains("weekly") {
        return "每周".to_string();
    }

    if repeat_rule_json.contains("workdays") {
        return "工作日".to_string();
    }

    if repeat_rule_json.contains("none") {
        if let Some(time) = extract_json_string_field(repeat_rule_json, "time") {
            return format!("单次 {time}");
        }
    }

    "单次提醒".to_string()
}

fn summarize_month_entries(
    occurrences: &[ReminderOccurrence],
    month_key: &str,
) -> Vec<CalendarDaySummary> {
    let mut grouped = std::collections::BTreeMap::<String, usize>::new();

    for occurrence in occurrences
        .iter()
        .filter(|item| date_part(&item.scheduled_at).starts_with(month_key))
    {
        let date = date_part(&occurrence.scheduled_at).to_string();
        *grouped.entry(date).or_insert(0) += 1;
    }

    grouped
        .into_iter()
        .map(|(date, reminder_count)| CalendarDaySummary { date, reminder_count })
        .collect()
}

fn extract_json_string_field(payload: &str, field_name: &str) -> Option<String> {
    let search = format!("\"{field_name}\":\"");
    let start = payload.find(&search)? + search.len();
    let remaining = &payload[start..];
    let end = remaining.find('"')?;
    Some(remaining[..end].to_string())
}

fn event_type_label(event_type: &ReminderEventType) -> &'static str {
    match event_type {
        ReminderEventType::Text => "文本提醒",
        ReminderEventType::SystemAction => "系统动作",
    }
}

fn map_settings_view(settings: &AppSettings) -> SettingsViewData {
    SettingsViewData {
        default_grace_minutes: settings.default_grace_minutes,
        startup_with_windows: settings.startup_with_windows,
        close_to_tray_on_close: settings.close_to_tray_on_close,
        quiet_hours_enabled: settings.quiet_hours_enabled,
        quiet_hours_start: settings.quiet_hours_start.clone(),
        quiet_hours_end: settings.quiet_hours_end.clone(),
    }
}

fn map_settings_error(error: SettingsError) -> ReminderTemplateCommandError {
    let message = match error {
        SettingsError::InvalidGraceMinutes => "默认宽容时间不合法",
        SettingsError::IncompleteQuietHours => "免打扰时间配置不完整",
    };

    command_error(message)
}

fn date_part(timestamp: &str) -> &str {
    timestamp.split_once(' ').map(|(date, _)| date).unwrap_or(timestamp)
}

fn month_key(date: &str) -> String {
    date.split('-').take(2).collect::<Vec<_>>().join("-")
}

fn time_part(timestamp: &str) -> &str {
    timestamp
        .split_once(' ')
        .map(|(_, time)| &time[..5])
        .unwrap_or(timestamp)
}

fn add_minutes_to_timestamp(timestamp: &str, minutes: u32) -> String {
    let (date, time) = timestamp
        .split_once(' ')
        .expect("timestamp should contain date and time");
    let mut parts = time.split(':');
    let hour = parts.next().unwrap().parse::<u32>().unwrap();
    let minute = parts.next().unwrap().parse::<u32>().unwrap();
    let second = parts.next().unwrap();

    let total_minutes = hour * 60 + minute + minutes;
    let next_hour = (total_minutes / 60) % 24;
    let next_minute = total_minutes % 60;

    format!("{date} {next_hour:02}:{next_minute:02}:{second}")
}

fn status_label(status: &str) -> &str {
    match status {
        "grace" => "宽容中",
        "completed" => "已完成",
        "skipped" => "已跳过",
        "missed" => "已错过",
        _ => "待处理",
    }
}

fn map_template_error(error: ReminderTemplateError) -> ReminderTemplateCommandError {
    let message = match error {
        ReminderTemplateError::NotFound => "提醒模板不存在",
        ReminderTemplateError::EmptyTitle => "提醒标题不能为空",
        ReminderTemplateError::InvalidEventPayload => "提醒事件内容不合法",
        ReminderTemplateError::InvalidRepeatRule => "重复规则不合法",
        ReminderTemplateError::NegativeGraceMinutes => "宽容时间不能为负数",
    };

    command_error(message)
}

fn command_error(message: &str) -> ReminderTemplateCommandError {
    ReminderTemplateCommandError {
        message: message.to_string(),
    }
}

#[tauri::command]
pub fn list_reminder_templates_command(
    state: tauri::State<'_, ReminderTemplateState>,
) -> Result<Vec<ReminderTemplateListItem>, ReminderTemplateCommandError> {
    list_reminder_templates(&state)
}

#[tauri::command]
pub fn toggle_reminder_template_command(
    state: tauri::State<'_, ReminderTemplateState>,
    id: String,
    enabled: bool,
) -> Result<ReminderTemplateListItem, ReminderTemplateCommandError> {
    toggle_reminder_template(&state, id, enabled)
}

#[tauri::command]
pub fn duplicate_reminder_template_command(
    state: tauri::State<'_, ReminderTemplateState>,
    id: String,
) -> Result<ReminderTemplateListItem, ReminderTemplateCommandError> {
    duplicate_reminder_template(&state, id)
}

#[tauri::command]
pub fn get_today_dashboard_command(
    runtime: tauri::State<'_, AppRuntimeState>,
    templates: tauri::State<'_, ReminderTemplateState>,
) -> Result<TodayDashboardData, ReminderTemplateCommandError> {
    get_today_dashboard(&runtime, &templates)
}

#[tauri::command]
pub fn mark_next_reminder_completed_command(
    runtime: tauri::State<'_, AppRuntimeState>,
    templates: tauri::State<'_, ReminderTemplateState>,
) -> Result<TodayDashboardData, ReminderTemplateCommandError> {
    let mut occurrences = runtime
        .occurrences
        .lock()
        .map_err(|_| command_error("提醒实例状态不可用"))?;

    if let Some(occurrence) = occurrences.iter_mut().find(|item| item.status == "grace") {
        occurrence.status = "completed".to_string();
        occurrence.handled_at = Some(occurrence.scheduled_at.clone());
    }

    drop(occurrences);

    get_today_dashboard(&runtime, &templates)
}

#[tauri::command]
pub fn get_calendar_overview_command(
    runtime: tauri::State<'_, AppRuntimeState>,
    templates: tauri::State<'_, ReminderTemplateState>,
    selected_date: String,
) -> Result<CalendarOverviewData, ReminderTemplateCommandError> {
    get_calendar_overview(&runtime, &templates, selected_date)
}

#[tauri::command]
pub fn create_calendar_event_command(
    runtime: tauri::State<'_, AppRuntimeState>,
    templates: tauri::State<'_, ReminderTemplateState>,
    title: String,
    selected_date: String,
    time: String,
) -> Result<CalendarOverviewData, ReminderTemplateCommandError> {
    create_calendar_event(
        &runtime,
        &templates,
        CreateCalendarEventInput {
            title,
            selected_date,
            time,
        },
    )
}

#[tauri::command]
pub fn get_settings_command(
    runtime: tauri::State<'_, AppRuntimeState>,
) -> Result<SettingsViewData, ReminderTemplateCommandError> {
    get_settings(&runtime)
}

#[tauri::command]
pub fn update_settings_command(
    runtime: tauri::State<'_, AppRuntimeState>,
    default_grace_minutes: i32,
    startup_with_windows: bool,
    close_to_tray_on_close: bool,
) -> Result<SettingsViewData, ReminderTemplateCommandError> {
    update_settings(
        &runtime,
        UpdateSettingsInput {
            default_grace_minutes,
            startup_with_windows,
            close_to_tray_on_close,
        },
    )
}

#[cfg(test)]
mod tests {
    use std::sync::MutexGuard;

    use crate::commands::app::{
        create_calendar_event, default_app_settings, duplicate_reminder_template,
        get_calendar_overview, get_settings, get_today_dashboard, list_reminder_templates,
        seed_occurrences, seed_reminder_templates, toggle_reminder_template, update_settings,
        CreateCalendarEventInput, UpdateSettingsInput,
    };
    use crate::state::app_runtime::AppRuntimeState;
    use crate::state::reminder_templates::ReminderTemplateState;

    fn create_state() -> ReminderTemplateState {
        ReminderTemplateState::new(seed_reminder_templates())
    }

    fn create_runtime_state() -> AppRuntimeState {
        AppRuntimeState::new(seed_occurrences(), default_app_settings())
    }

    fn lock_state(state: &ReminderTemplateState) -> MutexGuard<'_, crate::db::reminder_template_repository::InMemoryReminderTemplateRepository> {
        state.repository.lock().expect("state lock should succeed")
    }

    #[test]
    fn lists_seeded_templates() {
        let state = create_state();

        let templates = list_reminder_templates(&state).expect("list should succeed");

        assert_eq!(templates.len(), 2);
        assert_eq!(templates[0].id, "tpl_1");
        assert_eq!(templates[0].schedule_summary, "每天 08:00");
        assert_eq!(templates[0].event_type_label, "文本提醒");
    }

    #[test]
    fn toggles_template_state() {
        let state = create_state();

        let updated = toggle_reminder_template(&state, "tpl_1".to_string(), false)
            .expect("toggle should succeed");

        assert!(!updated.enabled);
        assert!(!lock_state(&state).get("tpl_1").expect("template should exist").enabled);
    }

    #[test]
    fn duplicates_template() {
        let state = create_state();

        let duplicate =
            duplicate_reminder_template(&state, "tpl_1".to_string()).expect("duplicate should succeed");

        assert_eq!(duplicate.id, "tpl_3");
        assert_eq!(duplicate.title, "喝水提醒（副本）");
        assert_eq!(lock_state(&state).list().len(), 3);
    }

    #[test]
    fn returns_today_dashboard_data() {
        let runtime = create_runtime_state();
        let templates = create_state();

        let dashboard = get_today_dashboard(&runtime, &templates).expect("today data should load");

        assert_eq!(dashboard.next_reminder_title, "准备休息");
        assert_eq!(dashboard.next_reminder_time, "22:30");
        assert_eq!(dashboard.highlighted_status, "宽容中");
    }

    #[test]
    fn returns_calendar_entries_for_selected_date() {
        let runtime = create_runtime_state();
        let templates = create_state();

        let overview = get_calendar_overview(&runtime, &templates, "2026-04-22".to_string())
            .expect("calendar data should load");

        assert_eq!(overview.selected_date, "2026-04-22");
        assert_eq!(overview.month_key, "2026-04");
        assert_eq!(overview.month_entries.len(), 2);
        assert_eq!(overview.entries.len(), 2);
        assert_eq!(overview.entries[0].title, "喝水提醒");
    }

    #[test]
    fn creates_calendar_event_for_selected_date() {
        let runtime = create_runtime_state();
        let templates = create_state();

        let overview = create_calendar_event(
            &runtime,
            &templates,
            CreateCalendarEventInput {
                title: "深度工作".to_string(),
                selected_date: "2026-04-22".to_string(),
                time: "14:30".to_string(),
            },
        )
        .expect("calendar event should be created");

        assert_eq!(overview.entries.len(), 3);
        assert!(overview.entries.iter().any(|item| item.title == "深度工作"));
        assert!(lock_state(&templates)
            .list()
            .iter()
            .any(|item| item.title == "深度工作"));
    }

    #[test]
    fn reads_settings_view() {
        let runtime = create_runtime_state();

        let settings = get_settings(&runtime).expect("settings should load");

        assert_eq!(settings.default_grace_minutes, 10);
        assert!(!settings.startup_with_windows);
        assert!(settings.close_to_tray_on_close);
        assert_eq!(settings.quiet_hours_start.as_deref(), Some("22:00"));
    }

    #[test]
    fn updates_settings_view() {
        let runtime = create_runtime_state();

        let settings = update_settings(
            &runtime,
            UpdateSettingsInput {
                default_grace_minutes: 15,
                startup_with_windows: true,
                close_to_tray_on_close: false,
            },
        )
        .expect("settings should update");

        assert_eq!(settings.default_grace_minutes, 15);
        assert!(settings.startup_with_windows);
        assert!(!settings.close_to_tray_on_close);
    }
}
