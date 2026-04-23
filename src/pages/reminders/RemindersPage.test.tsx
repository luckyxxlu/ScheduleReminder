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

    const toggle = await screen.findByRole('button', { name: '已启用' })
    fireEvent.click(toggle)

    await waitFor(() => {
      expect(mockedToggleReminderTemplate).toHaveBeenCalledWith('tpl_1', false)
      expect(screen.getByRole('button', { name: '已暂停' })).toBeInTheDocument()
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
    fireEvent.click(screen.getByRole('button', { name: '新建提醒模板' }))

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

  it('shows visible error when template creation fails', async () => {
    mockedCreateReminderTemplate.mockRejectedValueOnce(new Error('数据库连接失败'))

    render(<RemindersPage />)

    fireEvent.change(await screen.findByLabelText('提醒标题'), { target: { value: '深度工作' } })
    fireEvent.change(screen.getByLabelText('提醒内容'), { target: { value: '开始 45 分钟专注工作' } })
    fireEvent.click(screen.getByRole('button', { name: '新建提醒模板' }))

    expect(await screen.findByText('数据库连接失败')).toBeInTheDocument()
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

  it('shows validation message before creating template with missing fields', async () => {
    render(<RemindersPage />)

    fireEvent.change(await screen.findByLabelText('提醒标题'), { target: { value: '   ' } })
    fireEvent.change(screen.getByLabelText('提醒内容'), { target: { value: '' } })
    fireEvent.click(screen.getByRole('button', { name: '新建提醒模板' }))

    expect(await screen.findByText('请先填写提醒标题和提醒内容')).toBeInTheDocument()
    expect(mockedCreateReminderTemplate).not.toHaveBeenCalled()
  })
})
