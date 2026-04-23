import { invoke } from '@tauri-apps/api/core'

import {
  createCalendarEvent,
  getCalendarOverview,
  getTodayDashboard,
  graceNextReminderTenMinutes,
  markNextReminderCompleted,
  skipNextReminder,
  snoozeNextReminder,
} from './dashboard'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const mockedInvoke = vi.mocked(invoke)

describe('dashboard service', () => {
  beforeEach(() => {
    mockedInvoke.mockReset()
  })

  it('loads today dashboard', async () => {
    mockedInvoke.mockResolvedValue({ nextReminderTitle: '准备休息' })

    const result = await getTodayDashboard()

    expect(mockedInvoke).toHaveBeenCalledWith('get_today_dashboard')
    expect(result).toEqual({ nextReminderTitle: '准备休息' })
  })

  it('marks grace reminder completed', async () => {
    mockedInvoke.mockResolvedValue({ highlightedStatus: '已完成' })

    const result = await markNextReminderCompleted()

    expect(mockedInvoke).toHaveBeenCalledWith('mark_next_reminder_completed')
    expect(result).toEqual({ highlightedStatus: '已完成' })
  })

  it('applies fixed ten minute grace', async () => {
    mockedInvoke.mockResolvedValue({ highlightedStatus: '宽容中' })

    const result = await graceNextReminderTenMinutes()

    expect(mockedInvoke).toHaveBeenCalledWith('grace_next_reminder_ten_minutes')
    expect(result).toEqual({ highlightedStatus: '宽容中' })
  })

  it('snoozes grace reminder with selected minutes', async () => {
    mockedInvoke.mockResolvedValue({ highlightedStatus: '宽容中' })

    const result = await snoozeNextReminder(15)

    expect(mockedInvoke).toHaveBeenCalledWith('snooze_next_reminder', { minutes: 15 })
    expect(result).toEqual({ highlightedStatus: '宽容中' })
  })

  it('skips grace reminder for today', async () => {
    mockedInvoke.mockResolvedValue({ highlightedStatus: '已跳过' })

    const result = await skipNextReminder()

    expect(mockedInvoke).toHaveBeenCalledWith('skip_next_reminder')
    expect(result).toEqual({ highlightedStatus: '已跳过' })
  })

  it('loads calendar overview for selected date', async () => {
    mockedInvoke.mockResolvedValue({ selectedDate: '2026-04-22', entries: [] })

    const result = await getCalendarOverview('2026-04-22')

    expect(mockedInvoke).toHaveBeenCalledWith('get_calendar_overview', {
      selectedDate: '2026-04-22',
    })
    expect(result).toEqual({ selectedDate: '2026-04-22', entries: [] })
  })

  it('creates calendar event for selected date', async () => {
    mockedInvoke.mockResolvedValue({ selectedDate: '2026-04-22', entries: [] })

    const result = await createCalendarEvent({
      title: '深度工作',
      message: '开始今天的专注时段',
      selectedDate: '2026-04-22',
      time: '14:30',
    })

    expect(mockedInvoke).toHaveBeenCalledWith('create_calendar_event', {
      title: '深度工作',
      message: '开始今天的专注时段',
      selectedDate: '2026-04-22',
      time: '14:30',
    })
    expect(result).toEqual({ selectedDate: '2026-04-22', entries: [] })
  })

  it('creates calendar event for selected date with seconds', async () => {
    mockedInvoke.mockResolvedValue({ selectedDate: '2026-04-22', entries: [] })

    const result = await createCalendarEvent({
      title: '秒级提醒',
      message: '开始精确提醒',
      selectedDate: '2026-04-22',
      time: '14:30:45',
    })

    expect(mockedInvoke).toHaveBeenCalledWith('create_calendar_event', {
      title: '秒级提醒',
      message: '开始精确提醒',
      selectedDate: '2026-04-22',
      time: '14:30:45',
    })
    expect(result).toEqual({ selectedDate: '2026-04-22', entries: [] })
  })
})
