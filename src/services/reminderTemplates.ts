import { invoke } from '@tauri-apps/api/core'

export type ReminderTemplateListItem = {
  id: string
  title: string
  message: string
  category: string | null
  repeatRuleJson: string
  defaultGraceMinutes: number
  note: string | null
  scheduleSummary: string
  eventTypeLabel: string
  enabled: boolean
}

export type CreateReminderTemplateInput = {
  title: string
  message: string
  category?: string
  repeatRuleJson: string
  defaultGraceMinutes: number
  note?: string
}

export async function listReminderTemplates(): Promise<ReminderTemplateListItem[]> {
  return invoke<ReminderTemplateListItem[]>('list_reminder_templates')
}

export async function toggleReminderTemplate(
  id: string,
  enabled: boolean,
): Promise<ReminderTemplateListItem> {
  return invoke<ReminderTemplateListItem>('toggle_reminder_template', {
    id,
    enabled,
  })
}

export async function duplicateReminderTemplate(id: string): Promise<ReminderTemplateListItem> {
  return invoke<ReminderTemplateListItem>('duplicate_reminder_template', { id })
}

export async function createReminderTemplate(input: CreateReminderTemplateInput): Promise<ReminderTemplateListItem> {
  return invoke<ReminderTemplateListItem>('create_reminder_template', {
    title: input.title,
    message: input.message,
    category: input.category ?? null,
    repeat_rule_json: input.repeatRuleJson,
    default_grace_minutes: input.defaultGraceMinutes,
    note: input.note ?? null,
  })
}

export async function updateReminderTemplate(input: CreateReminderTemplateInput & { id: string; enabled: boolean }): Promise<ReminderTemplateListItem> {
  return invoke<ReminderTemplateListItem>('update_reminder_template', {
    id: input.id,
    title: input.title,
    message: input.message,
    category: input.category ?? null,
    repeat_rule_json: input.repeatRuleJson,
    default_grace_minutes: input.defaultGraceMinutes,
    note: input.note ?? null,
    enabled: input.enabled,
  })
}
