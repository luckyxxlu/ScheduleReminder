import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { emit } from '@tauri-apps/api/event'

import { createCalendarEvent, deleteCalendarEvent, getCalendarOverview } from '../../services/dashboard'
import { CalendarPage } from './CalendarPage'

vi.mock('@tauri-apps/api/event', () => ({
  emit: vi.fn().mockResolvedValue(undefined),
}))

vi.mock('../../services/dashboard', () => ({
  getCalendarOverview: vi.fn(),
  createCalendarEvent: vi.fn(),
  deleteCalendarEvent: vi.fn(),
}))

const mockedEmit = vi.mocked(emit)
const mockedGetCalendarOverview = vi.mocked(getCalendarOverview)
const mockedCreateCalendarEvent = vi.mocked(createCalendarEvent)
const mockedDeleteCalendarEvent = vi.mocked(deleteCalendarEvent)

describe('CalendarPage', () => {
  beforeEach(() => {
    mockedEmit.mockReset()
    mockedGetCalendarOverview.mockReset()
    mockedCreateCalendarEvent.mockReset()
    mockedDeleteCalendarEvent.mockReset()

    mockedGetCalendarOverview.mockResolvedValue({
      selectedDate: '2026-04-22',
      monthKey: '2026-04',
      monthEntries: [{ date: '2026-04-22', reminderCount: 1 }],
      entries: [{ id: 'occ_1', date: '2026-04-22', time: '08:00', title: '喝水提醒', message: '喝水时间到了', status: '待处理' }],
      recentActions: [],
    })

    mockedCreateCalendarEvent.mockResolvedValue({
      selectedDate: '2026-04-22',
      monthKey: '2026-04',
      monthEntries: [{ date: '2026-04-22', reminderCount: 2 }],
      entries: [
        { id: 'occ_1', date: '2026-04-22', time: '08:00', title: '喝水提醒', message: '喝水时间到了', status: '待处理' },
        { id: 'occ_2', date: '2026-04-22', time: '19:30', title: '晚间整理', message: '收一下桌面', status: '待处理' },
      ],
      recentActions: [],
    })

    mockedDeleteCalendarEvent.mockResolvedValue({
      selectedDate: '2026-04-22',
      monthKey: '2026-04',
      monthEntries: [{ date: '2026-04-22', reminderCount: 0 }],
      entries: [],
      recentActions: [],
    })
  })

  it('emits dashboard refresh after creating a calendar event', async () => {
    render(<CalendarPage />)

    await screen.findByText('2026 年 4 月')
    fireEvent.change(screen.getByLabelText('事件标题'), { target: { value: '晚间整理' } })
    fireEvent.change(screen.getByLabelText('事件内容'), { target: { value: '收一下桌面' } })
    fireEvent.click(screen.getByRole('button', { name: /添加到/ }))

    await waitFor(() => {
      expect(mockedCreateCalendarEvent).toHaveBeenCalled()
      expect(mockedEmit).toHaveBeenCalledWith('today-dashboard-refresh-requested', { source: 'calendar' })
    })
  })

  it('creates a calendar event with seconds precision', async () => {
    render(<CalendarPage />)

    await screen.findByText('2026 年 4 月')
    expect(screen.getByText('触发时间（支持秒）')).toBeInTheDocument()
    expect(screen.getByLabelText('触发时间')).toHaveAttribute('step', '1')
    fireEvent.change(screen.getByLabelText('事件标题'), { target: { value: '秒级整理' } })
    fireEvent.change(screen.getByLabelText('事件内容'), { target: { value: '精确到秒地提醒收尾' } })
    fireEvent.change(screen.getByLabelText('触发时间'), { target: { value: '19:30:45' } })
    fireEvent.click(screen.getByRole('button', { name: /添加到/ }))

    await waitFor(() => {
      expect(mockedCreateCalendarEvent).toHaveBeenCalledWith({
        title: '秒级整理',
        message: '精确到秒地提醒收尾',
        selectedDate: expect.any(String),
        time: '19:30:45',
      })
    })
  })

  it('deletes a calendar event and emits dashboard refresh', async () => {
    render(<CalendarPage />)

    await screen.findByText('2026 年 4 月')
    fireEvent.click(screen.getByRole('button', { name: '删除 喝水提醒' }))

    await waitFor(() => {
      expect(mockedDeleteCalendarEvent).toHaveBeenCalledWith({
        occurrenceId: 'occ_1',
        selectedDate: expect.any(String),
      })
      expect(mockedEmit).toHaveBeenCalledWith('today-dashboard-refresh-requested', { source: 'calendar' })
    })
  })
})
