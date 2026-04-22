import { fireEvent, render, screen, waitFor } from '@testing-library/react'

import { getSettings, updateSettings } from '../../services/settings'

import { SettingsPage } from './SettingsPage'

vi.mock('../../services/settings', () => ({
  getSettings: vi.fn(),
  updateSettings: vi.fn(),
}))

const mockedGetSettings = vi.mocked(getSettings)
const mockedUpdateSettings = vi.mocked(updateSettings)

describe('SettingsPage', () => {
  beforeEach(() => {
    mockedGetSettings.mockResolvedValue({
      defaultGraceMinutes: 10,
      startupWithWindows: false,
      closeToTrayOnClose: true,
      quietHoursEnabled: true,
      quietHoursStart: '22:00',
      quietHoursEnd: '07:00',
    })
    mockedUpdateSettings.mockImplementation(async (input) => ({
      defaultGraceMinutes: input.defaultGraceMinutes,
      startupWithWindows: input.startupWithWindows,
      closeToTrayOnClose: input.closeToTrayOnClose,
      quietHoursEnabled: true,
      quietHoursStart: '22:00',
      quietHoursEnd: '07:00',
    }))
  })

  it('renders settings sections', () => {
    render(<SettingsPage />)

    expect(screen.getByText('默认宽容时间')).toBeInTheDocument()
    expect(screen.getByText('免打扰')).toBeInTheDocument()
    expect(screen.getByText('关闭窗口时继续在后台运行')).toBeInTheDocument()
  })

  it('updates grace minutes input value', async () => {
    render(<SettingsPage />)

    const input = screen.getByLabelText('默认宽容时间') as HTMLInputElement
    fireEvent.change(input, { target: { value: '15' } })

    await waitFor(() => {
      expect(mockedUpdateSettings).toHaveBeenCalledWith({
        defaultGraceMinutes: 15,
        startupWithWindows: false,
        closeToTrayOnClose: true,
      })
      expect(input.value).toBe('15')
    })
  })

  it('toggles startup option', async () => {
    render(<SettingsPage />)

    const checkbox = screen.getByLabelText('开机自启') as HTMLInputElement
    fireEvent.click(checkbox)

    await waitFor(() => {
      expect(mockedUpdateSettings).toHaveBeenCalledWith({
        defaultGraceMinutes: 10,
        startupWithWindows: true,
        closeToTrayOnClose: true,
      })
      expect(checkbox.checked).toBe(true)
    })
  })

  it('updates close behavior option', async () => {
    render(<SettingsPage />)

    const checkbox = screen.getByLabelText('关闭时后台运行') as HTMLInputElement
    fireEvent.click(checkbox)

    await waitFor(() => {
      expect(mockedUpdateSettings).toHaveBeenCalledWith({
        defaultGraceMinutes: 10,
        startupWithWindows: false,
        closeToTrayOnClose: false,
      })
      expect(checkbox.checked).toBe(false)
    })
  })
})
