import { useState } from 'react'

export function SettingsPage() {
  const [graceMinutes, setGraceMinutes] = useState('10')
  const [startup, setStartup] = useState(false)

  return (
    <section className="page-section">
      <header className="page-header">
        <h2>设置</h2>
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
        </div>
      </article>
    </section>
  )
}
