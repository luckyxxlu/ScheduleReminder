use chrono::{Duration, NaiveDateTime, NaiveTime};
use serde::Serialize;

use crate::db::persistence::{
    delete_occurrence_and_logs, delete_template, save_action_log, save_all_occurrences,
    save_all_templates, save_settings, PersistenceError,
};
use crate::db::reminder_template_repository::InMemoryReminderTemplateRepository;
use crate::models::reminder_action_log::ReminderActionLog;
use crate::models::reminder_occurrence::ReminderOccurrence;
use crate::models::reminder_template::{
    CreateReminderTemplateInput, ReminderEventType, ReminderTemplate, ReminderTemplateError,
};
use crate::scheduler::grace::{complete_occurrence, skip_occurrence, snooze_occurrence, GraceError};
use crate::settings::app_settings::{set_launch_on_startup, validate_settings, AppSettings, SettingsError};
use crate::state::app_runtime::AppRuntimeState;
use crate::state::database::DatabaseState;
use crate::state::reminder_templates::ReminderTemplateState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderTemplateListItem {
    pub id: String,
    pub title: String,
    pub message: String,
    pub category: Option<String>,
    pub repeat_rule_json: String,
    pub default_grace_minutes: i32,
    pub note: Option<String>,
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
    pub active_reminder_id: String,
    pub next_reminder_title: String,
    pub next_reminder_time: String,
    pub next_reminder_message: String,
    pub next_reminder_status: String,
    pub next_reminder_notification_state: String,
    pub next_reminder_grace_deadline: Option<String>,
    pub next_reminder_available_actions: Vec<String>,
    pub highlighted_status: String,
    pub today_timeline: Vec<TodayTimelineItem>,
    pub recent_actions: Vec<TodayActionItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayTimelineItem {
    pub id: String,
    pub time: String,
    pub title: String,
    pub message: String,
    pub status: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayActionItem {
    pub id: String,
    pub action_label: String,
    pub action_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEntry {
    pub id: String,
    pub date: String,
    pub time: String,
    pub title: String,
    pub message: String,
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
    pub recent_actions: Vec<TodayActionItem>,
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
    pub message: String,
    pub selected_date: String,
    pub time: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteCalendarEventInput {
    pub occurrence_id: String,
    pub selected_date: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateReminderTemplateCommandInput {
    pub title: String,
    pub message: String,
    pub category: Option<String>,
    pub repeat_rule_json: String,
    pub default_grace_minutes: i32,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateReminderTemplateCommandInput {
    pub id: String,
    pub title: String,
    pub message: String,
    pub category: Option<String>,
    pub repeat_rule_json: String,
    pub default_grace_minutes: i32,
    pub note: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TodayReminderAction {
    Complete,
    GraceTenMinutes,
    Snooze(u32),
    Skip,
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("你好，{name}。欢迎使用时间助手。")
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
    let today = chrono::Local::now().date_naive();
    let tomorrow = today + Duration::days(1);
    let today_key = today.format("%Y-%m-%d").to_string();
    let tomorrow_key = tomorrow.format("%Y-%m-%d").to_string();

    vec![
        ReminderOccurrence {
            id: "occ_1".to_string(),
            template_id: "tpl_1".to_string(),
            scheduled_at: format!("{today_key} 08:00:00"),
            grace_deadline_at: format!("{today_key} 08:10:00"),
            snoozed_until: Some(format!("{today_key} 08:10:00")),
            status: "grace".to_string(),
            handled_at: None,
        },
        ReminderOccurrence {
            id: "occ_2".to_string(),
            template_id: "tpl_2".to_string(),
            scheduled_at: format!("{today_key} 22:30:00"),
            grace_deadline_at: format!("{today_key} 22:45:00"),
            snoozed_until: None,
            status: "pending".to_string(),
            handled_at: None,
        },
        ReminderOccurrence {
            id: "occ_3".to_string(),
            template_id: "tpl_1".to_string(),
            scheduled_at: format!("{tomorrow_key} 08:00:00"),
            grace_deadline_at: format!("{tomorrow_key} 08:10:00"),
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
    database: &DatabaseState,
    id: String,
    enabled: bool,
) -> Result<ReminderTemplateListItem, ReminderTemplateCommandError> {
    let mut repository = state
        .repository
        .lock()
        .map_err(|_| command_error("提醒模板状态不可用"))?;

    let updated = repository
        .toggle_enabled(&id, enabled)
        .map_err(map_template_error)?;

    save_all_templates(&database.pool, &repository.list()).map_err(map_persistence_error)?;

    Ok(map_template_list_item(updated))
}

pub fn duplicate_reminder_template(
    state: &ReminderTemplateState,
    database: &DatabaseState,
    id: String,
) -> Result<ReminderTemplateListItem, ReminderTemplateCommandError> {
    let mut repository = state
        .repository
        .lock()
        .map_err(|_| command_error("提醒模板状态不可用"))?;

    let duplicated = repository
        .duplicate(&id)
        .map_err(map_template_error)?;

    save_all_templates(&database.pool, &repository.list()).map_err(map_persistence_error)?;

    Ok(map_template_list_item(duplicated))
}

pub fn create_reminder_template(
    state: &ReminderTemplateState,
    database: &DatabaseState,
    input: CreateReminderTemplateCommandInput,
) -> Result<ReminderTemplateListItem, ReminderTemplateCommandError> {
    let mut repository = state
        .repository
        .lock()
        .map_err(|_| command_error("提醒模板状态不可用"))?;

    let created = repository
        .create(CreateReminderTemplateInput {
            title: input.title.trim().to_string(),
            category: input.category.and_then(normalize_optional_text),
            event_type: ReminderEventType::Text,
            event_payload_json: format!(r#"{{"message":"{}"}}"#, escape_json_string(&input.message)),
            repeat_rule_json: input.repeat_rule_json,
            default_grace_minutes: input.default_grace_minutes,
            notify_sound: true,
            note: input.note.and_then(normalize_optional_text),
        })
        .map_err(map_template_error)?;

    save_all_templates(&database.pool, &repository.list()).map_err(map_persistence_error)?;

    Ok(map_template_list_item(created))
}

pub fn update_reminder_template(
    state: &ReminderTemplateState,
    database: &DatabaseState,
    input: UpdateReminderTemplateCommandInput,
) -> Result<ReminderTemplateListItem, ReminderTemplateCommandError> {
    let mut repository = state
        .repository
        .lock()
        .map_err(|_| command_error("提醒模板状态不可用"))?;

    let updated = repository
        .update(crate::models::reminder_template::UpdateReminderTemplateInput {
            id: input.id,
            title: input.title.trim().to_string(),
            category: input.category.and_then(normalize_optional_text),
            event_type: ReminderEventType::Text,
            event_payload_json: format!(r#"{{"message":"{}"}}"#, escape_json_string(&input.message)),
            repeat_rule_json: input.repeat_rule_json,
            default_grace_minutes: input.default_grace_minutes,
            notify_sound: true,
            note: input.note.and_then(normalize_optional_text),
            enabled: input.enabled,
        })
        .map_err(map_template_error)?;

    save_all_templates(&database.pool, &repository.list()).map_err(map_persistence_error)?;

    Ok(map_template_list_item(updated))
}

pub fn get_today_dashboard(
    runtime: &AppRuntimeState,
    templates: &ReminderTemplateState,
) -> Result<TodayDashboardData, ReminderTemplateCommandError> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let occurrences = runtime
        .occurrences
        .lock()
        .map_err(|_| command_error("提醒实例状态不可用"))?;
    let repository = templates
        .repository
        .lock()
        .map_err(|_| command_error("提醒模板状态不可用"))?;
    let action_logs = runtime
        .action_logs
        .lock()
        .map_err(|_| command_error("提醒日志状态不可用"))?;

    let today_occurrences = occurrences
        .iter()
        .filter(|item| date_part(&item.scheduled_at) == today.as_str())
        .collect::<Vec<_>>();
    let recent_actions: Vec<TodayActionItem> = action_logs
        .iter()
        .filter(|item| item.action_at.starts_with(&format!("{today} ")))
        .take(4)
        .map(map_today_action_item)
        .collect();
    let highlighted_status = highlighted_status_for_today(&today_occurrences).to_string();

    let next_occurrence = today_occurrences
        .iter()
        .copied()
        .filter(|item| matches!(item.status.as_str(), "grace" | "pending"))
        .min_by(|left, right| {
            occurrence_priority(&left.status)
                .cmp(&occurrence_priority(&right.status))
                .then_with(|| left.scheduled_at.cmp(&right.scheduled_at))
        });

    let next_occurrence = match next_occurrence {
        None => {
            let today_timeline = today_occurrences
                .iter()
                .filter_map(|item| {
                    repository.get(&item.template_id).map(|t| TodayTimelineItem {
                        id: item.id.clone(),
                        time: time_part(&item.scheduled_at).to_string(),
                        title: t.title.clone(),
                        message: extract_json_string_field(&t.event_payload_json, "message")
                            .unwrap_or_else(|| "提醒内容缺失".to_string()),
                        status: status_label(&item.status).to_string(),
                        is_active: false,
                    })
                })
                .collect::<Vec<_>>();

            return Ok(TodayDashboardData {
                active_reminder_id: String::new(),
                next_reminder_title: "暂无待处理提醒".to_string(),
                next_reminder_time: String::new(),
                next_reminder_message: "今天的提醒已全部处理完毕，或暂时没有安排。".to_string(),
                next_reminder_status: "暂无".to_string(),
                next_reminder_notification_state: "没有待处理的提醒，可以去提醒页添加新模板。".to_string(),
                next_reminder_grace_deadline: None,
                next_reminder_available_actions: Vec::new(),
                highlighted_status,
                today_timeline,
                recent_actions,
            });
        }
        Some(occ) => occ,
    };

    let template = repository
        .get(&next_occurrence.template_id)
        .ok_or_else(|| command_error("下一条提醒缺少模板信息"))?;

    let next_message = extract_json_string_field(&template.event_payload_json, "message")
        .unwrap_or_else(|| "提醒内容缺失".to_string());

    let timeline = today_occurrences
        .iter()
        .filter_map(|item| {
            repository.get(&item.template_id).map(|timeline_template| TodayTimelineItem {
                id: item.id.clone(),
                time: time_part(&item.scheduled_at).to_string(),
                title: timeline_template.title.clone(),
                message: extract_json_string_field(&timeline_template.event_payload_json, "message")
                    .unwrap_or_else(|| "提醒内容缺失".to_string()),
                status: status_label(&item.status).to_string(),
                is_active: item.id == next_occurrence.id,
            })
        })
        .collect::<Vec<_>>();

    let next_status = status_label(&next_occurrence.status).to_string();
    let notification_state = if next_occurrence.status == "grace" {
        "这条提醒已进入宽容时间，对应 Windows 通知已触发。".to_string()
    } else {
        "到达提醒时间后会发送 Windows 通知。".to_string()
    };

    let available_actions = if next_occurrence.status == "grace" {
        vec![
            "complete".to_string(),
            "grace_10_minutes".to_string(),
            "snooze".to_string(),
            "skip".to_string(),
        ]
    } else {
        Vec::new()
    };

    let next_reminder_time = if next_occurrence.status == "pending" {
        time_part(
            next_occurrence
                .snoozed_until
                .as_deref()
                .unwrap_or(&next_occurrence.scheduled_at),
        )
        .to_string()
    } else {
        time_part(&next_occurrence.scheduled_at).to_string()
    };

    Ok(TodayDashboardData {
        active_reminder_id: next_occurrence.id.clone(),
        next_reminder_title: template.title.clone(),
        next_reminder_time,
        next_reminder_message: next_message,
        next_reminder_status: next_status,
        next_reminder_notification_state: notification_state,
        next_reminder_grace_deadline: Some(time_part(&next_occurrence.grace_deadline_at).to_string()),
        next_reminder_available_actions: available_actions,
        highlighted_status,
        today_timeline: timeline,
        recent_actions,
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
    let action_logs = runtime
        .action_logs
        .lock()
        .map_err(|_| command_error("提醒日志状态不可用"))?;

    let month_key = month_key(&selected_date);
    let selected_date_prefix = month_day_prefix(&selected_date);
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
                message: extract_json_string_field(&template.event_payload_json, "message")
                    .unwrap_or_else(|| "提醒内容缺失".to_string()),
                status: status_label(&item.status).to_string(),
            })
        })
        .collect::<Vec<_>>();

    Ok(CalendarOverviewData {
        selected_date,
        month_key,
        month_entries,
        entries,
        recent_actions: action_logs
            .iter()
            .filter(|item| item.action_at.starts_with(&selected_date_prefix))
            .take(4)
            .map(map_today_action_item)
            .collect(),
    })
}

pub fn create_calendar_event(
    runtime: &AppRuntimeState,
    templates: &ReminderTemplateState,
    database: &DatabaseState,
    input: CreateCalendarEventInput,
) -> Result<CalendarOverviewData, ReminderTemplateCommandError> {
    if input.title.trim().is_empty() {
        return Err(command_error("日历事件标题不能为空"));
    }

    if input.message.trim().is_empty() {
        return Err(command_error("提醒内容不能为空"));
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
                event_payload_json: format!(r#"{{"message":"{}"}}"#, escape_json_string(input.message.trim())),
                repeat_rule_json: format!(r#"{{"type":"none","time":"{}"}}"#, input.time),
                default_grace_minutes: settings.default_grace_minutes,
                notify_sound: true,
                note: Some(format!("日历事件 {} {}", input.selected_date, input.time)),
            })
            .map_err(map_template_error)?
    };

    {
        let repository = templates
            .repository
            .lock()
            .map_err(|_| command_error("提醒模板状态不可用"))?;
        save_all_templates(&database.pool, &repository.list()).map_err(map_persistence_error)?;
    }

    let mut occurrences = runtime
        .occurrences
        .lock()
        .map_err(|_| command_error("提醒实例状态不可用"))?;

    let next_id = occurrences.len() + 1;
    let normalized_time = normalize_time_input(&input.time)?;
    let scheduled_at = format!("{} {normalized_time}", input.selected_date);
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

    save_all_occurrences(&database.pool, &occurrences).map_err(map_persistence_error)?;

    drop(occurrences);

    get_calendar_overview(runtime, templates, input.selected_date)
}

pub fn delete_calendar_event(
    runtime: &AppRuntimeState,
    templates: &ReminderTemplateState,
    database: &DatabaseState,
    input: DeleteCalendarEventInput,
) -> Result<CalendarOverviewData, ReminderTemplateCommandError> {
    let (removed_template_id, should_delete_template) = {
        let mut occurrences = runtime
            .occurrences
            .lock()
            .map_err(|_| command_error("提醒实例状态不可用"))?;

        let removed_occurrence = occurrences
            .iter()
            .find(|item| item.id == input.occurrence_id)
            .cloned()
            .ok_or_else(|| command_error("要删除的提醒事件不存在"))?;

        occurrences.retain(|item| item.id != input.occurrence_id);
        let template_id = removed_occurrence.template_id.clone();
        let should_delete_template = !occurrences.iter().any(|item| item.template_id == template_id);

        (template_id, should_delete_template)
    };

    {
        let mut action_logs = runtime
            .action_logs
            .lock()
            .map_err(|_| command_error("提醒日志状态不可用"))?;
        action_logs.retain(|item| item.occurrence_id != input.occurrence_id);
    }

    delete_occurrence_and_logs(&database.pool, &input.occurrence_id).map_err(map_persistence_error)?;

    {
        let mut repository = templates
            .repository
            .lock()
            .map_err(|_| command_error("提醒模板状态不可用"))?;

        let is_calendar_template = repository
            .get(&removed_template_id)
            .and_then(|template| template.category.as_deref())
            == Some("calendar");

        if should_delete_template && is_calendar_template {
            repository.delete(&removed_template_id).map_err(map_template_error)?;
            delete_template(&database.pool, &removed_template_id).map_err(map_persistence_error)?;
        }
    }

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
    database: &DatabaseState,
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
    save_settings(&database.pool, &settings).map_err(map_persistence_error)?;

    Ok(map_settings_view(&settings))
}

fn map_template_list_item(template: ReminderTemplate) -> ReminderTemplateListItem {
    ReminderTemplateListItem {
        id: template.id,
        title: template.title,
        message: extract_json_string_field(&template.event_payload_json, "message")
            .unwrap_or_else(|| "提醒内容缺失".to_string()),
        category: template.category.clone(),
        repeat_rule_json: template.repeat_rule_json.clone(),
        default_grace_minutes: template.default_grace_minutes,
        note: template.note.clone(),
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

fn normalize_optional_text(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn escape_json_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
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

fn highlighted_status_for_today(occurrences: &[&ReminderOccurrence]) -> String {
    if let Some(item) = occurrences.iter().find(|item| item.status == "grace") {
        return status_label(&item.status).to_string();
    }

    occurrences
        .iter()
        .filter(|item| matches!(item.status.as_str(), "completed" | "skipped" | "missed"))
        .max_by(|left, right| {
            left.handled_at
                .as_deref()
                .unwrap_or(&left.scheduled_at)
                .cmp(right.handled_at.as_deref().unwrap_or(&right.scheduled_at))
        })
        .map(|item| status_label(&item.status).to_string())
        .unwrap_or_else(|| "待处理".to_string())
}

fn map_today_action_item(log: &ReminderActionLog) -> TodayActionItem {
    TodayActionItem {
        id: log.id.clone(),
        action_label: action_label(&log.action).to_string(),
        action_at: log.action_at.clone(),
    }
}

fn action_label(action: &str) -> &str {
    match action {
        "completed" => "已完成",
        "grace_10_minutes" => "宽容 10 分钟",
        "snoozed" => "稍后提醒",
        "skipped" => "跳过今天",
        "notification_dispatched" => "已发送通知",
        "marked_missed" => "已错过",
        _ => "已处理",
    }
}

fn apply_today_reminder_action(
    runtime: &AppRuntimeState,
    templates: &ReminderTemplateState,
    database: &DatabaseState,
    action: TodayReminderAction,
) -> Result<TodayDashboardData, ReminderTemplateCommandError> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut occurrences = runtime
        .occurrences
        .lock()
        .map_err(|_| command_error("提醒实例状态不可用"))?;

    let occurrence = occurrences
        .iter_mut()
        .find(|item| item.status == "grace" && date_part(&item.scheduled_at) == today.as_str())
        .ok_or_else(|| command_error("当前没有宽容中的提醒"))?;
    let action_time = chrono::Local::now()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let log: ReminderActionLog = match action {
        TodayReminderAction::Complete => {
            complete_occurrence(occurrence, &action_time).map_err(map_grace_error)?
        }
        TodayReminderAction::GraceTenMinutes => snooze_occurrence(
            occurrence,
            &action_time,
            10,
            "grace_10_minutes",
        )
        .map_err(map_grace_error)?,
        TodayReminderAction::Snooze(minutes) => {
            snooze_occurrence(occurrence, &action_time, minutes, "snoozed")
                .map_err(map_grace_error)?
        }
        TodayReminderAction::Skip => skip_occurrence(occurrence, &action_time).map_err(map_grace_error)?,
    };

    save_all_occurrences(&database.pool, &occurrences).map_err(map_persistence_error)?;
    save_action_log(&database.pool, &log).map_err(map_persistence_error)?;

    drop(occurrences);

    runtime
        .action_logs
        .lock()
        .map_err(|_| command_error("提醒日志状态不可用"))?
        .insert(0, log);

    get_today_dashboard(runtime, templates)
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

fn month_day_prefix(date: &str) -> String {
    format!("{date} ")
}

fn time_part(timestamp: &str) -> &str {
    let Some((_, time)) = timestamp.split_once(' ') else {
        return timestamp;
    };

    if time.len() >= 8 && !time.ends_with(":00") {
        &time[..8]
    } else if time.len() >= 5 {
        &time[..5]
    } else {
        time
    }
}

fn normalize_time_input(time: &str) -> Result<String, ReminderTemplateCommandError> {
    NaiveTime::parse_from_str(time, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(time, "%H:%M"))
        .map(|parsed| parsed.format("%H:%M:%S").to_string())
        .map_err(|_| command_error("提醒时间格式不正确"))
}

fn occurrence_priority(status: &str) -> u8 {
    match status {
        "grace" => 0,
        "pending" => 1,
        _ => 2,
    }
}

fn add_minutes_to_timestamp(timestamp: &str, minutes: u32) -> String {
    let parsed = NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S")
        .expect("timestamp should match %Y-%m-%d %H:%M:%S");

    (parsed + Duration::minutes(minutes as i64))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
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

fn map_persistence_error(error: PersistenceError) -> ReminderTemplateCommandError {
    let message = match error {
        PersistenceError::DatabaseUnavailable => "数据库连接失败",
        PersistenceError::StatementExecutionFailed => "数据库写入失败",
    };

    command_error(message)
}

fn map_grace_error(error: GraceError) -> ReminderTemplateCommandError {
    let message = match error {
        GraceError::InvalidSnoozeMinutes => "稍后提醒仅支持 5/10/15/30 分钟",
        GraceError::InvalidTimestamp => "提醒时间数据格式无效，请重新保存提醒",
        GraceError::NotInGrace => "当前没有宽容中的提醒",
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
    database: tauri::State<'_, DatabaseState>,
    id: String,
    enabled: bool,
) -> Result<ReminderTemplateListItem, ReminderTemplateCommandError> {
    toggle_reminder_template(&state, &database, id, enabled)
}

#[tauri::command]
pub fn duplicate_reminder_template_command(
    state: tauri::State<'_, ReminderTemplateState>,
    database: tauri::State<'_, DatabaseState>,
    id: String,
) -> Result<ReminderTemplateListItem, ReminderTemplateCommandError> {
    duplicate_reminder_template(&state, &database, id)
}

#[tauri::command]
pub fn create_reminder_template_command(
    state: tauri::State<'_, ReminderTemplateState>,
    database: tauri::State<'_, DatabaseState>,
    title: String,
    message: String,
    category: Option<String>,
    repeat_rule_json: String,
    default_grace_minutes: i32,
    note: Option<String>,
) -> Result<ReminderTemplateListItem, ReminderTemplateCommandError> {
    create_reminder_template(
        &state,
        &database,
        CreateReminderTemplateCommandInput {
            title,
            message,
            category,
            repeat_rule_json,
            default_grace_minutes,
            note,
        },
    )
}

#[tauri::command]
pub fn update_reminder_template_command(
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
    update_reminder_template(
        &state,
        &database,
        UpdateReminderTemplateCommandInput {
            id,
            title,
            message,
            category,
            repeat_rule_json,
            default_grace_minutes,
            note,
            enabled,
        },
    )
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
    database: tauri::State<'_, DatabaseState>,
) -> Result<TodayDashboardData, ReminderTemplateCommandError> {
    mark_next_reminder_completed(&runtime, &templates, &database)
}

pub fn mark_next_reminder_completed(
    runtime: &AppRuntimeState,
    templates: &ReminderTemplateState,
    database: &DatabaseState,
) -> Result<TodayDashboardData, ReminderTemplateCommandError> {
    apply_today_reminder_action(runtime, templates, database, TodayReminderAction::Complete)
}

pub fn mark_next_reminder_grace_ten_minutes(
    runtime: &AppRuntimeState,
    templates: &ReminderTemplateState,
    database: &DatabaseState,
) -> Result<TodayDashboardData, ReminderTemplateCommandError> {
    apply_today_reminder_action(runtime, templates, database, TodayReminderAction::GraceTenMinutes)
}

pub fn snooze_next_reminder(
    runtime: &AppRuntimeState,
    templates: &ReminderTemplateState,
    database: &DatabaseState,
    minutes: u32,
) -> Result<TodayDashboardData, ReminderTemplateCommandError> {
    apply_today_reminder_action(runtime, templates, database, TodayReminderAction::Snooze(minutes))
}

pub fn skip_next_reminder(
    runtime: &AppRuntimeState,
    templates: &ReminderTemplateState,
    database: &DatabaseState,
) -> Result<TodayDashboardData, ReminderTemplateCommandError> {
    apply_today_reminder_action(runtime, templates, database, TodayReminderAction::Skip)
}

#[tauri::command]
pub fn grace_next_reminder_ten_minutes_command(
    runtime: tauri::State<'_, AppRuntimeState>,
    templates: tauri::State<'_, ReminderTemplateState>,
    database: tauri::State<'_, DatabaseState>,
) -> Result<TodayDashboardData, ReminderTemplateCommandError> {
    mark_next_reminder_grace_ten_minutes(&runtime, &templates, &database)
}

#[tauri::command]
pub fn snooze_next_reminder_command(
    runtime: tauri::State<'_, AppRuntimeState>,
    templates: tauri::State<'_, ReminderTemplateState>,
    database: tauri::State<'_, DatabaseState>,
    minutes: u32,
) -> Result<TodayDashboardData, ReminderTemplateCommandError> {
    snooze_next_reminder(&runtime, &templates, &database, minutes)
}

#[tauri::command]
pub fn skip_next_reminder_command(
    runtime: tauri::State<'_, AppRuntimeState>,
    templates: tauri::State<'_, ReminderTemplateState>,
    database: tauri::State<'_, DatabaseState>,
) -> Result<TodayDashboardData, ReminderTemplateCommandError> {
    skip_next_reminder(&runtime, &templates, &database)
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
    database: tauri::State<'_, DatabaseState>,
    title: String,
    message: String,
    selected_date: String,
    time: String,
) -> Result<CalendarOverviewData, ReminderTemplateCommandError> {
    create_calendar_event(
        &runtime,
        &templates,
        &database,
        CreateCalendarEventInput {
            title,
            message,
            selected_date,
            time,
        },
    )
}

#[tauri::command]
pub fn delete_calendar_event_command(
    runtime: tauri::State<'_, AppRuntimeState>,
    templates: tauri::State<'_, ReminderTemplateState>,
    database: tauri::State<'_, DatabaseState>,
    occurrence_id: String,
    selected_date: String,
) -> Result<CalendarOverviewData, ReminderTemplateCommandError> {
    delete_calendar_event(
        &runtime,
        &templates,
        &database,
        DeleteCalendarEventInput {
            occurrence_id,
            selected_date,
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
    database: tauri::State<'_, DatabaseState>,
    default_grace_minutes: i32,
    startup_with_windows: bool,
    close_to_tray_on_close: bool,
) -> Result<SettingsViewData, ReminderTemplateCommandError> {
    update_settings(
        &runtime,
        &database,
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
        create_calendar_event, default_app_settings, delete_calendar_event, duplicate_reminder_template,
        get_calendar_overview, get_settings, get_today_dashboard, list_reminder_templates,
        mark_next_reminder_completed, mark_next_reminder_grace_ten_minutes,
        seed_occurrences, seed_reminder_templates, skip_next_reminder, snooze_next_reminder,
        toggle_reminder_template, update_reminder_template, update_settings, CreateCalendarEventInput,
        CreateReminderTemplateCommandInput, DeleteCalendarEventInput, UpdateReminderTemplateCommandInput,
        UpdateSettingsInput,
    };
    use crate::db::migration::initialize_database;
    use crate::db::persistence::{bootstrap_defaults, load_occurrences};
    use crate::state::app_runtime::AppRuntimeState;
    use crate::state::database::DatabaseState;
    use crate::state::reminder_templates::ReminderTemplateState;

    fn create_state() -> ReminderTemplateState {
        ReminderTemplateState::new(seed_reminder_templates())
    }

    fn create_runtime_state() -> AppRuntimeState {
        AppRuntimeState::new(seed_occurrences(), vec![], default_app_settings())
    }

    fn create_database_state(test_name: &str) -> Option<DatabaseState> {
        let temp_dir = std::env::temp_dir();
        let database_path = temp_dir.join(format!(
            "schedule_reminder_cmd_{test_name}_{}.db",
            std::process::id()
        ));
        let database_url = format!("sqlite://{}", database_path.display());
        let pool = initialize_database(&database_url).ok()?;
        bootstrap_defaults(
            &pool,
            &seed_reminder_templates().list(),
            &seed_occurrences(),
            &default_app_settings(),
        )
        .ok()?;
        Some(DatabaseState::with_database_path(
            pool,
            database_path.to_string_lossy().to_string(),
        ))
    }

    fn cleanup_database_state(database: DatabaseState) {
        let database_path = database.database_path.clone().unwrap_or_default();
        if database_path.is_empty() {
            return;
        }
        drop(database);

        let _ = std::fs::remove_file(database_path);
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
        let Some(database) = create_database_state("toggle") else {
            return;
        };

        let updated = toggle_reminder_template(&state, &database, "tpl_1".to_string(), false)
            .expect("toggle should succeed");

        assert!(!updated.enabled);
        assert!(!lock_state(&state).get("tpl_1").expect("template should exist").enabled);
        cleanup_database_state(database);
    }

    #[test]
    fn duplicates_template() {
        let state = create_state();
        let Some(database) = create_database_state("duplicate") else {
            return;
        };

        let duplicate = duplicate_reminder_template(&state, &database, "tpl_1".to_string())
            .expect("duplicate should succeed");

        assert_eq!(duplicate.id, "tpl_3");
        assert_eq!(duplicate.title, "喝水提醒（副本）");
        assert_eq!(lock_state(&state).list().len(), 3);
        cleanup_database_state(database);
    }

    #[test]
    fn updates_template_content() {
        let state = create_state();
        let Some(database) = create_database_state("update_template") else {
            return;
        };

        let updated = update_reminder_template(
            &state,
            &database,
            UpdateReminderTemplateCommandInput {
                id: "tpl_1".to_string(),
                title: "补水提醒".to_string(),
                message: "现在去接一杯温水".to_string(),
                category: Some("health".to_string()),
                repeat_rule_json: r#"{"type":"daily","interval":1,"time":"09:30"}"#.to_string(),
                default_grace_minutes: 20,
                note: Some("上午第二次补水".to_string()),
                enabled: true,
            },
        )
        .expect("update should succeed");

        assert_eq!(updated.title, "补水提醒");
        assert_eq!(updated.message, "现在去接一杯温水");
        assert_eq!(updated.default_grace_minutes, 20);
        cleanup_database_state(database);
    }

    #[test]
    fn normalizes_optional_fields_when_creating_template() {
        let state = create_state();
        let Some(database) = create_database_state("create_template_normalize") else {
            return;
        };

        let created = crate::commands::app::create_reminder_template(
            &state,
            &database,
            CreateReminderTemplateCommandInput {
                title: "  深度工作  ".to_string(),
                message: "开始今天的专注时段".to_string(),
                category: Some("   ".to_string()),
                repeat_rule_json: r#"{"type":"daily","interval":1,"time":"14:30"}"#.to_string(),
                default_grace_minutes: 10,
                note: Some("  下午专注块  ".to_string()),
            },
        )
        .expect("create should succeed");

        assert_eq!(created.title, "深度工作");
        assert_eq!(created.category, None);
        assert_eq!(created.note.as_deref(), Some("下午专注块"));
        cleanup_database_state(database);
    }

    #[test]
    fn returns_today_dashboard_data() {
        let runtime = create_runtime_state();
        let templates = create_state();

        let dashboard = get_today_dashboard(&runtime, &templates).expect("today data should load");

        assert_eq!(dashboard.next_reminder_title, "喝水提醒");
        assert_eq!(dashboard.next_reminder_time, "08:00");
        assert_eq!(dashboard.next_reminder_message, "喝水时间到了");
        assert_eq!(dashboard.next_reminder_status, "宽容中");
        assert_eq!(dashboard.highlighted_status, "宽容中");
        assert_eq!(dashboard.next_reminder_available_actions.len(), 4);
        assert!(dashboard.today_timeline.iter().any(|item| item.title == "喝水提醒"));
        assert!(dashboard.recent_actions.is_empty());
    }

    #[test]
    fn returns_empty_dashboard_when_no_pending_reminders() {
        let completed_occurrences = seed_occurrences()
            .into_iter()
            .map(|mut occ| {
                occ.status = "completed".to_string();
                occ
            })
            .collect();
        let runtime = AppRuntimeState::new(completed_occurrences, vec![], default_app_settings());
        let templates = create_state();

        let dashboard = get_today_dashboard(&runtime, &templates).expect("empty dashboard should load without error");

        assert_eq!(dashboard.active_reminder_id, "");
        assert_eq!(dashboard.next_reminder_title, "暂无待处理提醒");
        assert!(dashboard.next_reminder_available_actions.is_empty());
        assert!(dashboard.recent_actions.is_empty());
    }

    #[test]
    fn returns_pending_dashboard_without_quick_actions() {
        let mut occurrences = seed_occurrences();
        occurrences[0].status = "pending".to_string();
        occurrences[0].snoozed_until = None;
        let runtime = AppRuntimeState::new(occurrences, vec![], default_app_settings());
        let templates = create_state();

        let dashboard = get_today_dashboard(&runtime, &templates).expect("today data should load");

        assert_eq!(dashboard.next_reminder_status, "待处理");
        assert!(dashboard.next_reminder_available_actions.is_empty());
        assert_eq!(
            dashboard.next_reminder_notification_state,
            "到达提醒时间后会发送 Windows 通知。"
        );
    }

    #[test]
    fn returns_pending_dashboard_time_from_snoozed_until() {
        let mut occurrences = seed_occurrences();
        occurrences[0].status = "pending".to_string();
        occurrences[0].snoozed_until = Some(format!("{} 08:05:00", chrono::Local::now().format("%Y-%m-%d")));
        occurrences[0].grace_deadline_at = format!("{} 08:15:00", chrono::Local::now().format("%Y-%m-%d"));
        let runtime = AppRuntimeState::new(occurrences, vec![], default_app_settings());
        let templates = create_state();

        let dashboard = get_today_dashboard(&runtime, &templates).expect("today data should load");

        assert_eq!(dashboard.next_reminder_status, "待处理");
        assert_eq!(dashboard.next_reminder_time, "08:05");
    }

    #[test]
    fn marks_next_reminder_as_completed() {
        let runtime = create_runtime_state();
        let templates = create_state();
        let Some(database) = create_database_state("complete_today") else {
            return;
        };

        let dashboard = mark_next_reminder_completed(&runtime, &templates, &database)
            .expect("mark complete should succeed");
        let persisted = load_occurrences(&database.pool).expect("occurrences should persist");

        assert_eq!(dashboard.highlighted_status, "已完成");
        assert!(persisted.iter().any(|item| item.id == "occ_1" && item.status == "completed"));
        cleanup_database_state(database);
    }

    #[test]
    fn applies_grace_ten_minutes_to_active_reminder() {
        let runtime = create_runtime_state();
        let templates = create_state();
        let Some(database) = create_database_state("grace_ten") else {
            return;
        };

        let dashboard = mark_next_reminder_grace_ten_minutes(&runtime, &templates, &database)
            .expect("grace ten should succeed");
        let persisted = load_occurrences(&database.pool).expect("occurrences should persist");
        let occurrence = persisted
            .iter()
            .find(|item| item.id == "occ_1")
            .expect("updated occurrence should exist");
        let snoozed_until = chrono::NaiveDateTime::parse_from_str(
            occurrence
                .snoozed_until
                .as_deref()
                .expect("snoozed_until should be set"),
            "%Y-%m-%d %H:%M:%S",
        )
        .expect("snoozed_until should parse");
        let grace_deadline = chrono::NaiveDateTime::parse_from_str(
            &occurrence.grace_deadline_at,
            "%Y-%m-%d %H:%M:%S",
        )
        .expect("grace_deadline should parse");
        let now = chrono::Local::now().naive_local();

        assert_eq!(dashboard.highlighted_status, "待处理");
        assert_eq!(occurrence.status, "pending");
        assert!((540..=660).contains(&(snoozed_until - now).num_seconds()));
        assert_eq!((grace_deadline - snoozed_until).num_minutes(), 10);
        cleanup_database_state(database);
    }

    #[test]
    fn snoozes_active_reminder_with_selected_minutes() {
        let runtime = create_runtime_state();
        let templates = create_state();
        let Some(database) = create_database_state("snooze") else {
            return;
        };

        let dashboard = snooze_next_reminder(&runtime, &templates, &database, 15)
            .expect("snooze should succeed");
        let persisted = load_occurrences(&database.pool).expect("occurrences should persist");
        let occurrence = persisted
            .iter()
            .find(|item| item.id == "occ_1")
            .expect("updated occurrence should exist");
        let snoozed_until = chrono::NaiveDateTime::parse_from_str(
            occurrence
                .snoozed_until
                .as_deref()
                .expect("snoozed_until should be set"),
            "%Y-%m-%d %H:%M:%S",
        )
        .expect("snoozed_until should parse");
        let grace_deadline = chrono::NaiveDateTime::parse_from_str(
            &occurrence.grace_deadline_at,
            "%Y-%m-%d %H:%M:%S",
        )
        .expect("grace_deadline should parse");
        let now = chrono::Local::now().naive_local();

        assert_eq!(dashboard.highlighted_status, "待处理");
        assert_eq!(occurrence.status, "pending");
        assert!((840..=960).contains(&(snoozed_until - now).num_seconds()));
        assert_eq!((grace_deadline - snoozed_until).num_minutes(), 10);
        cleanup_database_state(database);
    }

    #[test]
    fn snoozes_from_current_time_instead_of_previous_trigger_time() {
        let runtime = create_runtime_state();
        let templates = create_state();
        let Some(database) = create_database_state("snooze_now_base") else {
            return;
        };

        let dashboard = snooze_next_reminder(&runtime, &templates, &database, 5)
            .expect("snooze should succeed");
        let persisted = load_occurrences(&database.pool).expect("occurrences should persist");
        let occurrence = persisted
            .iter()
            .find(|item| item.id == "occ_1")
            .expect("updated occurrence should exist");

        assert_eq!(dashboard.highlighted_status, "待处理");
        assert_eq!(occurrence.status, "pending");

        let snoozed_until = chrono::NaiveDateTime::parse_from_str(
            occurrence
                .snoozed_until
                .as_deref()
                .expect("snoozed_until should be set"),
            "%Y-%m-%d %H:%M:%S",
        )
        .expect("snoozed_until should parse");
        let now = chrono::Local::now().naive_local();
        let diff_seconds = (snoozed_until - now).num_seconds();

        assert!((240..=360).contains(&diff_seconds));
        cleanup_database_state(database);
    }

    #[test]
    fn skips_active_grace_reminder_for_today() {
        let runtime = create_runtime_state();
        let templates = create_state();
        let Some(database) = create_database_state("skip") else {
            return;
        };

        let dashboard = skip_next_reminder(&runtime, &templates, &database)
            .expect("skip should succeed");
        let persisted = load_occurrences(&database.pool).expect("occurrences should persist");

        assert_eq!(dashboard.highlighted_status, "已跳过");
        assert!(persisted.iter().any(|item| item.id == "occ_1" && item.status == "skipped"));
        cleanup_database_state(database);
    }

    #[test]
    fn returns_calendar_entries_for_selected_date() {
        let runtime = create_runtime_state();
        let templates = create_state();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let month_key = chrono::Local::now().format("%Y-%m").to_string();

        let overview = get_calendar_overview(&runtime, &templates, today.clone())
            .expect("calendar data should load");

        assert_eq!(overview.selected_date, today);
        assert_eq!(overview.month_key, month_key);
        assert_eq!(overview.month_entries.len(), 2);
        assert_eq!(overview.entries.len(), 2);
        assert_eq!(overview.entries[0].title, "喝水提醒");
        assert_eq!(overview.entries[0].message, "喝水时间到了");
    }

    #[test]
    fn creates_calendar_event_for_selected_date() {
        let runtime = create_runtime_state();
        let templates = create_state();
        let Some(database) = create_database_state("calendar_event") else {
            return;
        };

        let overview = create_calendar_event(
            &runtime,
            &templates,
            &database,
            CreateCalendarEventInput {
                title: "深度工作".to_string(),
                message: "开始今天的专注时段".to_string(),
                selected_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
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
        cleanup_database_state(database);
    }

    #[test]
    fn creates_calendar_event_for_selected_date_with_seconds() {
        let runtime = create_runtime_state();
        let templates = create_state();
        let Some(database) = create_database_state("calendar_event_seconds") else {
            return;
        };

        let selected_date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let overview = create_calendar_event(
            &runtime,
            &templates,
            &database,
            CreateCalendarEventInput {
                title: "秒级提醒".to_string(),
                message: "精确到秒的提醒".to_string(),
                selected_date: selected_date.clone(),
                time: "14:30:45".to_string(),
            },
        )
        .expect("calendar event with seconds should be created");

        assert!(overview.entries.iter().any(|item| item.title == "秒级提醒" && item.time == "14:30:45"));

        let persisted = load_occurrences(&database.pool).expect("occurrences should persist");
        assert!(persisted.iter().any(|item| {
            item.template_id.starts_with("tpl_")
                && item.scheduled_at == format!("{selected_date} 14:30:45")
                && item.grace_deadline_at == format!("{selected_date} 14:40:45")
        }));

        cleanup_database_state(database);
    }

    #[test]
    fn creates_calendar_event_with_cross_day_grace_deadline() {
        let runtime = create_runtime_state();
        let templates = create_state();
        let Some(database) = create_database_state("calendar_event_cross_day") else {
            return;
        };

        {
            let mut settings = runtime.settings.lock().expect("settings lock should succeed");
            settings.default_grace_minutes = 10;
        }

        create_calendar_event(
            &runtime,
            &templates,
            &database,
            CreateCalendarEventInput {
                title: "睡前收尾".to_string(),
                message: "把今天的事情收一收".to_string(),
                selected_date: "2026-04-30".to_string(),
                time: "23:55".to_string(),
            },
        )
        .expect("calendar event should be created");

        let persisted = load_occurrences(&database.pool).expect("occurrences should persist");

        assert!(persisted.iter().any(|item| {
            item.scheduled_at == "2026-04-30 23:55:00"
                && item.grace_deadline_at == "2026-05-01 00:05:00"
        }));
        cleanup_database_state(database);
    }

    #[test]
    fn deletes_calendar_event_occurrence() {
        let runtime = create_runtime_state();
        let templates = create_state();
        let Some(database) = create_database_state("delete_calendar_event") else {
            return;
        };

        let overview = delete_calendar_event(
            &runtime,
            &templates,
            &database,
            DeleteCalendarEventInput {
                occurrence_id: "occ_1".to_string(),
                selected_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            },
        )
        .expect("calendar event should be deleted");

        let persisted_occurrences = load_occurrences(&database.pool).expect("occurrences should persist");

        assert!(!persisted_occurrences.iter().any(|item| item.id == "occ_1"));
        assert!(lock_state(&templates).get("tpl_1").is_some());
        assert!(!overview.entries.iter().any(|item| item.id == "occ_1"));
        cleanup_database_state(database);
    }

    #[test]
    fn rejects_calendar_event_without_title() {
        let runtime = create_runtime_state();
        let templates = create_state();
        let Some(database) = create_database_state("calendar_event_empty_title") else {
            return;
        };

        let error = create_calendar_event(
            &runtime,
            &templates,
            &database,
            CreateCalendarEventInput {
                title: "   ".to_string(),
                message: "开始今天的专注时段".to_string(),
                selected_date: "2026-04-22".to_string(),
                time: "14:30".to_string(),
            },
        )
        .expect_err("empty title should fail");

        assert_eq!(error.message, "日历事件标题不能为空");
        cleanup_database_state(database);
    }

    #[test]
    fn rejects_calendar_event_without_message() {
        let runtime = create_runtime_state();
        let templates = create_state();
        let Some(database) = create_database_state("calendar_event_empty_message") else {
            return;
        };

        let error = create_calendar_event(
            &runtime,
            &templates,
            &database,
            CreateCalendarEventInput {
                title: "深度工作".to_string(),
                message: "   ".to_string(),
                selected_date: "2026-04-22".to_string(),
                time: "14:30".to_string(),
            },
        )
        .expect_err("empty message should fail");

        assert_eq!(error.message, "提醒内容不能为空");
        cleanup_database_state(database);
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
        let Some(database) = create_database_state("settings") else {
            return;
        };

        let settings = update_settings(
            &runtime,
            &database,
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
        cleanup_database_state(database);
    }
}
