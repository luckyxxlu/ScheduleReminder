use crate::models::reminder_action_log::ReminderActionLog;
use crate::models::reminder_occurrence::ReminderOccurrence;
use crate::models::reminder_template::{ReminderEventType, ReminderTemplate};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventExecutionError {
    InvalidTextPayload,
    InvalidSystemActionPayload,
    UnsupportedSystemAction,
    MissingConfirmation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationPayload {
    pub title: String,
    pub body: String,
    pub actions: Vec<String>,
}

pub fn build_notification_payload(
    template: &ReminderTemplate,
    occurrence: &ReminderOccurrence,
) -> Result<NotificationPayload, EventExecutionError> {
    match template.event_type {
        ReminderEventType::Text => {
            let message = extract_payload_value(&template.event_payload_json, "message")
                .ok_or(EventExecutionError::InvalidTextPayload)?;

            Ok(NotificationPayload {
                title: template.title.clone(),
                body: format!("{} | 原定时间 {}", message, occurrence.scheduled_at),
                actions: vec!["完成".to_string(), "延后".to_string(), "打开详情".to_string()],
            })
        }
        ReminderEventType::SystemAction => {
            let action = extract_payload_value(&template.event_payload_json, "action")
                .ok_or(EventExecutionError::InvalidSystemActionPayload)?;
            let message = extract_payload_value(&template.event_payload_json, "message")
                .ok_or(EventExecutionError::InvalidSystemActionPayload)?;

            if action != "shutdown" {
                return Err(EventExecutionError::UnsupportedSystemAction);
            }

            Ok(NotificationPayload {
                title: template.title.clone(),
                body: format!("{} | 将在确认后执行关机", message),
                actions: vec!["延后".to_string(), "取消".to_string(), "打开详情".to_string()],
            })
        }
    }
}

pub fn confirm_system_action(
    template: &ReminderTemplate,
    occurrence: &ReminderOccurrence,
    confirmed: bool,
    now: &str,
) -> Result<ReminderActionLog, EventExecutionError> {
    let action = extract_payload_value(&template.event_payload_json, "action")
        .ok_or(EventExecutionError::InvalidSystemActionPayload)?;

    if action != "shutdown" {
        return Err(EventExecutionError::UnsupportedSystemAction);
    }

    if !confirmed {
        return Err(EventExecutionError::MissingConfirmation);
    }

    Ok(ReminderActionLog {
        id: format!("log_system_action_{}", occurrence.id),
        occurrence_id: occurrence.id.clone(),
        action: "shutdown_confirmed".to_string(),
        action_at: now.to_string(),
        payload_json: Some(r#"{"action":"shutdown"}"#.to_string()),
    })
}

fn extract_payload_value(payload_json: &str, key: &str) -> Option<String> {
    let pattern = format!(r#""{key}":""#);
    let start = payload_json.find(&pattern)? + pattern.len();
    let rest = &payload_json[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

#[cfg(test)]
mod tests {
    use crate::models::reminder_occurrence::ReminderOccurrence;
    use crate::models::reminder_template::{ReminderEventType, ReminderTemplate};

    use super::{build_notification_payload, confirm_system_action, EventExecutionError};

    fn occurrence() -> ReminderOccurrence {
        ReminderOccurrence {
            id: "occ_1".to_string(),
            template_id: "tpl_1".to_string(),
            scheduled_at: "2026-04-22 22:30:00".to_string(),
            grace_deadline_at: "2026-04-22 22:45:00".to_string(),
            snoozed_until: None,
            status: "grace".to_string(),
            handled_at: None,
        }
    }

    fn text_template() -> ReminderTemplate {
        ReminderTemplate {
            id: "tpl_1".to_string(),
            title: "喝水提醒".to_string(),
            category: Some("health".to_string()),
            event_type: ReminderEventType::Text,
            event_payload_json: r#"{"message":"喝水时间到了"}"#.to_string(),
            repeat_rule_json: r#"{"type":"daily","interval":1}"#.to_string(),
            default_grace_minutes: 10,
            notify_sound: true,
            note: None,
            enabled: true,
        }
    }

    fn shutdown_template() -> ReminderTemplate {
        ReminderTemplate {
            id: "tpl_2".to_string(),
            title: "关机提醒".to_string(),
            category: Some("system".to_string()),
            event_type: ReminderEventType::SystemAction,
            event_payload_json: r#"{"action":"shutdown","message":"到时间了，准备关机休息"}"#.to_string(),
            repeat_rule_json: r#"{"type":"daily","interval":1}"#.to_string(),
            default_grace_minutes: 10,
            notify_sound: true,
            note: None,
            enabled: true,
        }
    }

    #[test]
    fn builds_text_notification_payload() {
        let payload = build_notification_payload(&text_template(), &occurrence())
            .expect("text payload should build");

        assert!(payload.body.contains("喝水时间到了"));
        assert_eq!(payload.actions, vec!["完成", "延后", "打开详情"]);
    }

    #[test]
    fn builds_shutdown_notification_payload() {
        let payload = build_notification_payload(&shutdown_template(), &occurrence())
            .expect("shutdown payload should build");

        assert!(payload.body.contains("将在确认后执行关机"));
        assert_eq!(payload.actions, vec!["延后", "取消", "打开详情"]);
    }

    #[test]
    fn rejects_invalid_text_payload() {
        let mut template = text_template();
        template.event_payload_json = r#"{"foo":"bar"}"#.to_string();

        let error = build_notification_payload(&template, &occurrence())
            .expect_err("invalid text payload should fail");

        assert_eq!(error, EventExecutionError::InvalidTextPayload);
    }

    #[test]
    fn confirms_shutdown_action_only_when_confirmed() {
        let log = confirm_system_action(&shutdown_template(), &occurrence(), true, "2026-04-22 22:31:00")
            .expect("confirmed shutdown should succeed");

        assert_eq!(log.action, "shutdown_confirmed");
    }

    #[test]
    fn rejects_shutdown_without_confirmation() {
        let error = confirm_system_action(&shutdown_template(), &occurrence(), false, "2026-04-22 22:31:00")
            .expect_err("shutdown without confirmation should fail");

        assert_eq!(error, EventExecutionError::MissingConfirmation);
    }

    #[test]
    fn rejects_unsupported_system_action() {
        let mut template = shutdown_template();
        template.event_payload_json = r#"{"action":"restart","message":"重启设备"}"#.to_string();

        let error = build_notification_payload(&template, &occurrence())
            .expect_err("unsupported action should fail");

        assert_eq!(error, EventExecutionError::UnsupportedSystemAction);
    }
}
