import { render, screen } from '@testing-library/react'

import { App } from './App'

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

describe('App', () => {
  beforeEach(() => {
    window.location.hash = '#/'
  })

  it('renders the main navigation', async () => {
    render(<App />)

    expect(screen.getByRole('navigation', { name: '主导航' })).toBeInTheDocument()
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
})
