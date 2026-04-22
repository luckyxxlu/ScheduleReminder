import { useEffect, useState } from 'react'

import { getSettings, updateSettings } from '../../services/settings'

export function SettingsPage() {
  const [graceMinutes, setGraceMinutes] = useState('10')
  const [startup, setStartup] = useState(false)
  const [closeToTrayOnClose, setCloseToTrayOnClose] = useState(true)

  useEffect(() => {
    void getSettings().then((settings) => {
      setGraceMinutes(String(settings.defaultGraceMinutes))
      setStartup(settings.startupWithWindows)
      setCloseToTrayOnClose(settings.closeToTrayOnClose)
    })
  }, [])

  async function handleGraceMinutesChange(value: string) {
    setGraceMinutes(value)
    const parsed = Number(value)
    if (Number.isNaN(parsed)) {
      return
    }

    const next = await updateSettings({
      defaultGraceMinutes: parsed,
      startupWithWindows: startup,
      closeToTrayOnClose,
    })
    setGraceMinutes(String(next.defaultGraceMinutes))
  }

  async function handleStartupChange(checked: boolean) {
    setStartup(checked)
    const next = await updateSettings({
      defaultGraceMinutes: Number(graceMinutes),
      startupWithWindows: checked,
      closeToTrayOnClose,
    })
    setStartup(next.startupWithWindows)
  }

  async function handleCloseBehaviorChange(checked: boolean) {
    setCloseToTrayOnClose(checked)
    const next = await updateSettings({
      defaultGraceMinutes: Number(graceMinutes),
      startupWithWindows: startup,
      closeToTrayOnClose: checked,
    })
    setCloseToTrayOnClose(next.closeToTrayOnClose)
  }

  return (
    <section className="page-section">
      <header className="page-header">
        <div>
          <h2>设置</h2>
          <p className="page-subtitle">通知策略、后台运行方式和系统行为都在这里统一调整。</p>
        </div>
      </header>

      <article className="panel">
        <span className="panel-label">应用偏好</span>
        <strong className="panel-title">默认宽容时间与系统行为</strong>
        <div className="settings-group">
          <label className="settings-field">
            <span>默认宽容时间</span>
            <input
              aria-label="默认宽容时间"
              type="number"
              value={graceMinutes}
              onChange={(event) => void handleGraceMinutesChange(event.target.value)}
            />
          </label>

          <fieldset className="settings-fieldset">
            <legend>免打扰</legend>
            <p className="panel-text">22:00 - 07:00</p>
          </fieldset>

          <label className="settings-checkbox">
            <input
              aria-label="开机自启"
              type="checkbox"
              checked={startup}
              onChange={(event) => void handleStartupChange(event.target.checked)}
            />
            <span>开机自启</span>
          </label>

          <label className="settings-checkbox">
            <input
              aria-label="关闭时后台运行"
              type="checkbox"
              checked={closeToTrayOnClose}
              onChange={(event) => void handleCloseBehaviorChange(event.target.checked)}
            />
            <span>关闭窗口时继续在后台运行</span>
          </label>

          <p className="panel-text">开启后，点击窗口右上角关闭按钮不会退出应用，而是隐藏到后台继续运行。</p>
        </div>
      </article>
    </section>
  )
}
