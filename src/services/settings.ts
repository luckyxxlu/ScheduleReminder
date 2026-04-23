import { invoke } from '@tauri-apps/api/core'

export type SettingsViewData = {
  defaultGraceMinutes: number
  startupWithWindows: boolean
  closeToTrayOnClose: boolean
  quietHoursEnabled: boolean
  quietHoursStart: string | null
  quietHoursEnd: string | null
}

export async function getSettings(): Promise<SettingsViewData> {
  return invoke<SettingsViewData>('get_settings')
}

export async function updateSettings(input: {
  defaultGraceMinutes: number
  startupWithWindows: boolean
  closeToTrayOnClose: boolean
}): Promise<SettingsViewData> {
  return invoke<SettingsViewData>('update_settings', {
    defaultGraceMinutes: input.defaultGraceMinutes,
    startupWithWindows: input.startupWithWindows,
    closeToTrayOnClose: input.closeToTrayOnClose,
  })
}
