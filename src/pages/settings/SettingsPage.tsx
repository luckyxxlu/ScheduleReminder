import { useEffect, useState } from 'react'

import { getSettings, updateSettings } from '../../services/settings'
import { extractErrorMessage } from '../../utils/errors'

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
        setErrorMessage(extractErrorMessage(error, '设置加载失败'))
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
      setErrorMessage(extractErrorMessage(error, '设置保存失败'))
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
        <div className="action-row action-row-compact">
          <button className="primary-button" disabled={isLoading || isSaving} type="button" onClick={() => void handleSave()}>
            {isSaving ? '正在保存...' : '保存所有设置'}
          </button>
        </div>
      </header>

      {isLoading ? <p className="panel-text">正在加载设置...</p> : null}
      {successMessage ? <p className="success-text">{successMessage}</p> : null}
      {errorMessage ? <p className="error-text">{errorMessage}</p> : null}

      <div className="page-grid-two" style={{ minHeight: 'auto' }}>
        <article className="panel">
          <span className="panel-label">应用偏好</span>
          <strong className="panel-title">调度与宽容</strong>
          <p className="panel-text" style={{ marginBottom: 32 }}>调整提醒的缓冲时间与响应策略。</p>

          <div className="settings-group">
            <label className="settings-field">
              <span>默认宽容分钟数</span>
              <input
                aria-label="默认宽容时间"
                type="number"
                style={{ width: 120, textAlign: 'right' }}
                value={graceMinutes}
                onChange={(event) => setGraceMinutes(event.target.value)}
              />
            </label>
            <p className="panel-text" style={{ fontSize: 13, marginTop: 8 }}>当提醒触发后，你有这段时间来完成它，超时将被标记为错过。</p>
          </div>

          <div style={{ marginTop: 'auto', paddingTop: 24, borderTop: '1px solid rgba(0,0,0,0.05)' }}>
            <span className="panel-label">通知偏好 (开发中)</span>
            <label className="settings-checkbox" style={{ opacity: 0.7 }}>
              <span>系统横幅通知</span>
              <input type="checkbox" checked disabled />
            </label>
            <label className="settings-checkbox" style={{ opacity: 0.7 }}>
              <span>播放提示音</span>
              <input type="checkbox" checked disabled />
            </label>
          </div>
        </article>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '24px', overflowY: 'auto', paddingBottom: '24px' }}>
          <article className="panel">
            <span className="panel-label">系统集成</span>
            <strong className="panel-title">后台运行方式</strong>
            <p className="panel-text" style={{ marginBottom: 24 }}>让提醒在后台稳定工作。</p>

            <div className="settings-group" style={{ marginBottom: 0 }}>
              <label className="settings-checkbox">
                <span>开机自启</span>
                <input
                  aria-label="开机自启"
                  type="checkbox"
                  checked={startup}
                  onChange={(event) => setStartup(event.target.checked)}
                />
              </label>
              <p className="panel-text" style={{ fontSize: 13, marginLeft: 20, marginBottom: 12 }}>推荐开启，以便电脑启动后立刻接管全天提醒。</p>

              <label className="settings-checkbox" style={{ marginBottom: 4 }}>
                <span>关闭窗口时继续在后台运行</span>
                <input
                  aria-label="关闭时后台运行"
                  type="checkbox"
                  checked={closeToTrayOnClose}
                  onChange={(event) => setCloseToTrayOnClose(event.target.checked)}
                />
              </label>
              <p className="panel-text" style={{ fontSize: 13, marginLeft: 20 }}>开启后，点击窗口右上角关闭按钮不会退出应用，而是隐藏到 Windows 系统托盘，可通过托盘图标重新打开。</p>
            </div>
          </article>

          <article className="panel">
            <span className="panel-label">数据与支持</span>
            <strong className="panel-title">应用维护</strong>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12, marginTop: 16 }}>
              <button className="secondary-button" type="button" onClick={() => alert('数据已导出 (演示)')}>
                导出提醒数据
              </button>
              <button className="secondary-button" type="button" onClick={() => alert('当前已是最新版本')}>
                检查更新
              </button>
            </div>
            <p className="panel-text" style={{ fontSize: 12, marginTop: 16, textAlign: 'center', opacity: 0.5 }}>
              时间助手 v1.0.1 · 基于 Tauri 构建
            </p>
          </article>
        </div>
      </div>
    </section>
  )
}
