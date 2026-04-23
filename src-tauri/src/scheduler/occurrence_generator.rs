use std::collections::HashSet;

use chrono::{Datelike, Duration, NaiveDate};

use crate::models::reminder_occurrence::{ReminderOccurrence, ReminderOccurrenceError};
use crate::models::reminder_template::ReminderTemplate;

use super::repeat_rule::{parse_repeat_rule, RepeatRule};

pub fn generate_occurrences(
    template: &ReminderTemplate,
    start_date: &str,
    time: &str,
    count: usize,
) -> Result<Vec<ReminderOccurrence>, ReminderOccurrenceError> {
    let rule = parse_repeat_rule(&template.repeat_rule_json)
        .map_err(|_| ReminderOccurrenceError::InvalidRepeatRule)?;

    let dates = generate_dates(&rule, start_date, count);
    let mut occurrences = Vec::new();
    let mut keys = HashSet::new();

    for (index, date) in dates.into_iter().enumerate() {
        let scheduled_at = if time.matches(':').count() == 2 {
            format!("{date} {time}")
        } else {
            format!("{date} {time}:00")
        };

        if !keys.insert((template.id.clone(), scheduled_at.clone())) {
            return Err(ReminderOccurrenceError::DuplicateOccurrence);
        }

        occurrences.push(ReminderOccurrence {
            id: format!("occ_{}_{}", template.id, index + 1),
            template_id: template.id.clone(),
            scheduled_at: scheduled_at.clone(),
            grace_deadline_at: scheduled_at,
            snoozed_until: None,
            status: "pending".to_string(),
            handled_at: None,
        });
    }

    Ok(occurrences)
}

fn generate_dates(rule: &RepeatRule, start_date: &str, count: usize) -> Vec<String> {
    match rule {
        RepeatRule::None => vec![start_date.to_string()],
        RepeatRule::Daily { interval } => (0..count)
            .map(|offset| shift_day(start_date, (*interval as usize) * offset))
            .collect(),
        RepeatRule::Workdays => {
            let mut result = Vec::new();
            let mut offset = 0;
            while result.len() < count {
                let candidate = shift_day(start_date, offset);
                let weekday = weekday_number(&candidate);
                if (1..=5).contains(&weekday) {
                    result.push(candidate);
                }
                offset += 1;
            }
            result
        }
        RepeatRule::Weekly { interval, weekdays } => {
            let mut result = Vec::new();
            let start_weekday = weekday_number(start_date);
            let base_day = day_number(start_date);

            for week in 0..count.max(1) * 2 {
                let week_offset = week * (*interval as usize) * 7;
                let mut week_candidates = Vec::new();

                for weekday in weekdays {
                    let mut day_offset = week_offset;
                    if *weekday as usize >= start_weekday {
                        day_offset += *weekday as usize - start_weekday;
                    } else {
                        day_offset += 7 - start_weekday + *weekday as usize;
                    }

                    let candidate = day_from_number(base_day + day_offset);
                    if candidate >= start_date.to_string() {
                        week_candidates.push(candidate);
                    }
                }

                week_candidates.sort();
                result.extend(week_candidates);

                if result.len() >= count {
                    result.truncate(count);
                    return result;
                }
            }

            result
        }
    }
}

fn shift_day(start_date: &str, offset: usize) -> String {
    let parsed = parse_date(start_date);
    (parsed + Duration::days(offset as i64))
        .format("%Y-%m-%d")
        .to_string()
}

fn day_number(date: &str) -> usize {
    let parsed = parse_date(date);
    parsed.num_days_from_ce() as usize
}

fn day_from_number(value: usize) -> String {
    NaiveDate::from_num_days_from_ce_opt(value as i32)
        .expect("day number should be valid")
        .format("%Y-%m-%d")
        .to_string()
}

fn weekday_number(date: &str) -> usize {
    parse_date(date).weekday().number_from_monday() as usize
}

fn parse_date(date: &str) -> NaiveDate {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("date should match %Y-%m-%d")
}

#[cfg(test)]
mod tests {
    use crate::models::reminder_occurrence::ReminderOccurrenceError;
    use crate::models::reminder_template::{ReminderEventType, ReminderTemplate};

    use super::generate_occurrences;

    fn template_with_rule(repeat_rule_json: &str) -> ReminderTemplate {
        ReminderTemplate {
            id: "tpl_1".to_string(),
            title: "喝水提醒".to_string(),
            category: Some("health".to_string()),
            event_type: ReminderEventType::Text,
            event_payload_json: r#"{"message":"喝水时间到了"}"#.to_string(),
            repeat_rule_json: repeat_rule_json.to_string(),
            default_grace_minutes: 10,
            notify_sound: true,
            note: None,
            enabled: true,
        }
    }

    #[test]
    fn generates_single_occurrence_for_none_rule() {
        let template = template_with_rule(r#"{"type":"none"}"#);

        let occurrences =
            generate_occurrences(&template, "2026-04-22", "08:00", 1).expect("should generate");

        assert_eq!(occurrences.len(), 1);
        assert_eq!(occurrences[0].scheduled_at, "2026-04-22 08:00:00");
    }

    #[test]
    fn generates_multiple_occurrences_for_daily_rule() {
        let template = template_with_rule(r#"{"type":"daily","interval":1}"#);

        let occurrences =
            generate_occurrences(&template, "2026-04-22", "08:00", 3).expect("should generate");

        assert_eq!(occurrences.len(), 3);
        assert_eq!(occurrences[0].scheduled_at, "2026-04-22 08:00:00");
        assert_eq!(occurrences[1].scheduled_at, "2026-04-23 08:00:00");
        assert_eq!(occurrences[2].scheduled_at, "2026-04-24 08:00:00");
    }

    #[test]
    fn generates_workday_occurrences() {
        let template = template_with_rule(r#"{"type":"workdays"}"#);

        let occurrences =
            generate_occurrences(&template, "2026-04-22", "08:00", 3).expect("should generate");

        assert_eq!(occurrences.len(), 3);
    }

    #[test]
    fn generates_daily_occurrences_across_month_boundary() {
        let template = template_with_rule(r#"{"type":"daily","interval":1}"#);

        let occurrences =
            generate_occurrences(&template, "2026-04-30", "08:00", 3).expect("should generate");

        assert_eq!(occurrences[0].scheduled_at, "2026-04-30 08:00:00");
        assert_eq!(occurrences[1].scheduled_at, "2026-05-01 08:00:00");
        assert_eq!(occurrences[2].scheduled_at, "2026-05-02 08:00:00");
    }

    #[test]
    fn skips_weekend_days_for_workday_occurrences() {
        let template = template_with_rule(r#"{"type":"workdays"}"#);

        let occurrences =
            generate_occurrences(&template, "2026-04-24", "08:00", 3).expect("should generate");

        assert_eq!(occurrences[0].scheduled_at, "2026-04-24 08:00:00");
        assert_eq!(occurrences[1].scheduled_at, "2026-04-27 08:00:00");
        assert_eq!(occurrences[2].scheduled_at, "2026-04-28 08:00:00");
    }

    #[test]
    fn generates_weekly_occurrences() {
        let template = template_with_rule(r#"{"type":"weekly","interval":1,"weekdays":[1,3,5]}"#);

        let occurrences =
            generate_occurrences(&template, "2026-04-22", "08:00", 4).expect("should generate");

        assert_eq!(occurrences.len(), 4);
    }

    #[test]
    fn generates_expected_weekly_dates_for_selected_weekdays() {
        let template = template_with_rule(r#"{"type":"weekly","interval":1,"weekdays":[1,3,5]}"#);

        let occurrences =
            generate_occurrences(&template, "2026-04-22", "08:00", 4).expect("should generate");

        assert_eq!(occurrences[0].scheduled_at, "2026-04-22 08:00:00");
        assert_eq!(occurrences[1].scheduled_at, "2026-04-24 08:00:00");
        assert_eq!(occurrences[2].scheduled_at, "2026-04-27 08:00:00");
        assert_eq!(occurrences[3].scheduled_at, "2026-04-29 08:00:00");
    }

    #[test]
    fn rejects_invalid_repeat_rule() {
        let template = template_with_rule(r#"{"interval":1}"#);

        let error = generate_occurrences(&template, "2026-04-22", "08:00", 1)
            .expect_err("invalid repeat rule should fail");

        assert_eq!(
            error,
            ReminderOccurrenceError::InvalidRepeatRule
        );
    }

    #[test]
    fn rejects_duplicate_occurrences_when_rule_repeats_same_slot() {
        let template = template_with_rule(r#"{"type":"weekly","interval":1,"weekdays":[3,3]}"#);

        let error = generate_occurrences(&template, "2026-04-22", "08:00", 2)
            .expect_err("duplicate slot should fail");

        assert_eq!(error, ReminderOccurrenceError::DuplicateOccurrence);
    }
}
