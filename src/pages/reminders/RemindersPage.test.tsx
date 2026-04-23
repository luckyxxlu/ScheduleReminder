import { fireEvent, render, screen, waitFor } from '@testing-library/react'

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual<typeof import('react-router-dom')>('react-router-dom')

  return {
    ...actual,
    useNavigate: () => vi.fn(),
  }
})

import { createReminderTemplate, duplicateReminderTemplate, listReminderTemplates, toggleReminderTemplate, updateReminderTemplate } from '../../services/reminderTemplates'

import { RemindersPage } from './RemindersPage'

vi.mock('../../services/reminderTemplates', () => ({
  listReminderTemplates: vi.fn(),
  toggleReminderTemplate: vi.fn(),
  duplicateReminderTemplate: vi.fn(),
  createReminderTemplate: vi.fn(),
  updateReminderTemplate: vi.fn(),
}))

const mockedListReminderTemplates = vi.mocked(listReminderTemplates)
const mockedToggleReminderTemplate = vi.mocked(toggleReminderTemplate)
const mockedDuplicateReminderTemplate = vi.mocked(duplicateReminderTemplate)
const mockedCreateReminderTemplate = vi.mocked(createReminderTemplate)
const mockedUpdateReminderTemplate = vi.mocked(updateReminderTemplate)

function createDeferredPromise<T>() {
  let resolve!: (value: T) => void
  let reject!: (reason?: unknown) => void

  const promise = new Promise<T>((innerResolve, innerReject) => {
    resolve = innerResolve
    reject = innerReject
  })

  return { promise, resolve, reject }
}

const templates = [
  {
    id: 'tpl_1',
    title: '喝水提醒',
    message: '喝水时间到了',
    category: 'health',
    repeatRuleJson: '{"type":"daily","interval":1,"time":"08:00"}',
    defaultGraceMinutes: 10,
    note: null,
    scheduleSummary: '每天 08:00',
    eventTypeLabel: '文本提醒',
    enabled: true,
  },
]

describe('RemindersPage', () => {
  beforeEach(() => {
    mockedListReminderTemplates.mockReset()
    mockedToggleReminderTemplate.mockReset()
    mockedDuplicateReminderTemplate.mockReset()
    mockedCreateReminderTemplate.mockReset()
    mockedUpdateReminderTemplate.mockReset()

    mockedListReminderTemplates.mockResolvedValue(templates)
    mockedToggleReminderTemplate.mockResolvedValue({
      ...templates[0],
      enabled: false,
    })
    mockedDuplicateReminderTemplate.mockResolvedValue({
      ...templates[0],
      id: 'tpl_2',
      title: '喝水提醒（副本）',
    })
    mockedCreateReminderTemplate.mockResolvedValue({
      id: 'tpl_3',
      title: '深度工作',
      message: '开始 45 分钟专注工作',
      category: 'focus',
      repeatRuleJson: '{"type":"daily","interval":1,"time":"14:30"}',
      defaultGraceMinutes: 10,
      note: '',
      scheduleSummary: '每天 14:30',
      eventTypeLabel: '文本提醒',
      enabled: true,
    })
    mockedUpdateReminderTemplate.mockResolvedValue({
      ...templates[0],
      title: '补水提醒',
      message: '现在去接一杯温水',
      repeatRuleJson: '{"type":"daily","interval":1,"time":"09:30"}',
      defaultGraceMinutes: 20,
      note: '上午第二次补水',
      scheduleSummary: '每天 09:30',
    })
  })

  it('renders reminder template list', async () => {
    render(<RemindersPage />)

    expect(screen.getByText('提醒模板列表')).toBeInTheDocument()
    expect(await screen.findByText('喝水提醒')).toBeInTheDocument()
  })

  it('loads reminder templates from backend service', async () => {
    render(<RemindersPage />)

    expect(await screen.findByText('喝水提醒')).toBeInTheDocument()
    expect(screen.getByText('每天 08:00 | 文本提醒')).toBeInTheDocument()
  })

  it('toggles template enabled state', async () => {
    render(<RemindersPage />)

    const toggle = await screen.findByRole('button', { name: '停用' })
    fireEvent.click(toggle)

    await waitFor(() => {
      expect(mockedToggleReminderTemplate).toHaveBeenCalledWith('tpl_1', false)
      expect(screen.getByRole('button', { name: '启用' })).toBeInTheDocument()
      expect(screen.getByText('已暂停')).toBeInTheDocument()
    })
  })

  it('keeps other templates when toggling one template and supports enabling again', async () => {
    mockedListReminderTemplates.mockResolvedValueOnce([
      {
        ...templates[0],
        enabled: false,
      },
      {
        ...templates[0],
        id: 'tpl_2',
        title: '午休提醒',
        scheduleSummary: '每天 13:00',
      },
    ])
    mockedToggleReminderTemplate.mockResolvedValueOnce({
      ...templates[0],
      enabled: true,
    })

    render(<RemindersPage />)

    fireEvent.click(await screen.findByRole('button', { name: '启用' }))

    await waitFor(() => {
      expect(screen.getByText('午休提醒')).toBeInTheDocument()
      expect(screen.getByText('喝水提醒 已启用')).toBeInTheDocument()
      expect(screen.getAllByText('运行中').length).toBeGreaterThan(0)
    })
  })

  it('duplicates template through backend service', async () => {
    render(<RemindersPage />)

    fireEvent.click(await screen.findByRole('button', { name: '复制 喝水提醒' }))

    await waitFor(() => {
      expect(mockedDuplicateReminderTemplate).toHaveBeenCalledWith('tpl_1')
      expect(screen.getByText('喝水提醒（副本）')).toBeInTheDocument()
    })
  })

  it('creates reminder template with custom content', async () => {
    render(<RemindersPage />)

    fireEvent.change(await screen.findByLabelText('提醒标题'), { target: { value: '深度工作' } })
    fireEvent.change(screen.getByLabelText('提醒内容'), { target: { value: '开始 45 分钟专注工作' } })
    fireEvent.change(screen.getByLabelText('提醒分类'), { target: { value: 'focus' } })
    fireEvent.change(screen.getByLabelText('模板时间'), { target: { value: '14:30' } })
    fireEvent.click(screen.getByRole('button', { name: '保存模板' }))

    await waitFor(() => {
      expect(mockedCreateReminderTemplate).toHaveBeenCalledWith({
        title: '深度工作',
        message: '开始 45 分钟专注工作',
        category: 'focus',
        repeatRuleJson: '{"type":"daily","interval":1,"time":"14:30"}',
        defaultGraceMinutes: 10,
        note: '',
      })
      expect(screen.getByText('深度工作')).toBeInTheDocument()
      expect(screen.getByText('已保存提醒模板 深度工作')).toBeInTheDocument()
    })
  })

  it('creates one-time reminder template with none repeat rule', async () => {
    render(<RemindersPage />)

    fireEvent.change(await screen.findByLabelText('提醒标题'), { target: { value: '单次复盘' } })
    fireEvent.change(screen.getByLabelText('提醒内容'), { target: { value: '今天下班前做一次复盘' } })
    fireEvent.change(screen.getByLabelText('重复方式'), { target: { value: 'none' } })
    fireEvent.change(screen.getByLabelText('模板时间'), { target: { value: '18:30' } })
    fireEvent.click(screen.getByRole('button', { name: '保存模板' }))

    await waitFor(() => {
      expect(mockedCreateReminderTemplate).toHaveBeenCalledWith({
        title: '单次复盘',
        message: '今天下班前做一次复盘',
        category: '',
        repeatRuleJson: '{"type":"none","time":"18:30"}',
        defaultGraceMinutes: 10,
        note: '',
      })
    })
  })

  it('creates workday reminder template with workdays repeat rule', async () => {
    render(<RemindersPage />)

    fireEvent.change(await screen.findByLabelText('提醒标题'), { target: { value: '工间站立' } })
    fireEvent.change(screen.getByLabelText('提醒内容'), { target: { value: '站起来活动两分钟' } })
    fireEvent.change(screen.getByLabelText('重复方式'), { target: { value: 'workdays' } })
    fireEvent.change(screen.getByLabelText('模板时间'), { target: { value: '15:00' } })
    fireEvent.click(screen.getByRole('button', { name: '保存模板' }))

    await waitFor(() => {
      expect(mockedCreateReminderTemplate).toHaveBeenCalledWith({
        title: '工间站立',
        message: '站起来活动两分钟',
        category: '',
        repeatRuleJson: '{"type":"workdays","time":"15:00"}',
        defaultGraceMinutes: 10,
        note: '',
      })
    })
  })

  it('shows visible error when template creation fails', async () => {
    mockedCreateReminderTemplate.mockRejectedValueOnce(new Error('数据库连接失败'))

    render(<RemindersPage />)

    fireEvent.change(await screen.findByLabelText('提醒标题'), { target: { value: '深度工作' } })
    fireEvent.change(screen.getByLabelText('提醒内容'), { target: { value: '开始 45 分钟专注工作' } })
    fireEvent.click(screen.getByRole('button', { name: '保存模板' }))

    expect(await screen.findByText('数据库连接失败')).toBeInTheDocument()
  })

  it('shows fallback error when template creation fails with non-error rejection', async () => {
    mockedCreateReminderTemplate.mockRejectedValueOnce('bad')

    render(<RemindersPage />)

    fireEvent.change(await screen.findByLabelText('提醒标题'), { target: { value: '深度工作' } })
    fireEvent.change(screen.getByLabelText('提醒内容'), { target: { value: '开始 45 分钟专注工作' } })
    fireEvent.click(screen.getByRole('button', { name: '保存模板' }))

    expect(await screen.findByText('保存提醒模板失败')).toBeInTheDocument()
  })

  it('supports editing existing template from user flow', async () => {
    render(<RemindersPage />)

    fireEvent.click(await screen.findByRole('button', { name: '编辑 喝水提醒' }))
    fireEvent.change(screen.getByLabelText('提醒标题'), { target: { value: '补水提醒' } })
    fireEvent.change(screen.getByLabelText('提醒内容'), { target: { value: '现在去接一杯温水' } })
    fireEvent.change(screen.getByLabelText('模板时间'), { target: { value: '09:30' } })
    fireEvent.change(screen.getByLabelText('模板宽容时间'), { target: { value: '20' } })
    fireEvent.change(screen.getByLabelText('提醒备注'), { target: { value: '上午第二次补水' } })
    fireEvent.click(screen.getByRole('button', { name: '保存修改' }))

    await waitFor(() => {
      expect(mockedUpdateReminderTemplate).toHaveBeenCalledWith({
        id: 'tpl_1',
        title: '补水提醒',
        message: '现在去接一杯温水',
        category: 'health',
        repeatRuleJson: '{"type":"daily","interval":1,"time":"09:30"}',
        defaultGraceMinutes: 20,
        note: '上午第二次补水',
        enabled: true,
      })
      expect(screen.getByText('已更新提醒模板 补水提醒')).toBeInTheDocument()
    })
  })

  it('keeps non-updated templates when editing one template', async () => {
    mockedListReminderTemplates.mockResolvedValueOnce([
      ...templates,
      {
        ...templates[0],
        id: 'tpl_2',
        title: '午休提醒',
        scheduleSummary: '每天 13:00',
      },
    ])

    render(<RemindersPage />)

    fireEvent.click(await screen.findByRole('button', { name: '编辑 喝水提醒' }))
    fireEvent.change(screen.getByLabelText('提醒标题'), { target: { value: '补水提醒' } })
    fireEvent.change(screen.getByLabelText('提醒内容'), { target: { value: '现在去接一杯温水' } })
    fireEvent.click(screen.getByRole('button', { name: '保存修改' }))

    await waitFor(() => {
      expect(screen.getByText('午休提醒')).toBeInTheDocument()
      expect(screen.getByText('补水提醒')).toBeInTheDocument()
    })
  })

  it('shows validation message before saving edited template with missing fields', async () => {
    render(<RemindersPage />)

    fireEvent.click(await screen.findByRole('button', { name: '编辑 喝水提醒' }))
    fireEvent.change(screen.getByLabelText('提醒标题'), { target: { value: '   ' } })
    fireEvent.change(screen.getByLabelText('提醒内容'), { target: { value: '' } })
    fireEvent.click(screen.getByRole('button', { name: '保存修改' }))

    expect(await screen.findByText('请先填写提醒标题和提醒内容')).toBeInTheDocument()
    expect(mockedUpdateReminderTemplate).not.toHaveBeenCalled()
  })

  it('supports editing none repeat template and saving fallback fields', async () => {
    mockedListReminderTemplates.mockResolvedValueOnce([
      {
        ...templates[0],
        id: 'tpl_9',
        title: '单次提醒',
        repeatRuleJson: '{"type":"none"}',
        category: null,
        note: null,
      },
    ])
    mockedUpdateReminderTemplate.mockResolvedValueOnce({
      ...templates[0],
      id: 'tpl_9',
      title: '单次提醒',
      repeatRuleJson: '{"type":"none","time":"09:00"}',
    })

    render(<RemindersPage />)

    fireEvent.click(await screen.findByRole('button', { name: '编辑 单次提醒' }))
    fireEvent.click(screen.getByRole('button', { name: '保存修改' }))

    await waitFor(() => {
      expect(mockedUpdateReminderTemplate).toHaveBeenCalledWith({
        id: 'tpl_9',
        title: '单次提醒',
        message: '喝水时间到了',
        category: '',
        repeatRuleJson: '{"type":"none","time":"09:00"}',
        defaultGraceMinutes: 10,
        note: '',
        enabled: true,
      })
    })
  })

  it('supports editing workday template and keeps workday repeat rule', async () => {
    mockedListReminderTemplates.mockResolvedValueOnce([
      {
        ...templates[0],
        id: 'tpl_7',
        title: '工作日站立',
        repeatRuleJson: '{"type":"workdays","time":"10:30"}',
        scheduleSummary: '工作日 10:30',
      },
    ])
    mockedUpdateReminderTemplate.mockResolvedValueOnce({
      ...templates[0],
      id: 'tpl_7',
      title: '工作日站立',
      repeatRuleJson: '{"type":"workdays","time":"11:00"}',
      scheduleSummary: '工作日 11:00',
    })

    render(<RemindersPage />)

    fireEvent.click(await screen.findByRole('button', { name: '编辑 工作日站立' }))
    fireEvent.change(screen.getByLabelText('模板时间'), { target: { value: '11:00' } })
    fireEvent.click(screen.getByRole('button', { name: '保存修改' }))

    await waitFor(() => {
      expect(mockedUpdateReminderTemplate).toHaveBeenCalledWith({
        id: 'tpl_7',
        title: '工作日站立',
        message: '喝水时间到了',
        category: 'health',
        repeatRuleJson: '{"type":"workdays","time":"11:00"}',
        defaultGraceMinutes: 10,
        note: '',
        enabled: true,
      })
    })
  })

  it('shows validation message before creating template with missing fields', async () => {
    render(<RemindersPage />)

    fireEvent.change(await screen.findByLabelText('提醒标题'), { target: { value: '   ' } })
    fireEvent.change(screen.getByLabelText('提醒内容'), { target: { value: '' } })
    fireEvent.click(screen.getByRole('button', { name: '保存模板' }))

    expect(await screen.findByText('请先填写提醒标题和提醒内容')).toBeInTheDocument()
    expect(mockedCreateReminderTemplate).not.toHaveBeenCalled()
  })

  it('shows visible load error when templates fail to load', async () => {
    mockedListReminderTemplates.mockRejectedValueOnce(new Error('提醒模板加载失败'))

    render(<RemindersPage />)

    expect(await screen.findByText('提醒模板加载失败')).toBeInTheDocument()
  })

  it('shows fallback load error for non-error rejection', async () => {
    mockedListReminderTemplates.mockRejectedValueOnce('bad')

    render(<RemindersPage />)

    expect(await screen.findByText('提醒模板加载失败')).toBeInTheDocument()
  })

  it('ignores resolved list request after unmount', async () => {
    const deferred = createDeferredPromise<Awaited<ReturnType<typeof listReminderTemplates>>>()
    mockedListReminderTemplates.mockReturnValueOnce(deferred.promise)

    const { unmount } = render(<RemindersPage />)
    unmount()

    deferred.resolve(templates)

    await waitFor(() => {
      expect(screen.queryByText('喝水提醒')).not.toBeInTheDocument()
    })
  })

  it('ignores rejected list request after unmount', async () => {
    const deferred = createDeferredPromise<Awaited<ReturnType<typeof listReminderTemplates>>>()
    mockedListReminderTemplates.mockReturnValueOnce(deferred.promise)

    const { unmount } = render(<RemindersPage />)
    unmount()

    deferred.reject(new Error('不应显示'))

    await waitFor(() => {
      expect(screen.queryByText('不应显示')).not.toBeInTheDocument()
    })
  })

  it('shows visible error when toggling template fails', async () => {
    mockedToggleReminderTemplate.mockRejectedValueOnce(new Error('提醒模板状态更新失败'))

    render(<RemindersPage />)

    fireEvent.click(await screen.findByRole('button', { name: '停用' }))

    expect(await screen.findByText('提醒模板状态更新失败')).toBeInTheDocument()
  })

  it('shows fallback error when toggling template fails with non-error rejection', async () => {
    mockedToggleReminderTemplate.mockRejectedValueOnce('bad')

    render(<RemindersPage />)

    fireEvent.click(await screen.findByRole('button', { name: '停用' }))

    expect(await screen.findByText('提醒模板状态更新失败')).toBeInTheDocument()
  })

  it('shows visible error when duplication fails with error object', async () => {
    mockedDuplicateReminderTemplate.mockRejectedValueOnce(new Error('复制服务不可用'))

    render(<RemindersPage />)

    fireEvent.click(await screen.findByRole('button', { name: '复制 喝水提醒' }))

    expect(await screen.findByText('复制服务不可用')).toBeInTheDocument()
  })

  it('shows fallback error when duplication fails with non-error rejection', async () => {
    mockedDuplicateReminderTemplate.mockRejectedValueOnce('bad')

    render(<RemindersPage />)

    fireEvent.click(await screen.findByRole('button', { name: '复制 喝水提醒' }))

    expect(await screen.findByText('复制提醒模板失败')).toBeInTheDocument()
  })

  it('resets form when leaving edit mode from header action', async () => {
    render(<RemindersPage />)

    fireEvent.click(await screen.findByRole('button', { name: '编辑 喝水提醒' }))
    expect(screen.getByDisplayValue('喝水提醒')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: '开始新建' }))

    expect(screen.queryByDisplayValue('喝水提醒')).not.toBeInTheDocument()
    expect(screen.getByRole('button', { name: '新建提醒模板' })).toBeInTheDocument()
  })

  it('navigates to calendar from secondary action', async () => {
    render(<RemindersPage />)

    await screen.findByText('喝水提醒')
    fireEvent.click(screen.getByRole('button', { name: '去日历查看' }))
  })

  it('shows fallback error when update fails with non-error rejection', async () => {
    mockedUpdateReminderTemplate.mockRejectedValueOnce('bad')

    render(<RemindersPage />)

    fireEvent.click(await screen.findByRole('button', { name: '编辑 喝水提醒' }))
    fireEvent.click(screen.getByRole('button', { name: '保存修改' }))

    expect(await screen.findByText('更新提醒模板失败')).toBeInTheDocument()
  })

  it('shows visible update error when update fails with error object', async () => {
    mockedUpdateReminderTemplate.mockRejectedValueOnce(new Error('更新服务不可用'))

    render(<RemindersPage />)

    fireEvent.click(await screen.findByRole('button', { name: '编辑 喝水提醒' }))
    fireEvent.click(screen.getByRole('button', { name: '保存修改' }))

    expect(await screen.findByText('更新服务不可用')).toBeInTheDocument()
  })

  it('does nothing when clicking header action outside editing mode', async () => {
    render(<RemindersPage />)

    await screen.findByText('喝水提醒')
    fireEvent.click(screen.getByRole('button', { name: '新建提醒模板' }))

    expect(screen.getByRole('button', { name: '保存模板' })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: '取消编辑' })).not.toBeInTheDocument()
  })
})
