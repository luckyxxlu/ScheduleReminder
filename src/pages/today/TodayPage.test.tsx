import { fireEvent, render, screen, waitFor } from '@testing-library/react'

import { getTodayDashboard, markNextReminderCompleted } from '../../services/dashboard'

import { TodayPage } from './TodayPage'

vi.mock('../../services/dashboard', () => ({
  getTodayDashboard: vi.fn(),
  markNextReminderCompleted: vi.fn(),
}))

const mockedGetTodayDashboard = vi.mocked(getTodayDashboard)
const mockedMarkNextReminderCompleted = vi.mocked(markNextReminderCompleted)

describe('TodayPage', () => {
  beforeEach(() => {
    mockedGetTodayDashboard.mockResolvedValue({
      nextReminderTitle: '准备休息',
      nextReminderTime: '22:30',
      nextReminderMessage: '宽容 15 分钟，支持稍后提醒与跳过今天。',
      highlightedStatus: '宽容中',
    })
    mockedMarkNextReminderCompleted.mockResolvedValue({
      nextReminderTitle: '准备休息',
      nextReminderTime: '22:30',
      nextReminderMessage: '宽容 15 分钟，支持稍后提醒与跳过今天。',
      highlightedStatus: '已完成',
    })
  })

  it('renders next reminder and grace section', async () => {
    render(<TodayPage />)

    expect(screen.getByText('下一条提醒')).toBeInTheDocument()
    expect(screen.getByText('宽容中的提醒')).toBeInTheDocument()
    expect(await screen.findByText('22:30 准备休息')).toBeInTheDocument()
  })

  it('shows action buttons for quick handling', async () => {
    render(<TodayPage />)

    await screen.findByText('22:30 准备休息')
    expect(screen.getByRole('button', { name: '完成' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '宽容 10 分钟' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '稍后提醒' })).toBeInTheDocument()
  })

  it('updates status when marking today reminder as completed', async () => {
    render(<TodayPage />)

    fireEvent.click(screen.getByRole('button', { name: '完成' }))

    await waitFor(() => {
      expect(mockedMarkNextReminderCompleted).toHaveBeenCalled()
      expect(screen.getByText('已完成')).toBeInTheDocument()
    })
  })
})
