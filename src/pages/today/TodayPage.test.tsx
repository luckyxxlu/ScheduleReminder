import { fireEvent, render, screen } from '@testing-library/react'

import { TodayPage } from './TodayPage'

describe('TodayPage', () => {
  it('renders next reminder and grace section', () => {
    render(<TodayPage />)

    expect(screen.getByText('下一条提醒')).toBeInTheDocument()
    expect(screen.getByText('宽容中的提醒')).toBeInTheDocument()
  })

  it('shows action buttons for quick handling', () => {
    render(<TodayPage />)

    expect(screen.getByRole('button', { name: '完成' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '宽容 10 分钟' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '稍后提醒' })).toBeInTheDocument()
  })

  it('updates status when marking today reminder as completed', () => {
    render(<TodayPage />)

    fireEvent.click(screen.getByRole('button', { name: '完成' }))

    expect(screen.getByText('已完成')).toBeInTheDocument()
  })
})
