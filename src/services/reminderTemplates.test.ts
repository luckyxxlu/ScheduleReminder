import { invoke } from '@tauri-apps/api/core'

import {
  createReminderTemplate,
  duplicateReminderTemplate,
  listReminderTemplates,
  toggleReminderTemplate,
  updateReminderTemplate,
} from './reminderTemplates'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const mockedInvoke = vi.mocked(invoke)

describe('reminderTemplates service', () => {
  beforeEach(() => {
    mockedInvoke.mockReset()
  })

  it('lists templates from tauri command', async () => {
    mockedInvoke.mockResolvedValue([{ id: 'tpl_1', title: '喝水提醒' }])

    const result = await listReminderTemplates()

    expect(mockedInvoke).toHaveBeenCalledWith('list_reminder_templates')
    expect(result).toEqual([{ id: 'tpl_1', title: '喝水提醒' }])
  })

  it('toggles template through tauri command', async () => {
    mockedInvoke.mockResolvedValue({ id: 'tpl_1', enabled: false })

    const result = await toggleReminderTemplate('tpl_1', false)

    expect(mockedInvoke).toHaveBeenCalledWith('toggle_reminder_template', {
      id: 'tpl_1',
      enabled: false,
    })
    expect(result).toEqual({ id: 'tpl_1', enabled: false })
  })

  it('duplicates template through tauri command', async () => {
    mockedInvoke.mockResolvedValue({ id: 'tpl_2', title: '喝水提醒（副本）' })

    const result = await duplicateReminderTemplate('tpl_1')

    expect(mockedInvoke).toHaveBeenCalledWith('duplicate_reminder_template', { id: 'tpl_1' })
    expect(result).toEqual({ id: 'tpl_2', title: '喝水提醒（副本）' })
  })

  it('creates template through tauri command', async () => {
    mockedInvoke.mockResolvedValue({ id: 'tpl_3', title: '深度工作' })

    const result = await createReminderTemplate({
      title: '深度工作',
      message: '开始 45 分钟专注工作',
      category: 'focus',
      repeatRuleJson: '{"type":"daily","interval":1,"time":"14:30"}',
      defaultGraceMinutes: 15,
      note: '下午专注块',
    })

    expect(mockedInvoke).toHaveBeenCalledWith('create_reminder_template', {
      title: '深度工作',
      message: '开始 45 分钟专注工作',
      category: 'focus',
      repeat_rule_json: '{"type":"daily","interval":1,"time":"14:30"}',
      default_grace_minutes: 15,
      note: '下午专注块',
    })
    expect(result).toEqual({ id: 'tpl_3', title: '深度工作' })
  })

  it('updates template through tauri command', async () => {
    mockedInvoke.mockResolvedValue({ id: 'tpl_1', title: '补水提醒' })

    const result = await updateReminderTemplate({
      id: 'tpl_1',
      title: '补水提醒',
      message: '现在去接一杯温水',
      category: 'health',
      repeatRuleJson: '{"type":"daily","interval":1,"time":"09:30"}',
      defaultGraceMinutes: 20,
      note: '上午第二次补水',
      enabled: true,
    })

    expect(mockedInvoke).toHaveBeenCalledWith('update_reminder_template', {
      id: 'tpl_1',
      title: '补水提醒',
      message: '现在去接一杯温水',
      category: 'health',
      repeat_rule_json: '{"type":"daily","interval":1,"time":"09:30"}',
      default_grace_minutes: 20,
      note: '上午第二次补水',
      enabled: true,
    })
    expect(result).toEqual({ id: 'tpl_1', title: '补水提醒' })
  })
})
