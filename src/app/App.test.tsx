import { render, screen } from '@testing-library/react'

import { App } from './App'

describe('App', () => {
  beforeEach(() => {
    window.location.hash = '#/'
  })

  it('renders the main navigation', () => {
    render(<App />)

    expect(screen.getByRole('navigation', { name: '主导航' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: '今天' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: '日历' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: '提醒' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: '设置' })).toBeInTheDocument()
  })

  it('renders the today page by default', () => {
    render(<App />)

    expect(screen.getByRole('heading', { name: '今天' })).toBeInTheDocument()
    expect(screen.getByText('下一条提醒')).toBeInTheDocument()
  })

  it('renders the calendar page when hash route changes', () => {
    window.location.hash = '#/calendar'

    render(<App />)

    expect(screen.getByRole('heading', { name: '日历' })).toBeInTheDocument()
    expect(screen.getByText('本月提醒分布')).toBeInTheDocument()
  })
})
