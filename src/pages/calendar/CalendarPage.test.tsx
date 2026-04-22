import { fireEvent, render, screen, waitFor } from '@testing-library/react'

import { createCalendarEvent, getCalendarOverview } from '../../services/dashboard'

import { CalendarPage } from './CalendarPage'

vi.mock('../../services/dashboard', () => ({
  getCalendarOverview: vi.fn(),
  createCalendarEvent: vi.fn(),
}))

const mockedGetCalendarOverview = vi.mocked(getCalendarOverview)
const mockedCreateCalendarEvent = vi.mocked(createCalendarEvent)

describe('CalendarPage', () => {
  beforeEach(() => {
    mockedGetCalendarOverview.mockImplementation(async (selectedDate: string) => ({
      monthKey: '2026-04',
      monthEntries: [
        { date: '2026-04-22', reminderCount: 2 },
        { date: '2026-04-23', reminderCount: 1 },
      ],
      selectedDate,
      entries:
        selectedDate === '2026-04-22'
          ? [
              {
                id: 'occ_1',
                date: '2026-04-22 08:00:00',
                time: '08:00',
                title: '喝水提醒',
                status: '宽容中',
              },
            ]
          : [],
    }))
    mockedCreateCalendarEvent.mockResolvedValue({
      monthKey: '2026-04',
      monthEntries: [
        { date: '2026-04-22', reminderCount: 3 },
        { date: '2026-04-23', reminderCount: 1 },
      ],
      selectedDate: '2026-04-22',
      entries: [
        { id: 'occ_1', date: '2026-04-22 08:00:00', time: '08:00', title: '喝水提醒', status: '宽容中' },
        { id: 'occ_4', date: '2026-04-22 14:30:00', time: '14:30', title: '深度工作', status: '待处理' },
      ],
    })
  })

  it('renders monthly reminder distribution', async () => {
    render(<CalendarPage />)

    expect(screen.getByText('本月提醒分布')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '回到今天' })).toBeInTheDocument()
    expect(await screen.findByText('2026 年 4 月')).toBeInTheDocument()
  })

  it('shows selected date detail when clicking a day', async () => {
    render(<CalendarPage />)

    fireEvent.click(screen.getByRole('button', { name: '选择 2026-04-22' }))

    await waitFor(() => {
      expect(screen.getByText('2026-04-22')).toBeInTheDocument()
      expect(screen.getByText('喝水提醒')).toBeInTheDocument()
    })
  })

  it('creates calendar event for selected day', async () => {
    render(<CalendarPage />)

    fireEvent.change(screen.getByLabelText('事件标题'), { target: { value: '深度工作' } })
    fireEvent.change(screen.getByLabelText('提醒时间'), { target: { value: '14:30' } })
    fireEvent.click(screen.getByRole('button', { name: '添加到 2026-04-22' }))

    await waitFor(() => {
      expect(mockedCreateCalendarEvent).toHaveBeenCalledWith({
        title: '深度工作',
        selectedDate: '2026-04-22',
        time: '14:30',
      })
      expect(screen.getByText('深度工作')).toBeInTheDocument()
    })
  })
})
