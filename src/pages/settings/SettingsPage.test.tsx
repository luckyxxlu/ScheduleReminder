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
    mockedGetSettings.mockReset()
    mockedUpdateSettings.mockReset()

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

  it('renders settings sections', async () => {
    render(<SettingsPage />)

    expect(await screen.findByText('默认宽容时间')).toBeInTheDocument()
    expect(screen.getByText('免打扰')).toBeInTheDocument()
    expect(screen.getByText('关闭窗口时继续在后台运行')).toBeInTheDocument()
  })

  it('updates grace minutes input value', async () => {
    render(<SettingsPage />)

    const input = (await screen.findByLabelText('默认宽容时间')) as HTMLInputElement
    fireEvent.change(input, { target: { value: '15' } })
    fireEvent.click(screen.getByRole('button', { name: '保存设置' }))

    await waitFor(() => {
      expect(mockedUpdateSettings).toHaveBeenCalledWith({
        defaultGraceMinutes: 15,
        startupWithWindows: false,
        closeToTrayOnClose: true,
      })
      expect(input.value).toBe('15')
      expect(screen.getByText('设置已保存')).toBeInTheDocument()
    })
  })

  it('toggles startup option', async () => {
    render(<SettingsPage />)

    const checkbox = (await screen.findByLabelText('开机自启')) as HTMLInputElement
    fireEvent.click(checkbox)
    fireEvent.click(screen.getByRole('button', { name: '保存设置' }))

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

    const checkbox = (await screen.findByLabelText('关闭时后台运行')) as HTMLInputElement
    fireEvent.click(checkbox)
    fireEvent.click(screen.getByRole('button', { name: '保存设置' }))

    await waitFor(() => {
      expect(mockedUpdateSettings).toHaveBeenCalledWith({
        defaultGraceMinutes: 10,
        startupWithWindows: false,
        closeToTrayOnClose: false,
      })
      expect(checkbox.checked).toBe(false)
    })
  })

  it('shows visible error when settings save fails', async () => {
    mockedUpdateSettings.mockRejectedValueOnce(new Error('数据库连接失败'))

    render(<SettingsPage />)

    fireEvent.change(await screen.findByLabelText('默认宽容时间'), { target: { value: '12' } })
    fireEvent.click(screen.getByRole('button', { name: '保存设置' }))

    expect(await screen.findByText('数据库连接失败')).toBeInTheDocument()
  })

  it('blocks saving when grace minutes is invalid', async () => {
    render(<SettingsPage />)

    fireEvent.change(await screen.findByLabelText('默认宽容时间'), { target: { value: '-1' } })
    fireEvent.click(screen.getByRole('button', { name: '保存设置' }))

    expect(await screen.findByText('默认宽容时间必须是大于等于 0 的数字')).toBeInTheDocument()
    expect(mockedUpdateSettings).not.toHaveBeenCalled()
  })
})
