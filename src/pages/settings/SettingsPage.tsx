import { useEffect, useState } from 'react'

import { getSettings, updateSettings } from '../../services/settings'

export function SettingsPage() {
  const [graceMinutes, setGraceMinutes] = useState('10')
  const [startup, setStartup] = useState(false)
  const [closeToTrayOnClose, setCloseToTrayOnClose] = useState(true)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const [successMessage, setSuccessMessage] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [isSaving, setIsSaving] = useState(false)

  useEffect(() => {
    getSettings()
      .then((settings) => {
        setGraceMinutes(String(settings.defaultGraceMinutes))
        setStartup(settings.startupWithWindows)
        setCloseToTrayOnClose(settings.closeToTrayOnClose)
        setErrorMessage(null)
      })
      .catch((error: unknown) => {
        setErrorMessage(error instanceof Error ? error.message : '设置加载失败')
      })
      .finally(() => {
        setIsLoading(false)
      })
  }, [])

  async function handleSave() {
    const parsed = Number(graceMinutes)
    if (Number.isNaN(parsed) || parsed < 0) {
      setErrorMessage('默认宽容时间必须是大于等于 0 的数字')
      setSuccessMessage(null)
      return
    }

    setIsSaving(true)

    try {
      const next = await updateSettings({
        defaultGraceMinutes: parsed,
        startupWithWindows: startup,
        closeToTrayOnClose,
      })
      setGraceMinutes(String(next.defaultGraceMinutes))
      setStartup(next.startupWithWindows)
      setCloseToTrayOnClose(next.closeToTrayOnClose)
      setErrorMessage(null)
      setSuccessMessage('设置已保存')
    } catch (error: unknown) {
      setErrorMessage(error instanceof Error ? error.message : '设置保存失败')
      setSuccessMessage(null)
    } finally {
      setIsSaving(false)
    }
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
        {isLoading ? <p className="panel-text">正在加载设置...</p> : null}
        {successMessage ? <p className="success-text">{successMessage}</p> : null}
        {errorMessage ? <p className="error-text">{errorMessage}</p> : null}
        <div className="settings-group">
          <label className="settings-field">
            <span>默认宽容时间</span>
            <input
              aria-label="默认宽容时间"
              type="number"
              value={graceMinutes}
              onChange={(event) => setGraceMinutes(event.target.value)}
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
              onChange={(event) => setStartup(event.target.checked)}
            />
            <span>开机自启</span>
          </label>

          <label className="settings-checkbox">
            <input
              aria-label="关闭时后台运行"
              type="checkbox"
              checked={closeToTrayOnClose}
              onChange={(event) => setCloseToTrayOnClose(event.target.checked)}
            />
            <span>关闭窗口时继续在后台运行</span>
          </label>

          <p className="panel-text">开启后，点击窗口右上角关闭按钮不会退出应用，而是隐藏到后台继续运行。</p>
          <div className="action-row">
            <button className="primary-button" disabled={isLoading || isSaving} type="button" onClick={() => void handleSave()}>
              {isSaving ? '正在保存...' : '保存设置'}
            </button>
          </div>
        </div>
      </article>
    </section>
  )
}
