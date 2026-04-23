import { fireEvent, render, screen, waitFor } from '@testing-library/react'

import { listen } from '@tauri-apps/api/event'

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

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  TauriEvent: {
    WINDOW_FOCUS: 'tauri://focus',
  },
}))

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
const mockedListen = vi.mocked(listen)

function createDeferredPromise<T>() {
  let resolve!: (value: T) => void
  let reject!: (reason?: unknown) => void

  const promise = new Promise<T>((innerResolve, innerReject) => {
    resolve = innerResolve
    reject = innerReject
  })

  return { promise, resolve, reject }
}

describe('TodayPage', () => {
  beforeEach(() => {
    mockedListen.mockReset()
    mockedListen.mockResolvedValue(() => {})
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
      nextReminderTime: '23:00',
      nextReminderMessage: '准备休息，放下屏幕',
      nextReminderStatus: '待处理',
      nextReminderNotificationState: '到达提醒时间后会发送 Windows 通知。',
      nextReminderGraceDeadline: '23:00',
      nextReminderAvailableActions: [],
      highlightedStatus: '待处理',
      todayTimeline: [
        {
          id: 'occ_2',
          time: '23:00',
          title: '准备休息',
          message: '准备休息，放下屏幕',
          status: '待处理',
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

  it('navigates to reminders from quick create action', async () => {
    render(<TodayPage />)

    await screen.findByText('22:30 准备休息')
    fireEvent.click(screen.getByRole('button', { name: '快速新建' }))
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
      expect(screen.getByText('已稍后提醒 30 分钟')).toBeInTheDocument()
      expect(screen.getByText('23:00 准备休息')).toBeInTheDocument()
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

  it('shows page load failure clearly', async () => {
    mockedGetTodayDashboard.mockRejectedValueOnce(new Error('今天页加载失败'))

    render(<TodayPage />)

    expect(await screen.findByText('今天页加载失败')).toBeInTheDocument()
  })

  it('shows fallback load failure for non-error rejection', async () => {
    mockedGetTodayDashboard.mockRejectedValueOnce('bad')

    render(<TodayPage />)

    expect(await screen.findByText('今天页加载失败')).toBeInTheDocument()
  })

  it('shows action failure when grace operation fails', async () => {
    mockedGraceNextReminderTenMinutes.mockRejectedValueOnce(new Error('提醒处理失败'))

    render(<TodayPage />)

    await screen.findByText('22:30 准备休息')
    fireEvent.click(screen.getByRole('button', { name: '宽容 10 分钟' }))

    expect(await screen.findByText('提醒处理失败')).toBeInTheDocument()
  })

  it('shows fallback action failure for non-error rejection', async () => {
    mockedSkipNextReminder.mockRejectedValueOnce('bad')

    render(<TodayPage />)

    await screen.findByText('22:30 准备休息')
    fireEvent.click(screen.getByRole('button', { name: '跳过今天' }))

    expect(await screen.findByText('提醒处理失败')).toBeInTheDocument()
  })

  it('hides grace deadline when backend does not provide one', async () => {
    mockedGetTodayDashboard.mockResolvedValueOnce({
      activeReminderId: 'occ_4',
      nextReminderTitle: '轻提醒',
      nextReminderTime: '10:00',
      nextReminderMessage: '只是简单提示一下',
      nextReminderStatus: '待处理',
      nextReminderNotificationState: '到达提醒时间后会发送 Windows 通知。',
      nextReminderGraceDeadline: null,
      nextReminderAvailableActions: [],
      highlightedStatus: '待处理',
      todayTimeline: [],
      recentActions: [],
    })

    render(<TodayPage />)

    await screen.findByText('10:00 轻提醒')
    expect(screen.queryByText(/宽容截止：/)).not.toBeInTheDocument()
    expect(screen.getByText('今天还没有提醒时间线。')).toBeInTheDocument()
  })

  it('renders inactive timeline items without active style', async () => {
    mockedGetTodayDashboard.mockResolvedValueOnce({
      activeReminderId: 'occ_6',
      nextReminderTitle: '收尾复盘',
      nextReminderTime: '18:00',
      nextReminderMessage: '整理今天完成情况',
      nextReminderStatus: '已完成',
      nextReminderNotificationState: '今日提醒已经处理完成。',
      nextReminderGraceDeadline: null,
      nextReminderAvailableActions: [],
      highlightedStatus: '已完成',
      todayTimeline: [
        {
          id: 'occ_6',
          time: '18:00',
          title: '收尾复盘',
          message: '整理今天完成情况',
          status: '已完成',
          isActive: false,
        },
      ],
      recentActions: [],
    })

    render(<TodayPage />)

    const timelineTitle = await screen.findByText('收尾复盘')
    expect(timelineTitle.closest('.timeline-item')).not.toHaveClass('timeline-item-active')
    expect(screen.getByText('今天已经开始推进了')).toBeInTheDocument()
  })

  it('ignores resolved dashboard request after unmount', async () => {
    const deferred = createDeferredPromise<Awaited<ReturnType<typeof getTodayDashboard>>>()
    mockedGetTodayDashboard.mockReturnValueOnce(deferred.promise)

    const { unmount } = render(<TodayPage />)
    unmount()

    deferred.resolve({
      activeReminderId: 'occ_5',
      nextReminderTitle: '晚间收尾',
      nextReminderTime: '21:30',
      nextReminderMessage: '整理今天的记录',
      nextReminderStatus: '待处理',
      nextReminderNotificationState: '到达提醒时间后会发送 Windows 通知。',
      nextReminderGraceDeadline: null,
      nextReminderAvailableActions: [],
      highlightedStatus: '待处理',
      todayTimeline: [],
      recentActions: [],
    })

    await waitFor(() => {
      expect(screen.queryByText('晚间收尾')).not.toBeInTheDocument()
    })
  })

  it('ignores rejected dashboard request after unmount', async () => {
    const deferred = createDeferredPromise<Awaited<ReturnType<typeof getTodayDashboard>>>()
    mockedGetTodayDashboard.mockReturnValueOnce(deferred.promise)

    const { unmount } = render(<TodayPage />)
    unmount()

    deferred.reject(new Error('不应显示'))

    await waitFor(() => {
      expect(screen.queryByText('不应显示')).not.toBeInTheDocument()
    })
  })

  it('refreshes immediately when receiving dashboard refresh event', async () => {
    const listeners = new Map<string, () => void>()

    mockedListen.mockImplementation(async (event, handler) => {
      listeners.set(String(event), () => handler({ event, id: 1, payload: undefined }))
      return () => {
        listeners.delete(String(event))
      }
    })

    mockedGetTodayDashboard
      .mockResolvedValueOnce({
        activeReminderId: 'occ_2',
        nextReminderTitle: '准备休息',
        nextReminderTime: '22:30',
        nextReminderMessage: '准备休息，放下屏幕',
        nextReminderStatus: '宽容中',
        nextReminderNotificationState: '这条提醒已进入宽容时间，对应 Windows 通知已触发。',
        nextReminderGraceDeadline: '22:45',
        nextReminderAvailableActions: ['complete', 'grace_10_minutes', 'snooze', 'skip'],
        highlightedStatus: '宽容中',
        todayTimeline: [],
        recentActions: [],
      })
      .mockResolvedValueOnce({
        activeReminderId: 'occ_7',
        nextReminderTitle: '新建联动提醒',
        nextReminderTime: '19:30',
        nextReminderMessage: '来自日历的新提醒',
        nextReminderStatus: '待处理',
        nextReminderNotificationState: '到达提醒时间后会发送 Windows 通知。',
        nextReminderGraceDeadline: null,
        nextReminderAvailableActions: [],
        highlightedStatus: '待处理',
        todayTimeline: [],
        recentActions: [],
      })

    render(<TodayPage />)

    await screen.findByText('22:30 准备休息')
    listeners.get('today-dashboard-refresh-requested')?.()

    expect(await screen.findByText('19:30 新建联动提醒')).toBeInTheDocument()
  })
})
