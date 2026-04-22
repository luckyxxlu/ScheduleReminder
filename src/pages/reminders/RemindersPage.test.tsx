import { fireEvent, render, screen } from '@testing-library/react'

import { RemindersPage } from './RemindersPage'

describe('RemindersPage', () => {
  it('renders reminder template list', () => {
    render(<RemindersPage />)

    expect(screen.getByText('提醒模板列表')).toBeInTheDocument()
    expect(screen.getByText('喝水提醒')).toBeInTheDocument()
  })

  it('toggles template enabled state', () => {
    render(<RemindersPage />)

    const toggle = screen.getByRole('button', { name: '已启用' })
    fireEvent.click(toggle)

    expect(screen.getByRole('button', { name: '已暂停' })).toBeInTheDocument()
  })

  it('renders edit and duplicate actions', () => {
    render(<RemindersPage />)

    expect(screen.getByRole('button', { name: '编辑 喝水提醒' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '复制 喝水提醒' })).toBeInTheDocument()
  })
})
