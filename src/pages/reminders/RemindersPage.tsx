import { useEffect, useState } from 'react'

import {
  duplicateReminderTemplate,
  listReminderTemplates,
  type ReminderTemplateListItem,
  toggleReminderTemplate,
} from '../../services/reminderTemplates'

export function RemindersPage() {
  const [templates, setTemplates] = useState<ReminderTemplateListItem[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)

  useEffect(() => {
    let isMounted = true

    listReminderTemplates()
      .then((items) => {
        if (!isMounted) {
          return
        }

        setTemplates(items)
        setErrorMessage(null)
      })
      .catch((error: unknown) => {
        if (!isMounted) {
          return
        }

        setErrorMessage(error instanceof Error ? error.message : '提醒模板加载失败')
      })
      .finally(() => {
        if (isMounted) {
          setIsLoading(false)
        }
      })

    return () => {
      isMounted = false
    }
  }, [])

  async function handleToggle(template: ReminderTemplateListItem) {
    const updated = await toggleReminderTemplate(template.id, !template.enabled)
    setTemplates((current) =>
      current.map((item) => {
        return item.id === updated.id ? updated : item
      }),
    )
  }

  async function handleDuplicate(id: string) {
    const duplicated = await duplicateReminderTemplate(id)
    setTemplates((current) => [...current, duplicated])
  }

  return (
    <section className="page-section">
      <header className="page-header">
        <div>
          <h2>提醒</h2>
          <p className="page-subtitle">固定作息模板集中管理，日历里新建的单次事件也会出现在这里。</p>
        </div>
        <button className="primary-button" type="button">新建提醒模板</button>
      </header>

      <article className="panel reminders-panel">
        <span className="panel-label">提醒模板列表</span>
        <strong className="panel-title">统一管理全部提醒</strong>
        <p className="panel-text">支持启停、复制与查看当前重复规则，后续新建/编辑会继续在这里完善。</p>

        {isLoading ? <p className="template-row panel-text">正在加载提醒模板...</p> : null}
        {errorMessage ? <p className="template-row error-text">{errorMessage}</p> : null}

        {templates.map((template) => (
          <div className="template-card" key={template.id}>
            <div className="template-card-header">
              <div>
                <strong>{template.title}</strong>
                <p className="panel-text">
                  {template.scheduleSummary} | {template.eventTypeLabel}
                </p>
              </div>
              <span className={`status-chip ${template.enabled ? 'status-已完成' : 'status-已跳过'}`}>
                {template.enabled ? '运行中' : '已暂停'}
              </span>
            </div>
            <div className="action-row">
              <button className="secondary-button" type="button" onClick={() => handleToggle(template)}>
                {template.enabled ? '已启用' : '已暂停'}
              </button>
              <button aria-label={`编辑 ${template.title}`} className="secondary-button" type="button">
                编辑
              </button>
              <button
                aria-label={`复制 ${template.title}`}
                className="secondary-button"
                type="button"
                onClick={() => handleDuplicate(template.id)}
              >
                复制
              </button>
            </div>
          </div>
        ))}
      </article>
    </section>
  )
}
