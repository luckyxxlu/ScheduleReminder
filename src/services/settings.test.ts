import { invoke } from '@tauri-apps/api/core'

import { getSettings, updateSettings } from './settings'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const mockedInvoke = vi.mocked(invoke)

describe('settings service', () => {
  beforeEach(() => {
    mockedInvoke.mockReset()
  })

  it('loads settings from tauri command', async () => {
    mockedInvoke.mockResolvedValue({ defaultGraceMinutes: 10, closeToTrayOnClose: true })

    const result = await getSettings()

    expect(mockedInvoke).toHaveBeenCalledWith('get_settings')
    expect(result).toEqual({ defaultGraceMinutes: 10, closeToTrayOnClose: true })
  })

  it('updates settings through tauri command', async () => {
    mockedInvoke.mockResolvedValue({
      defaultGraceMinutes: 15,
      startupWithWindows: true,
      closeToTrayOnClose: false,
    })

    const result = await updateSettings({
      defaultGraceMinutes: 15,
      startupWithWindows: true,
      closeToTrayOnClose: false,
    })

    expect(mockedInvoke).toHaveBeenCalledWith('update_settings', {
      default_grace_minutes: 15,
      startup_with_windows: true,
      close_to_tray_on_close: false,
    })
    expect(result).toEqual({ defaultGraceMinutes: 15, startupWithWindows: true, closeToTrayOnClose: false })
  })
})
