use chrono::{Duration, NaiveDateTime};

use crate::models::reminder_action_log::ReminderActionLog;
use crate::models::reminder_occurrence::ReminderOccurrence;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraceError {
    InvalidSnoozeMinutes,
    InvalidTimestamp,
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

    let grace_minutes = active_grace_minutes(occurrence)?;
    let next_trigger_at = add_minutes(now, minutes)?;

    occurrence.status = "pending".to_string();
    occurrence.snoozed_until = Some(next_trigger_at.clone());
    occurrence.grace_deadline_at = add_minutes(&next_trigger_at, grace_minutes)?;

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

fn add_minutes(timestamp: &str, minutes: u32) -> Result<String, GraceError> {
    let parsed = parse_timestamp(timestamp)?;

    Ok((parsed + Duration::minutes(minutes as i64))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string())
}

fn active_grace_minutes(occurrence: &ReminderOccurrence) -> Result<u32, GraceError> {
    let active_trigger_at = occurrence
        .snoozed_until
        .as_deref()
        .unwrap_or(&occurrence.scheduled_at);
    let trigger_at = parse_timestamp(active_trigger_at)?;
    let grace_deadline = parse_timestamp(&occurrence.grace_deadline_at)?;
    let minutes = (grace_deadline - trigger_at).num_minutes();

    if minutes > 0 {
        return Ok(minutes as u32);
    }

    // Older persisted data may have snoozed_until and grace_deadline_at set to the same value.
    let scheduled_at = parse_timestamp(&occurrence.scheduled_at)?;
    let fallback_minutes = (grace_deadline - scheduled_at).num_minutes();

    if fallback_minutes < 0 {
        return Err(GraceError::NotInGrace);
    }

    Ok(fallback_minutes as u32)
}

fn parse_timestamp(timestamp: &str) -> Result<NaiveDateTime, GraceError> {
    NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S")
        .map_err(|_| GraceError::InvalidTimestamp)
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

        assert_eq!(occurrence.status, "pending");
        assert_eq!(occurrence.snoozed_until.as_deref(), Some("2026-04-22 08:10:00"));
        assert_eq!(occurrence.grace_deadline_at.as_str(), "2026-04-22 08:20:00");
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

    #[test]
    fn snooze_crosses_midnight_into_next_day() {
        let mut occurrence = grace_occurrence();

        snooze_occurrence(&mut occurrence, "2026-04-30 23:55:00", 10, "grace_10_minutes")
            .expect("cross-day snooze should succeed");

        assert_eq!(occurrence.status, "pending");
        assert_eq!(occurrence.snoozed_until.as_deref(), Some("2026-05-01 00:05:00"));
        assert_eq!(occurrence.grace_deadline_at.as_str(), "2026-05-01 00:15:00");
    }

    #[test]
    fn preserves_existing_grace_window_after_repeat_snooze() {
        let mut occurrence = grace_occurrence();
        occurrence.snoozed_until = Some("2026-04-22 08:15:00".to_string());
        occurrence.grace_deadline_at = "2026-04-22 08:25:00".to_string();

        snooze_occurrence(&mut occurrence, "2026-04-22 08:16:00", 5, "snoozed")
            .expect("repeat snooze should keep original grace window");

        assert_eq!(occurrence.snoozed_until.as_deref(), Some("2026-04-22 08:21:00"));
        assert_eq!(occurrence.grace_deadline_at.as_str(), "2026-04-22 08:31:00");
    }

    #[test]
    fn supports_legacy_grace_data_when_snoozed_until_matches_deadline() {
        let mut occurrence = grace_occurrence();
        occurrence.snoozed_until = Some("2026-04-22 08:10:00".to_string());
        occurrence.grace_deadline_at = "2026-04-22 08:10:00".to_string();

        snooze_occurrence(&mut occurrence, "2026-04-22 08:04:00", 5, "snoozed")
            .expect("legacy persisted grace window should be recovered");

        assert_eq!(occurrence.snoozed_until.as_deref(), Some("2026-04-22 08:09:00"));
        assert_eq!(occurrence.grace_deadline_at.as_str(), "2026-04-22 08:19:00");
    }

    #[test]
    fn rejects_invalid_legacy_timestamp_without_panicking() {
        let mut occurrence = grace_occurrence();
        occurrence.snoozed_until = Some("invalid".to_string());
        occurrence.grace_deadline_at = "invalid".to_string();

        let error = snooze_occurrence(&mut occurrence, "2026-04-22 08:04:00", 5, "snoozed")
            .expect_err("invalid persisted timestamps should be reported");

        assert_eq!(error, GraceError::InvalidTimestamp);
    }
}
