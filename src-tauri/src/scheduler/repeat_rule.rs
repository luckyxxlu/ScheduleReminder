#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepeatRule {
    None,
    Daily { interval: u32 },
    Workdays,
    Weekly { interval: u32, weekdays: Vec<u8> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepeatRuleError {
    MissingType,
    UnsupportedType,
    MissingInterval,
    InvalidInterval,
    MissingWeekdays,
    InvalidWeekday,
}

pub fn parse_repeat_rule(input: &str) -> Result<RepeatRule, RepeatRuleError> {
    let normalized = input.replace(' ', "");

    let rule_type =
        extract_string_value(&normalized, "type").ok_or(RepeatRuleError::MissingType)?;

    match rule_type.as_str() {
        "none" => Ok(RepeatRule::None),
        "daily" => {
            let interval = extract_u32_value(&normalized, "interval")
                .ok_or(RepeatRuleError::MissingInterval)?;

            if interval == 0 {
                return Err(RepeatRuleError::InvalidInterval);
            }

            Ok(RepeatRule::Daily { interval })
        }
        "workdays" => Ok(RepeatRule::Workdays),
        "weekly" => {
            let interval = extract_u32_value(&normalized, "interval")
                .ok_or(RepeatRuleError::MissingInterval)?;

            if interval == 0 {
                return Err(RepeatRuleError::InvalidInterval);
            }

            let weekdays = extract_weekdays(&normalized).ok_or(RepeatRuleError::MissingWeekdays)?;

            if weekdays.is_empty() || weekdays.iter().any(|day| *day == 0 || *day > 7) {
                return Err(RepeatRuleError::InvalidWeekday);
            }

            Ok(RepeatRule::Weekly { interval, weekdays })
        }
        _ => Err(RepeatRuleError::UnsupportedType),
    }
}

fn extract_string_value(input: &str, key: &str) -> Option<String> {
    let pattern = format!(r#""{key}":""#);
    let start = input.find(&pattern)? + pattern.len();
    let rest = &input[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_u32_value(input: &str, key: &str) -> Option<u32> {
    let pattern = format!(r#""{key}":"#);
    let start = input.find(&pattern)? + pattern.len();
    let rest = &input[start..];
    let end = rest
        .find(|char: char| !char.is_ascii_digit())
        .unwrap_or(rest.len());

    rest[..end].parse::<u32>().ok()
}

fn extract_weekdays(input: &str) -> Option<Vec<u8>> {
    let pattern = r#""weekdays":["#;
    let start = input.find(pattern)? + pattern.len();
    let rest = &input[start..];
    let end = rest.find(']')?;
    let list = &rest[..end];

    if list.is_empty() {
        return Some(vec![]);
    }

    let mut weekdays = Vec::new();

    for part in list.split(',') {
        weekdays.push(part.parse::<u8>().ok()?);
    }

    Some(weekdays)
}

#[cfg(test)]
mod tests {
    use super::{parse_repeat_rule, RepeatRule, RepeatRuleError};

    #[test]
    fn parses_none_rule() {
        let rule = parse_repeat_rule(r#"{"type":"none"}"#).expect("none rule should parse");

        assert_eq!(rule, RepeatRule::None);
    }

    #[test]
    fn parses_daily_rule() {
        let rule =
            parse_repeat_rule(r#"{"type":"daily","interval":1}"#).expect("daily rule should parse");

        assert_eq!(rule, RepeatRule::Daily { interval: 1 });
    }

    #[test]
    fn parses_workdays_rule() {
        let rule = parse_repeat_rule(r#"{"type":"workdays"}"#).expect("workdays rule should parse");

        assert_eq!(rule, RepeatRule::Workdays);
    }

    #[test]
    fn parses_weekly_rule() {
        let rule = parse_repeat_rule(r#"{"type":"weekly","interval":1,"weekdays":[1,3,5]}"#)
            .expect("weekly rule should parse");

        assert_eq!(
            rule,
            RepeatRule::Weekly {
                interval: 1,
                weekdays: vec![1, 3, 5],
            }
        );
    }

    #[test]
    fn rejects_rule_without_type() {
        let error = parse_repeat_rule(r#"{"interval":1}"#).expect_err("missing type should fail");

        assert_eq!(error, RepeatRuleError::MissingType);
    }

    #[test]
    fn rejects_unsupported_type() {
        let error =
            parse_repeat_rule(r#"{"type":"monthly"}"#).expect_err("unsupported type should fail");

        assert_eq!(error, RepeatRuleError::UnsupportedType);
    }

    #[test]
    fn rejects_daily_rule_without_interval() {
        let error = parse_repeat_rule(r#"{"type":"daily"}"#)
            .expect_err("daily rule without interval should fail");

        assert_eq!(error, RepeatRuleError::MissingInterval);
    }

    #[test]
    fn rejects_zero_interval() {
        let error = parse_repeat_rule(r#"{"type":"daily","interval":0}"#)
            .expect_err("zero interval should fail");

        assert_eq!(error, RepeatRuleError::InvalidInterval);
    }

    #[test]
    fn rejects_weekly_rule_without_weekdays() {
        let error = parse_repeat_rule(r#"{"type":"weekly","interval":1}"#)
            .expect_err("weekly rule without weekdays should fail");

        assert_eq!(error, RepeatRuleError::MissingWeekdays);
    }

    #[test]
    fn rejects_invalid_weekday_values() {
        let error = parse_repeat_rule(r#"{"type":"weekly","interval":1,"weekdays":[0,8]}"#)
            .expect_err("invalid weekday values should fail");

        assert_eq!(error, RepeatRuleError::InvalidWeekday);
    }
}
