use std::collections::HashMap;

use crate::models::reminder_template::{
    CreateReminderTemplateInput, ReminderEventType, ReminderTemplate, ReminderTemplateError,
    UpdateReminderTemplateInput,
};

#[derive(Default)]
pub struct InMemoryReminderTemplateRepository {
    items: HashMap<String, ReminderTemplate>,
    next_id: u64,
}

impl InMemoryReminderTemplateRepository {
    pub fn list(&self) -> Vec<ReminderTemplate> {
        let mut items = self.items.values().cloned().collect::<Vec<_>>();
        items.sort_by(|left, right| left.id.cmp(&right.id));
        items
    }

    pub fn create(
        &mut self,
        input: CreateReminderTemplateInput,
    ) -> Result<ReminderTemplate, ReminderTemplateError> {
        validate_title(&input.title)?;
        validate_event_payload(&input.event_type, &input.event_payload_json)?;
        validate_repeat_rule(&input.repeat_rule_json)?;
        validate_grace_minutes(input.default_grace_minutes)?;

        self.next_id += 1;
        let template = ReminderTemplate {
            id: format!("tpl_{}", self.next_id),
            title: input.title,
            category: input.category,
            event_type: input.event_type,
            event_payload_json: input.event_payload_json,
            repeat_rule_json: input.repeat_rule_json,
            default_grace_minutes: input.default_grace_minutes,
            notify_sound: input.notify_sound,
            note: input.note,
            enabled: true,
        };

        self.items.insert(template.id.clone(), template.clone());

        Ok(template)
    }

    pub fn update(
        &mut self,
        input: UpdateReminderTemplateInput,
    ) -> Result<ReminderTemplate, ReminderTemplateError> {
        validate_title(&input.title)?;
        validate_event_payload(&input.event_type, &input.event_payload_json)?;
        validate_repeat_rule(&input.repeat_rule_json)?;
        validate_grace_minutes(input.default_grace_minutes)?;

        let item = self
            .items
            .get_mut(&input.id)
            .ok_or(ReminderTemplateError::NotFound)?;

        item.title = input.title;
        item.category = input.category;
        item.event_type = input.event_type;
        item.event_payload_json = input.event_payload_json;
        item.repeat_rule_json = input.repeat_rule_json;
        item.default_grace_minutes = input.default_grace_minutes;
        item.notify_sound = input.notify_sound;
        item.note = input.note;
        item.enabled = input.enabled;

        Ok(item.clone())
    }

    pub fn delete(&mut self, id: &str) -> Result<(), ReminderTemplateError> {
        self.items
            .remove(id)
            .map(|_| ())
            .ok_or(ReminderTemplateError::NotFound)
    }

    pub fn toggle_enabled(
        &mut self,
        id: &str,
        enabled: bool,
    ) -> Result<ReminderTemplate, ReminderTemplateError> {
        let item = self
            .items
            .get_mut(id)
            .ok_or(ReminderTemplateError::NotFound)?;

        item.enabled = enabled;

        Ok(item.clone())
    }

    pub fn duplicate(&mut self, id: &str) -> Result<ReminderTemplate, ReminderTemplateError> {
        let source = self.items.get(id).ok_or(ReminderTemplateError::NotFound)?.clone();

        self.next_id += 1;

        let copy = ReminderTemplate {
            id: format!("tpl_{}", self.next_id),
            title: format!("{}（副本）", source.title),
            category: source.category,
            event_type: source.event_type,
            event_payload_json: source.event_payload_json,
            repeat_rule_json: source.repeat_rule_json,
            default_grace_minutes: source.default_grace_minutes,
            notify_sound: source.notify_sound,
            note: source.note,
            enabled: source.enabled,
        };

        self.items.insert(copy.id.clone(), copy.clone());

        Ok(copy)
    }

    pub fn get(&self, id: &str) -> Option<&ReminderTemplate> {
        self.items.get(id)
    }
}

fn validate_title(title: &str) -> Result<(), ReminderTemplateError> {
    if title.trim().is_empty() {
        return Err(ReminderTemplateError::EmptyTitle);
    }

    Ok(())
}

fn validate_event_payload(
    event_type: &ReminderEventType,
    event_payload_json: &str,
) -> Result<(), ReminderTemplateError> {
    let payload = event_payload_json.trim();

    match event_type {
        ReminderEventType::Text => {
            if !payload.contains("message") {
                return Err(ReminderTemplateError::InvalidEventPayload);
            }
        }
        ReminderEventType::SystemAction => {
            if !payload.contains("shutdown") && !payload.contains("action") {
                return Err(ReminderTemplateError::InvalidEventPayload);
            }
        }
    }

    Ok(())
}

fn validate_repeat_rule(repeat_rule_json: &str) -> Result<(), ReminderTemplateError> {
    if !repeat_rule_json.contains("type") {
        return Err(ReminderTemplateError::InvalidRepeatRule);
    }

    Ok(())
}

fn validate_grace_minutes(default_grace_minutes: i32) -> Result<(), ReminderTemplateError> {
    if default_grace_minutes < 0 {
        return Err(ReminderTemplateError::NegativeGraceMinutes);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::InMemoryReminderTemplateRepository;
    use crate::models::reminder_template::{
        CreateReminderTemplateInput, ReminderEventType, ReminderTemplateError,
        UpdateReminderTemplateInput,
    };

    fn create_text_input() -> CreateReminderTemplateInput {
        CreateReminderTemplateInput {
            title: "喝水提醒".to_string(),
            category: Some("health".to_string()),
            event_type: ReminderEventType::Text,
            event_payload_json: r#"{"message":"喝水时间到了"}"#.to_string(),
            repeat_rule_json: r#"{"type":"daily","interval":1}"#.to_string(),
            default_grace_minutes: 10,
            notify_sound: true,
            note: None,
        }
    }

    #[test]
    fn creates_reminder_template() {
        let mut repository = InMemoryReminderTemplateRepository::default();

        let created = repository.create(create_text_input()).expect("create should succeed");

        assert_eq!(created.id, "tpl_1");
        assert_eq!(created.title, "喝水提醒");
        assert!(created.enabled);
    }

    #[test]
    fn rejects_empty_title_when_creating_template() {
        let mut repository = InMemoryReminderTemplateRepository::default();
        let mut input = create_text_input();
        input.title = "   ".to_string();

        let error = repository.create(input).expect_err("empty title should fail");

        assert_eq!(error, ReminderTemplateError::EmptyTitle);
    }

    #[test]
    fn rejects_invalid_event_payload_for_text_template() {
        let mut repository = InMemoryReminderTemplateRepository::default();
        let mut input = create_text_input();
        input.event_payload_json = r#"{"foo":"bar"}"#.to_string();

        let error = repository
            .create(input)
            .expect_err("invalid payload should fail");

        assert_eq!(error, ReminderTemplateError::InvalidEventPayload);
    }

    #[test]
    fn updates_existing_template() {
        let mut repository = InMemoryReminderTemplateRepository::default();
        let created = repository.create(create_text_input()).expect("create should succeed");

        let updated = repository
            .update(UpdateReminderTemplateInput {
                id: created.id.clone(),
                title: "晚间喝水提醒".to_string(),
                category: Some("night".to_string()),
                event_type: ReminderEventType::Text,
                event_payload_json: r#"{"message":"晚上也记得喝水"}"#.to_string(),
                repeat_rule_json: r#"{"type":"daily","interval":1}"#.to_string(),
                default_grace_minutes: 15,
                notify_sound: false,
                note: Some("晚间模式".to_string()),
                enabled: true,
            })
            .expect("update should succeed");

        assert_eq!(updated.title, "晚间喝水提醒");
        assert_eq!(updated.default_grace_minutes, 15);
        assert!(!updated.notify_sound);
    }

    #[test]
    fn deletes_existing_template() {
        let mut repository = InMemoryReminderTemplateRepository::default();
        let created = repository.create(create_text_input()).expect("create should succeed");

        repository.delete(&created.id).expect("delete should succeed");

        assert!(repository.get(&created.id).is_none());
    }

    #[test]
    fn toggles_template_enabled_state() {
        let mut repository = InMemoryReminderTemplateRepository::default();
        let created = repository.create(create_text_input()).expect("create should succeed");

        let toggled = repository
            .toggle_enabled(&created.id, false)
            .expect("toggle should succeed");

        assert!(!toggled.enabled);
    }

    #[test]
    fn duplicates_template_with_new_identifier() {
        let mut repository = InMemoryReminderTemplateRepository::default();
        let created = repository.create(create_text_input()).expect("create should succeed");

        let duplicate = repository.duplicate(&created.id).expect("duplicate should succeed");

        assert_ne!(duplicate.id, created.id);
        assert_eq!(duplicate.title, "喝水提醒（副本）");
    }

    #[test]
    fn rejects_negative_grace_minutes() {
        let mut repository = InMemoryReminderTemplateRepository::default();
        let mut input = create_text_input();
        input.default_grace_minutes = -1;

        let error = repository.create(input).expect_err("negative grace should fail");

        assert_eq!(error, ReminderTemplateError::NegativeGraceMinutes);
    }

    #[test]
    fn rejects_invalid_repeat_rule() {
        let mut repository = InMemoryReminderTemplateRepository::default();
        let mut input = create_text_input();
        input.repeat_rule_json = r#"{"interval":1}"#.to_string();

        let error = repository
            .create(input)
            .expect_err("invalid repeat rule should fail");

        assert_eq!(error, ReminderTemplateError::InvalidRepeatRule);
    }

    #[test]
    fn validates_system_action_payload() {
        let mut repository = InMemoryReminderTemplateRepository::default();
        let mut input = create_text_input();
        input.event_type = ReminderEventType::SystemAction;
        input.event_payload_json = r#"{"action":"shutdown","message":"准备关机"}"#.to_string();

        let created = repository.create(input).expect("system action payload should be valid");

        assert_eq!(created.event_type, ReminderEventType::SystemAction);
    }
}
