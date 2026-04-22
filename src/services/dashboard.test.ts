import { invoke } from '@tauri-apps/api/core'

import { createCalendarEvent, getCalendarOverview, getTodayDashboard, markNextReminderCompleted } from './dashboard'

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

  it('loads calendar overview for selected date', async () => {
    mockedInvoke.mockResolvedValue({ selectedDate: '2026-04-22', entries: [] })

    const result = await getCalendarOverview('2026-04-22')

    expect(mockedInvoke).toHaveBeenCalledWith('get_calendar_overview', {
      selected_date: '2026-04-22',
    })
    expect(result).toEqual({ selectedDate: '2026-04-22', entries: [] })
  })

  it('creates calendar event for selected date', async () => {
    mockedInvoke.mockResolvedValue({ selectedDate: '2026-04-22', entries: [] })

    const result = await createCalendarEvent({
      title: '深度工作',
      selectedDate: '2026-04-22',
      time: '14:30',
    })

    expect(mockedInvoke).toHaveBeenCalledWith('create_calendar_event', {
      title: '深度工作',
      selected_date: '2026-04-22',
      time: '14:30',
    })
    expect(result).toEqual({ selectedDate: '2026-04-22', entries: [] })
  })
})
