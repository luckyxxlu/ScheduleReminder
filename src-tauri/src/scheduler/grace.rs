use crate::models::reminder_action_log::ReminderActionLog;
use crate::models::reminder_occurrence::ReminderOccurrence;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraceError {
    InvalidSnoozeMinutes,
    NotInGrace,
}

pub fn snooze_occurrence(
    occurrence: &mut ReminderOccurrence,
    now: &str,
    minutes: u32,
    reason: &str,
) -> Result<ReminderActionLog, GraceError> {
    if occurrence.status != "grace" {
        return Err(GraceError::NotInGrace);
    }

    if !matches!(minutes, 5 | 10 | 15 | 30) {
        return Err(GraceError::InvalidSnoozeMinutes);
    }

    occurrence.snoozed_until = Some(add_minutes(now, minutes));

    Ok(ReminderActionLog {
        id: format!("log_snooze_{}_{}", occurrence.id, minutes),
        occurrence_id: occurrence.id.clone(),
        action: reason.to_string(),
        action_at: now.to_string(),
        payload_json: Some(format!(r#"{{"minutes":{minutes}}}"#)),
    })
}

pub fn complete_occurrence(
    occurrence: &mut ReminderOccurrence,
    now: &str,
) -> Result<ReminderActionLog, GraceError> {
    if occurrence.status != "grace" {
        return Err(GraceError::NotInGrace);
    }

    occurrence.status = "completed".to_string();
    occurrence.handled_at = Some(now.to_string());

    Ok(ReminderActionLog {
        id: format!("log_complete_{}", occurrence.id),
        occurrence_id: occurrence.id.clone(),
        action: "completed".to_string(),
        action_at: now.to_string(),
        payload_json: None,
    })
}

pub fn skip_occurrence(
    occurrence: &mut ReminderOccurrence,
    now: &str,
) -> Result<ReminderActionLog, GraceError> {
    if occurrence.status != "grace" {
        return Err(GraceError::NotInGrace);
    }

    occurrence.status = "skipped".to_string();
    occurrence.handled_at = Some(now.to_string());

    Ok(ReminderActionLog {
        id: format!("log_skip_{}", occurrence.id),
        occurrence_id: occurrence.id.clone(),
        action: "skipped".to_string(),
        action_at: now.to_string(),
        payload_json: None,
    })
}

fn add_minutes(timestamp: &str, minutes: u32) -> String {
    let (date, time) = timestamp.split_once(' ').expect("timestamp should contain date and time");
    let mut parts = time.split(':');
    let hour = parts.next().unwrap().parse::<u32>().unwrap();
    let minute = parts.next().unwrap().parse::<u32>().unwrap();
    let second = parts.next().unwrap();

    let total_minutes = hour * 60 + minute + minutes;
    let next_hour = (total_minutes / 60) % 24;
    let next_minute = total_minutes % 60;

    format!("{date} {next_hour:02}:{next_minute:02}:{second}")
}

#[cfg(test)]
mod tests {
    use crate::models::reminder_occurrence::ReminderOccurrence;

    use super::{complete_occurrence, skip_occurrence, snooze_occurrence, GraceError};

    fn grace_occurrence() -> ReminderOccurrence {
        ReminderOccurrence {
            id: "occ_1".to_string(),
            template_id: "tpl_1".to_string(),
            scheduled_at: "2026-04-22 08:00:00".to_string(),
            grace_deadline_at: "2026-04-22 08:10:00".to_string(),
            snoozed_until: None,
            status: "grace".to_string(),
            handled_at: None,
        }
    }

    #[test]
    fn applies_grace_ten_minutes_snooze() {
        let mut occurrence = grace_occurrence();

        let log = snooze_occurrence(&mut occurrence, "2026-04-22 08:00:00", 10, "grace_10_minutes")
            .expect("10 minute grace should succeed");

        assert_eq!(occurrence.snoozed_until.as_deref(), Some("2026-04-22 08:10:00"));
        assert_eq!(log.action, "grace_10_minutes");
    }

    #[test]
    fn applies_supported_snooze_options() {
        for minutes in [5_u32, 10, 15, 30] {
            let mut occurrence = grace_occurrence();
            let result = snooze_occurrence(&mut occurrence, "2026-04-22 08:00:00", minutes, "snoozed");
            assert!(result.is_ok());
        }
    }

    #[test]
    fn rejects_unsupported_snooze_options() {
        let mut occurrence = grace_occurrence();

        let error = snooze_occurrence(&mut occurrence, "2026-04-22 08:00:00", 20, "snoozed")
            .expect_err("unsupported snooze option should fail");

        assert_eq!(error, GraceError::InvalidSnoozeMinutes);
    }

    #[test]
    fn completes_occurrence_from_grace() {
        let mut occurrence = grace_occurrence();

        let log = complete_occurrence(&mut occurrence, "2026-04-22 08:03:00")
            .expect("complete should succeed");

        assert_eq!(occurrence.status, "completed");
        assert_eq!(occurrence.handled_at.as_deref(), Some("2026-04-22 08:03:00"));
        assert_eq!(log.action, "completed");
    }

    #[test]
    fn skips_occurrence_from_grace() {
        let mut occurrence = grace_occurrence();

        let log = skip_occurrence(&mut occurrence, "2026-04-22 08:03:00")
            .expect("skip should succeed");

        assert_eq!(occurrence.status, "skipped");
        assert_eq!(occurrence.handled_at.as_deref(), Some("2026-04-22 08:03:00"));
        assert_eq!(log.action, "skipped");
    }

    #[test]
    fn rejects_grace_actions_when_occurrence_is_not_in_grace() {
        let mut occurrence = grace_occurrence();
        occurrence.status = "pending".to_string();

        let error = complete_occurrence(&mut occurrence, "2026-04-22 08:03:00")
            .expect_err("complete outside grace should fail");

        assert_eq!(error, GraceError::NotInGrace);
    }
}
