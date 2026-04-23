import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { listen } from '@tauri-apps/api/event'

import { markNextReminderCompleted } from '../services/dashboard'

import { App } from './App'

vi.mock('@tauri-apps/api/event', () => ({
  emit: vi.fn(),
  listen: vi.fn().mockResolvedValue(() => {}),
  TauriEvent: {
    WINDOW_FOCUS: 'tauri://focus',
  },
}))

vi.mock('../services/dashboard', () => ({
  getTodayDashboard: vi.fn().mockResolvedValue({
    activeReminderId: 'occ_2',
    nextReminderTitle: '准备休息',
    nextReminderTime: '22:30',
    nextReminderMessage: '准备休息，放下屏幕',
    nextReminderStatus: '宽容中',
    nextReminderNotificationState: '这条提醒已进入宽容时间，对应 Windows 通知已触发。',
    nextReminderGraceDeadline: '22:45',
    nextReminderAvailableActions: ['complete', 'grace_10_minutes', 'snooze', 'skip'],
    highlightedStatus: '宽容中',
    todayTimeline: [
      {
        id: 'occ_2',
        time: '22:30',
        title: '准备休息',
        message: '准备休息，放下屏幕',
        status: '宽容中',
        isActive: true,
      },
    ],
    recentActions: [],
  }),
  markNextReminderCompleted: vi.fn().mockResolvedValue({
    activeReminderId: 'occ_2',
    nextReminderTitle: '准备休息',
    nextReminderTime: '22:30',
    nextReminderMessage: '准备休息，放下屏幕',
    nextReminderStatus: '已完成',
    nextReminderNotificationState: '这条提醒已进入宽容时间，对应 Windows 通知已触发。',
    nextReminderGraceDeadline: '22:45',
    nextReminderAvailableActions: [],
    highlightedStatus: '已完成',
    todayTimeline: [],
    recentActions: [],
  }),
  graceNextReminderTenMinutes: vi.fn(),
  snoozeNextReminder: vi.fn(),
  skipNextReminder: vi.fn(),
  getCalendarOverview: vi.fn().mockResolvedValue({
    monthKey: '2026-04',
    monthEntries: [{ date: '2026-04-22', reminderCount: 1 }],
    selectedDate: '2026-04-22',
    recentActions: [],
    entries: [{ id: 'occ_1', date: '2026-04-22 08:00:00', time: '08:00', title: '喝水提醒', message: '喝水时间到了', status: '宽容中' }],
  }),
  createCalendarEvent: vi.fn(),
}))

vi.mock('../services/settings', () => ({
  getSettings: vi.fn().mockResolvedValue({
    defaultGraceMinutes: 10,
    startupWithWindows: false,
    closeToTrayOnClose: true,
    quietHoursEnabled: true,
    quietHoursStart: '22:00',
    quietHoursEnd: '07:00',
  }),
  updateSettings: vi.fn(),
}))

vi.mock('../services/reminderTemplates', () => ({
  listReminderTemplates: vi.fn().mockResolvedValue([]),
  toggleReminderTemplate: vi.fn(),
  duplicateReminderTemplate: vi.fn(),
  createReminderTemplate: vi.fn(),
}))

const mockedListen = vi.mocked(listen)
const mockedMarkNextReminderCompleted = vi.mocked(markNextReminderCompleted)

describe('App', () => {
  beforeEach(() => {
    window.location.hash = '#/'
    mockedListen.mockClear()
  })

  it('renders the main navigation', async () => {
    render(<App />)

    expect(screen.getByRole('navigation', { name: '主导航' })).toBeInTheDocument()
    expect(screen.getByText('时间助手')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: '今天' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: '日历' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: '提醒' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: '设置' })).toBeInTheDocument()
    expect(await screen.findByText('22:30 准备休息')).toBeInTheDocument()
  })

  it('renders the today page by default', async () => {
    render(<App />)

    expect(screen.getByRole('heading', { name: '今天' })).toBeInTheDocument()
    expect(screen.getByText('下一条提醒')).toBeInTheDocument()
    expect(await screen.findByText('22:30 准备休息')).toBeInTheDocument()
  })

  it('renders the calendar page when hash route changes', async () => {
    window.location.hash = '#/calendar'

    render(<App />)

    expect(screen.getByRole('heading', { name: '日历' })).toBeInTheDocument()
    expect(screen.getByText('本月提醒分布')).toBeInTheDocument()
    expect(await screen.findByText('2026 年 4 月')).toBeInTheDocument()
  })

  it('closes the reminder overlay after completing the reminder', async () => {
    let reminderTriggeredHandler: ((event: { payload: { occurrenceId: string; title: string; message: string; scheduledTime: string; graceDeadline: string } }) => void | Promise<void>) | undefined

    mockedListen.mockImplementation(async (event, handler) => {
      if (event === 'reminder-triggered') {
        reminderTriggeredHandler = handler as typeof reminderTriggeredHandler
      }

      return () => {}
    })

    render(<App />)

    await waitFor(() => {
      expect(reminderTriggeredHandler).toBeDefined()
    })

    await act(async () => {
      await reminderTriggeredHandler?.({
        payload: {
          occurrenceId: 'occ_2',
          title: '准备休息',
          message: '准备休息，放下屏幕',
          scheduledTime: '22:30',
          graceDeadline: '22:45',
        },
      })
    })

    expect(await screen.findByLabelText('触发提醒')).toBeInTheDocument()
    fireEvent.click(screen.getByLabelText('触发提醒').querySelector('button') as HTMLButtonElement)

    await waitFor(() => {
      expect(mockedMarkNextReminderCompleted).toHaveBeenCalled()
      expect(screen.queryByLabelText('触发提醒')).not.toBeInTheDocument()
    })
  })

  it('closes the reminder overlay when pressing escape', async () => {
    let reminderTriggeredHandler: ((event: { payload: { occurrenceId: string; title: string; message: string; scheduledTime: string; graceDeadline: string } }) => void | Promise<void>) | undefined

    mockedListen.mockImplementation(async (event, handler) => {
      if (event === 'reminder-triggered') {
        reminderTriggeredHandler = handler as typeof reminderTriggeredHandler
      }

      return () => {}
    })

    render(<App />)

    await waitFor(() => {
      expect(reminderTriggeredHandler).toBeDefined()
    })

    await act(async () => {
      await reminderTriggeredHandler?.({
        payload: {
          occurrenceId: 'occ_2',
          title: '准备休息',
          message: '准备休息，放下屏幕',
          scheduledTime: '22:30',
          graceDeadline: '22:45',
        },
      })
    })

    expect(await screen.findByText('宽容截止至：22:45')).toBeInTheDocument()
    fireEvent.keyDown(window, { key: 'Escape' })

    await waitFor(() => {
      expect(screen.queryByText('宽容截止至：22:45')).not.toBeInTheDocument()
    })
  })
})
