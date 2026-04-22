import { invoke } from '@tauri-apps/api/core'

import {
  duplicateReminderTemplate,
  listReminderTemplates,
  toggleReminderTemplate,
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
})
