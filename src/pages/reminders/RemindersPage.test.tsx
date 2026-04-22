import { fireEvent, render, screen, waitFor } from '@testing-library/react'

import { duplicateReminderTemplate, listReminderTemplates, toggleReminderTemplate } from '../../services/reminderTemplates'

import { RemindersPage } from './RemindersPage'

vi.mock('../../services/reminderTemplates', () => ({
  listReminderTemplates: vi.fn(),
  toggleReminderTemplate: vi.fn(),
  duplicateReminderTemplate: vi.fn(),
}))

const mockedListReminderTemplates = vi.mocked(listReminderTemplates)
const mockedToggleReminderTemplate = vi.mocked(toggleReminderTemplate)
const mockedDuplicateReminderTemplate = vi.mocked(duplicateReminderTemplate)

const templates = [
  {
    id: 'tpl_1',
    title: '喝水提醒',
    scheduleSummary: '每天 08:00',
    eventTypeLabel: '文本提醒',
    enabled: true,
  },
]

describe('RemindersPage', () => {
  beforeEach(() => {
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
})
