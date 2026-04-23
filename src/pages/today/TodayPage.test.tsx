import { fireEvent, render, screen, waitFor } from '@testing-library/react'

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual<typeof import('react-router-dom')>('react-router-dom')

  return {
    ...actual,
    useNavigate: () => vi.fn(),
  }
})

import {
  getTodayDashboard,
  graceNextReminderTenMinutes,
  markNextReminderCompleted,
  skipNextReminder,
  snoozeNextReminder,
} from '../../services/dashboard'

import { TodayPage } from './TodayPage'

vi.mock('../../services/dashboard', () => ({
  getTodayDashboard: vi.fn(),
  graceNextReminderTenMinutes: vi.fn(),
  markNextReminderCompleted: vi.fn(),
  skipNextReminder: vi.fn(),
  snoozeNextReminder: vi.fn(),
}))

const mockedGetTodayDashboard = vi.mocked(getTodayDashboard)
const mockedGraceNextReminderTenMinutes = vi.mocked(graceNextReminderTenMinutes)
const mockedMarkNextReminderCompleted = vi.mocked(markNextReminderCompleted)
const mockedSkipNextReminder = vi.mocked(skipNextReminder)
const mockedSnoozeNextReminder = vi.mocked(snoozeNextReminder)

describe('TodayPage', () => {
  beforeEach(() => {
    mockedGetTodayDashboard.mockResolvedValue({
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
    })
    mockedMarkNextReminderCompleted.mockResolvedValue({
      activeReminderId: 'occ_2',
      nextReminderTitle: '准备休息',
      nextReminderTime: '22:30',
      nextReminderMessage: '准备休息，放下屏幕',
      nextReminderStatus: '已完成',
      nextReminderNotificationState: '这条提醒已进入宽容时间，对应 Windows 通知已触发。',
      nextReminderGraceDeadline: '22:45',
      nextReminderAvailableActions: [],
      highlightedStatus: '已完成',
      todayTimeline: [
        {
          id: 'occ_2',
          time: '22:30',
          title: '准备休息',
          message: '准备休息，放下屏幕',
          status: '已完成',
          isActive: true,
        },
      ],
      recentActions: [{ id: 'log_1', actionLabel: '已完成', actionAt: '2026-04-22 08:10:00' }],
    })
    mockedGraceNextReminderTenMinutes.mockResolvedValue({
      activeReminderId: 'occ_2',
      nextReminderTitle: '准备休息',
      nextReminderTime: '22:30',
      nextReminderMessage: '准备休息，放下屏幕',
      nextReminderStatus: '宽容中',
      nextReminderNotificationState: '这条提醒已进入宽容时间，对应 Windows 通知已触发。',
      nextReminderGraceDeadline: '22:55',
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
      recentActions: [{ id: 'log_2', actionLabel: '宽容 10 分钟', actionAt: '2026-04-22 08:10:00' }],
    })
    mockedSkipNextReminder.mockResolvedValue({
      activeReminderId: 'occ_2',
      nextReminderTitle: '准备休息',
      nextReminderTime: '22:30',
      nextReminderMessage: '准备休息，放下屏幕',
      nextReminderStatus: '已跳过',
      nextReminderNotificationState: '这条提醒已进入宽容时间，对应 Windows 通知已触发。',
      nextReminderGraceDeadline: '22:45',
      nextReminderAvailableActions: [],
      highlightedStatus: '已跳过',
      todayTimeline: [
        {
          id: 'occ_2',
          time: '22:30',
          title: '准备休息',
          message: '准备休息，放下屏幕',
          status: '已跳过',
          isActive: true,
        },
      ],
      recentActions: [{ id: 'log_3', actionLabel: '跳过今天', actionAt: '2026-04-22 08:10:00' }],
    })
    mockedSnoozeNextReminder.mockResolvedValue({
      activeReminderId: 'occ_2',
      nextReminderTitle: '准备休息',
      nextReminderTime: '22:30',
      nextReminderMessage: '准备休息，放下屏幕',
      nextReminderStatus: '宽容中',
      nextReminderNotificationState: '这条提醒已进入宽容时间，对应 Windows 通知已触发。',
      nextReminderGraceDeadline: '23:00',
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
      recentActions: [{ id: 'log_4', actionLabel: '稍后提醒', actionAt: '2026-04-22 08:10:00' }],
    })
  })

  it('renders next reminder and grace section', async () => {
    render(<TodayPage />)

    expect(screen.getByText('下一条提醒')).toBeInTheDocument()
    expect(screen.getByText('宽容中的提醒')).toBeInTheDocument()
    expect(screen.getByText('最近操作')).toBeInTheDocument()
    expect(await screen.findByText('22:30 准备休息')).toBeInTheDocument()
    expect(screen.getAllByText('准备休息，放下屏幕')).toHaveLength(2)
    expect(screen.getByText('这条提醒已进入宽容时间，对应 Windows 通知已触发。')).toBeInTheDocument()
    expect(screen.getByText('当前有提醒正在等待处理')).toBeInTheDocument()
  })

  it('shows action buttons for quick handling', async () => {
    render(<TodayPage />)

    await screen.findByText('22:30 准备休息')
    expect(screen.getByRole('button', { name: '完成' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '宽容 10 分钟' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '稍后提醒' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '跳过今天' })).toBeInTheDocument()
  })

  it('updates status when marking today reminder as completed', async () => {
    render(<TodayPage />)

    await screen.findByText('22:30 准备休息')
    fireEvent.click(screen.getByRole('button', { name: '完成' }))

    await waitFor(() => {
      expect(mockedMarkNextReminderCompleted).toHaveBeenCalled()
      expect(screen.getByText('2026-04-22 08:10:00')).toBeInTheDocument()
      expect(screen.getByText('已完成当前提醒')).toBeInTheDocument()
    })
  })

  it('snoozes reminder with selected minutes', async () => {
    render(<TodayPage />)

    await screen.findByText('22:30 准备休息')
    fireEvent.change(screen.getByLabelText('稍后提醒时长'), { target: { value: '30' } })
    fireEvent.click(screen.getByRole('button', { name: '稍后提醒' }))

    await waitFor(() => {
      expect(mockedSnoozeNextReminder).toHaveBeenCalledWith(30)
    })
  })

  it('skips reminder for today', async () => {
    render(<TodayPage />)

    await screen.findByText('22:30 准备休息')
    fireEvent.click(screen.getByRole('button', { name: '跳过今天' }))

    await waitFor(() => {
      expect(mockedSkipNextReminder).toHaveBeenCalled()
      expect(screen.getAllByText('已跳过').length).toBeGreaterThan(0)
    })
  })

  it('disables action buttons when reminder has not entered grace yet', async () => {
    mockedGetTodayDashboard.mockResolvedValueOnce({
      activeReminderId: 'occ_3',
      nextReminderTitle: '晨间计划',
      nextReminderTime: '08:00',
      nextReminderMessage: '开始今天的第一段专注安排',
      nextReminderStatus: '待处理',
      nextReminderNotificationState: '到达提醒时间后会发送 Windows 通知。',
      nextReminderGraceDeadline: '08:10',
      nextReminderAvailableActions: [],
      highlightedStatus: '待处理',
      todayTimeline: [
        {
          id: 'occ_3',
          time: '08:00',
          title: '晨间计划',
          message: '开始今天的第一段专注安排',
          status: '待处理',
          isActive: true,
        },
      ],
      recentActions: [],
    })

    render(<TodayPage />)

    expect(await screen.findByText('到达提醒时间后会发送 Windows 通知。')).toBeInTheDocument()
    expect(screen.getByText('下一条提醒尚未触发通知')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '完成' })).toBeDisabled()
    expect(screen.getByRole('button', { name: '跳过今天' })).toBeDisabled()
  })
})
