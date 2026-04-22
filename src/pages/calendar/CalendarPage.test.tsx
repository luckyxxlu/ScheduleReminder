import { fireEvent, render, screen } from '@testing-library/react'

import { CalendarPage } from './CalendarPage'

describe('CalendarPage', () => {
  it('renders monthly reminder distribution', () => {
    render(<CalendarPage />)

    expect(screen.getByText('本月提醒分布')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '回到今天' })).toBeInTheDocument()
  })

  it('shows selected date detail when clicking a day', () => {
    render(<CalendarPage />)

    fireEvent.click(screen.getByRole('button', { name: '22日' }))

    expect(screen.getByText('2026-04-22')).toBeInTheDocument()
    expect(screen.getByText('喝水提醒')).toBeInTheDocument()
  })
})
