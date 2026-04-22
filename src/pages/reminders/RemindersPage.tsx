import { useState } from 'react'

export function RemindersPage() {
  const [enabled, setEnabled] = useState(true)

  return (
    <section className="page-section">
      <header className="page-header">
        <h2>提醒</h2>
        <button className="primary-button" type="button">
          新建提醒模板
        </button>
      </header>

      <article className="panel">
        <span className="panel-label">提醒模板列表</span>
        <strong className="panel-title">统一管理全部提醒</strong>
        <p className="panel-text">后续会在这里接入筛选、启停、编辑、复制和删除操作。</p>
        <div className="template-row">
          <div>
            <strong>喝水提醒</strong>
            <p className="panel-text">每天 08:00 | 文本提醒</p>
          </div>
          <div className="action-row">
            <button className="secondary-button" type="button" onClick={() => setEnabled((value) => !value)}>
              {enabled ? '已启用' : '已暂停'}
            </button>
            <button aria-label="编辑 喝水提醒" className="secondary-button" type="button">
              编辑
            </button>
            <button aria-label="复制 喝水提醒" className="secondary-button" type="button">
              复制
            </button>
          </div>
        </div>
      </article>
    </section>
  )
}
