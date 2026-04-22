import { render, screen } from '@testing-library/react'

import { App } from './App'

vi.mock('../services/dashboard', () => ({
  getTodayDashboard: vi.fn().mockResolvedValue({
    nextReminderTitle: '准备休息',
    nextReminderTime: '22:30',
    nextReminderMessage: '宽容 15 分钟，支持稍后提醒与跳过今天。',
    highlightedStatus: '宽容中',
  }),
  markNextReminderCompleted: vi.fn().mockResolvedValue({
    nextReminderTitle: '准备休息',
    nextReminderTime: '22:30',
    nextReminderMessage: '宽容 15 分钟，支持稍后提醒与跳过今天。',
    highlightedStatus: '已完成',
  }),
  getCalendarOverview: vi.fn().mockResolvedValue({
    monthKey: '2026-04',
    monthEntries: [{ date: '2026-04-22', reminderCount: 1 }],
    selectedDate: '2026-04-22',
    entries: [{ id: 'occ_1', date: '2026-04-22 08:00:00', time: '08:00', title: '喝水提醒', status: '宽容中' }],
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
