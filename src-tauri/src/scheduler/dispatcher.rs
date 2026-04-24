use crate::models::reminder_action_log::ReminderActionLog;
use crate::models::reminder_occurrence::ReminderOccurrence;
use crate::models::reminder_template::ReminderTemplate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerScanResult {
    pub triggered_ids: Vec<String>,
    pub missed_ids: Vec<String>,
    pub logs: Vec<ReminderActionLog>,
    pub resynced_occurrences: Vec<ReminderOccurrence>,
}

pub fn scan_occurrences(
    now: &str,
    occurrences: &mut [ReminderOccurrence],
) -> SchedulerScanResult {
    let mut triggered_ids = Vec::new();
    let mut missed_ids = Vec::new();
    let mut logs = Vec::new();

    for occurrence in occurrences.iter_mut() {
        if occurrence.status == "pending" {
            let trigger_at = occurrence
                .snoozed_until
                .as_deref()
                .unwrap_or(&occurrence.scheduled_at);

            if trigger_at <= now {
                occurrence.status = "grace".to_string();
                triggered_ids.push(occurrence.id.clone());
                logs.push(ReminderActionLog {
                    id: format!("log_dispatch_{}", occurrence.id),
                    occurrence_id: occurrence.id.clone(),
                    action: "notification_dispatched".to_string(),
                    action_at: now.to_string(),
                    payload_json: None,
                });
                continue;
            }
        }

        let expiry = occurrence.grace_deadline_at.clone();

        if occurrence.status == "grace" && expiry.as_str() < now {
            occurrence.status = "missed".to_string();
            missed_ids.push(occurrence.id.clone());
            logs.push(ReminderActionLog {
                id: format!("log_missed_{}", occurrence.id),
                occurrence_id: occurrence.id.clone(),
                action: "marked_missed".to_string(),
                action_at: now.to_string(),
                payload_json: None,
            });
        }
    }

    SchedulerScanResult {
        triggered_ids,
        missed_ids,
        logs,
        resynced_occurrences: vec![],
    }
}

pub fn resync_occurrences(
    templates: &[ReminderTemplate],
    existing_occurrences: &[ReminderOccurrence],
    start_date: &str,
    time: &str,
    count: usize,
) -> SchedulerScanResult {
    use super::occurrence_generator::generate_occurrences;

    let mut known = existing_occurrences
        .iter()
        .map(|item| (item.template_id.clone(), item.scheduled_at.clone()))
        .collect::<std::collections::HashSet<_>>();
    let mut resynced_occurrences = Vec::new();

    for template in templates.iter().filter(|item| item.enabled) {
        if let Ok(generated) = generate_occurrences(template, start_date, time, count) {
            for occurrence in generated {
                let key = (occurrence.template_id.clone(), occurrence.scheduled_at.clone());
                if known.insert(key) {
                    resynced_occurrences.push(occurrence);
                }
            }
        }
    }

    SchedulerScanResult {
        triggered_ids: vec![],
        missed_ids: vec![],
        logs: vec![],
        resynced_occurrences,
    }
}

#[cfg(test)]
mod tests {
    use crate::models::reminder_occurrence::ReminderOccurrence;
    use crate::models::reminder_template::{ReminderEventType, ReminderTemplate};

    use super::{resync_occurrences, scan_occurrences};

    fn pending_occurrence() -> ReminderOccurrence {
        ReminderOccurrence {
            id: "occ_1".to_string(),
            template_id: "tpl_1".to_string(),
            scheduled_at: "2026-04-22 08:00:00".to_string(),
            grace_deadline_at: "2026-04-22 08:10:00".to_string(),
            snoozed_until: None,
            status: "pending".to_string(),
            handled_at: None,
        }
    }

    fn enabled_template() -> ReminderTemplate {
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

    #[test]
    fn moves_pending_occurrence_into_grace_when_due() {
        let mut occurrences = vec![pending_occurrence()];

        let result = scan_occurrences("2026-04-22 08:00:00", &mut occurrences);

        assert_eq!(occurrences[0].status, "grace");
        assert_eq!(result.triggered_ids, vec!["occ_1".to_string()]);
    }

    #[test]
    fn writes_notification_dispatched_log_when_occurrence_triggers() {
        let mut occurrences = vec![pending_occurrence()];

        let result = scan_occurrences("2026-04-22 08:00:00", &mut occurrences);

        assert_eq!(result.logs.len(), 1);
        assert_eq!(result.logs[0].action, "notification_dispatched");
        assert_eq!(result.logs[0].occurrence_id, "occ_1");
    }

    #[test]
    fn marks_grace_occurrence_as_missed_after_deadline() {
        let mut occurrence = pending_occurrence();
        occurrence.status = "grace".to_string();
        let mut occurrences = vec![occurrence];

        let result = scan_occurrences("2026-04-22 08:10:01", &mut occurrences);

        assert_eq!(occurrences[0].status, "missed");
        assert_eq!(result.missed_ids, vec!["occ_1".to_string()]);
        assert_eq!(result.logs[0].action, "marked_missed");
    }

    #[test]
    fn keeps_grace_occurrence_active_at_exact_deadline() {
        let mut occurrence = pending_occurrence();
        occurrence.status = "grace".to_string();
        let mut occurrences = vec![occurrence];

        let result = scan_occurrences("2026-04-22 08:10:00", &mut occurrences);

        assert_eq!(occurrences[0].status, "grace");
        assert!(result.missed_ids.is_empty());
        assert!(result.logs.is_empty());
    }

    #[test]
    fn uses_grace_deadline_at_as_expiry_for_grace() {
        let mut occurrence = pending_occurrence();
        occurrence.status = "grace".to_string();
        occurrence.snoozed_until = Some("2026-04-22 08:20:00".to_string());
        occurrence.grace_deadline_at = "2026-04-22 08:25:00".to_string();
        let mut occurrences = vec![occurrence];

        let early = scan_occurrences("2026-04-22 08:24:00", &mut occurrences);
        assert_eq!(occurrences[0].status, "grace");
        assert!(early.missed_ids.is_empty());

        let late = scan_occurrences("2026-04-22 08:25:01", &mut occurrences);
        assert_eq!(occurrences[0].status, "missed");
        assert_eq!(late.missed_ids, vec!["occ_1".to_string()]);
    }

    #[test]
    fn triggers_pending_snoozed_occurrence_when_snooze_time_arrives() {
        let mut occurrence = pending_occurrence();
        occurrence.snoozed_until = Some("2026-04-22 08:05:00".to_string());
        occurrence.grace_deadline_at = "2026-04-22 08:15:00".to_string();
        let mut occurrences = vec![occurrence];

        let result = scan_occurrences("2026-04-22 08:05:00", &mut occurrences);

        assert_eq!(occurrences[0].status, "grace");
        assert_eq!(result.triggered_ids, vec!["occ_1".to_string()]);
        assert_eq!(result.logs[0].action, "notification_dispatched");
    }

    #[test]
    fn resync_generates_missing_occurrences_for_enabled_templates() {
        let template = enabled_template();
        let existing = vec![ReminderOccurrence {
            id: "occ_existing".to_string(),
            template_id: "tpl_1".to_string(),
            scheduled_at: "2026-04-22 08:00:00".to_string(),
            grace_deadline_at: "2026-04-22 08:10:00".to_string(),
            snoozed_until: None,
            status: "pending".to_string(),
            handled_at: None,
        }];

        let result = resync_occurrences(&[template], &existing, "2026-04-22", "08:00", 3);

        assert_eq!(result.resynced_occurrences.len(), 2);
        assert_eq!(
            result.resynced_occurrences[0].scheduled_at,
            "2026-04-23 08:00:00"
        );
    }

    #[test]
    fn resync_ignores_disabled_templates() {
        let mut template = enabled_template();
        template.enabled = false;

        let result = resync_occurrences(&[template], &[], "2026-04-22", "08:00", 3);

        assert!(result.resynced_occurrences.is_empty());
    }
}
