import { fireEvent, render, screen } from '@testing-library/react'

import { SettingsPage } from './SettingsPage'

describe('SettingsPage', () => {
  it('renders settings sections', () => {
    render(<SettingsPage />)

    expect(screen.getByText('默认宽容时间')).toBeInTheDocument()
    expect(screen.getByText('免打扰')).toBeInTheDocument()
  })

  it('updates grace minutes input value', () => {
    render(<SettingsPage />)

    const input = screen.getByLabelText('默认宽容时间') as HTMLInputElement
    fireEvent.change(input, { target: { value: '15' } })

    expect(input.value).toBe('15')
  })

  it('toggles startup option', () => {
    render(<SettingsPage />)

    const checkbox = screen.getByLabelText('开机自启') as HTMLInputElement
    fireEvent.click(checkbox)

    expect(checkbox.checked).toBe(true)
  })
})
