import { invoke } from '@tauri-apps/api/core'

export type ReminderTemplateListItem = {
  id: string
  title: string
  scheduleSummary: string
  eventTypeLabel: string
  enabled: boolean
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
