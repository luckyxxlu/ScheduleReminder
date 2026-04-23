import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'

import {
  createReminderTemplate,
  duplicateReminderTemplate,
  listReminderTemplates,
  type ReminderTemplateListItem,
  toggleReminderTemplate,
  updateReminderTemplate,
} from '../../services/reminderTemplates'
import { extractErrorMessage } from '../../utils/errors'

export function RemindersPage() {
  const navigate = useNavigate()
  const [templates, setTemplates] = useState<ReminderTemplateListItem[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const [title, setTitle] = useState('')
  const [message, setMessage] = useState('')
  const [category, setCategory] = useState('')
  const [time, setTime] = useState(() => getCurrentTimeWithSeconds())
  const [repeatType, setRepeatType] = useState<'none' | 'daily' | 'workdays'>('daily')
  const [graceMinutes, setGraceMinutes] = useState('10')
  const [note, setNote] = useState('')
  const [successMessage, setSuccessMessage] = useState<string | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [editingTemplateId, setEditingTemplateId] = useState<string | null>(null)
  const [editingTemplateEnabled, setEditingTemplateEnabled] = useState(false)

  useEffect(() => {
    let isMounted = true

      listReminderTemplates()
      .then((items) => {
        if (!isMounted) {
          return
        }

        const filtered = items.filter(item => item.category !== 'calendar')
        setTemplates(filtered)
        setErrorMessage(null)
      })
      .catch((error: unknown) => {
        if (!isMounted) {
          return
        }

        setErrorMessage(extractErrorMessage(error, '提醒模板加载失败'))
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
    try {
      const updated = await toggleReminderTemplate(template.id, !template.enabled)
      setTemplates((current) =>
        current.map((item) => {
          return item.id === updated.id ? updated : item
        }),
      )
      setSuccessMessage(`${updated.title} 已${updated.enabled ? '启用' : '暂停'}`)
      setErrorMessage(null)
    } catch (error: unknown) {
      setErrorMessage(extractErrorMessage(error, '提醒模板状态更新失败'))
      setSuccessMessage(null)
    }
  }

  async function handleDuplicate(id: string) {
    try {
      const duplicated = await duplicateReminderTemplate(id)
      setTemplates((current) => [...current, duplicated])
      setSuccessMessage(`已复制模板 ${duplicated.title}`)
      setErrorMessage(null)
    } catch (error: unknown) {
      setErrorMessage(extractErrorMessage(error, '复制提醒模板失败'))
      setSuccessMessage(null)
    }
  }

  async function handleCreateTemplate() {
    if (!title.trim() || !message.trim()) {
      setErrorMessage('请先填写提醒标题和提醒内容')
      setSuccessMessage(null)
      return
    }

    setIsSubmitting(true)

    try {
      const created = await createReminderTemplate({
        title,
        message,
        category,
        repeatRuleJson:
          repeatType === 'none'
            ? `{"type":"none","time":"${time}"}`
            : repeatType === 'workdays'
              ? `{"type":"workdays","time":"${time}"}`
              : `{"type":"daily","interval":1,"time":"${time}"}`,
        defaultGraceMinutes: Number(graceMinutes),
        note,
      })

      if (created.category !== 'calendar') {
        setTemplates((current) => [...current, created])
      }
      setTitle('')
      setMessage('')
      setCategory('')
      setTime(getCurrentTimeWithSeconds())
      setRepeatType('daily')
      setGraceMinutes('10')
      setNote('')
      setErrorMessage(null)
      setSuccessMessage(`已保存提醒模板 ${created.title}`)
    } catch (error: unknown) {
      setErrorMessage(extractErrorMessage(error, '保存提醒模板失败'))
      setSuccessMessage(null)
    } finally {
      setIsSubmitting(false)
    }
  }

  async function handleEditTemplate(editingId: string, enabled: boolean) {
    if (!title.trim() || !message.trim()) {
      setErrorMessage('请先填写提醒标题和提醒内容')
      setSuccessMessage(null)
      return
    }

    setIsSubmitting(true)

    try {
      const updated = await updateReminderTemplate({
        id: editingId,
        title,
        message,
        category,
        repeatRuleJson:
          repeatType === 'none'
            ? `{"type":"none","time":"${time}"}`
            : repeatType === 'workdays'
              ? `{"type":"workdays","time":"${time}"}`
              : `{"type":"daily","interval":1,"time":"${time}"}`,
        defaultGraceMinutes: Number(graceMinutes),
        note,
        enabled,
      })

      setTemplates((current) => current.map((item) => (item.id === updated.id ? updated : item)))
      resetForm()
      setSuccessMessage(`已更新提醒模板 ${updated.title}`)
      setErrorMessage(null)
    } catch (error: unknown) {
      setErrorMessage(extractErrorMessage(error, '更新提醒模板失败'))
      setSuccessMessage(null)
    } finally {
      setIsSubmitting(false)
    }
  }

  function startEditing(template: ReminderTemplateListItem) {
    setEditingTemplateId(template.id)
    setEditingTemplateEnabled(template.enabled)
    setTitle(template.title)
    setMessage(template.message)
    setCategory(template.category ?? '')
    setTime(extractTimeFromRepeatRule(template.repeatRuleJson, '09:00'))
    setRepeatType(extractRepeatType(template.repeatRuleJson))
    setGraceMinutes(String(template.defaultGraceMinutes))
    setNote(template.note ?? '')
    setSuccessMessage(`正在编辑 ${template.title}`)
    setErrorMessage(null)
  }

  function resetForm() {
    setEditingTemplateId(null)
    setEditingTemplateEnabled(false)
    setTitle('')
    setMessage('')
    setCategory('')
    setTime('09:00')
    setRepeatType('daily')
    setGraceMinutes('10')
    setNote('')
  }

  return (
    <section className="page-section">
      <header className="page-header">
        <div>
          <h2>提醒</h2>
          <p className="page-subtitle">固定作息模板集中管理，日历里新建的单次事件也会出现在这里。</p>
        </div>
        <button className="primary-button" disabled={isSubmitting} type="button" onClick={() => (editingTemplateId ? resetForm() : undefined)}>
          {isSubmitting ? '正在保存...' : editingTemplateId ? '开始新建' : '新建提醒模板'}
        </button>
      </header>

      <div className="page-grid page-grid-two">
        <article className="panel">
          <span className="panel-label">快速创建</span>
          <strong className="panel-title">{editingTemplateId ? '编辑提醒模板' : '真正可用的提醒模板'}</strong>
          <p className="panel-text">标题和提醒内容完全分开填写。你可以把标题写成“深度工作”，内容写成“开始 45 分钟专注工作”。</p>
          {successMessage ? <p className="success-text">{successMessage}</p> : null}
          {errorMessage ? <p className="error-text">{errorMessage}</p> : null}
          <div className="reminder-form-grid" style={{ marginTop: 24 }}>
            <label className="calendar-input-group">
              <span>标题</span>
              <input aria-label="提醒标题" value={title} onChange={(event) => setTitle(event.target.value)} />
            </label>
            <label className="calendar-input-group">
              <span>提醒内容</span>
              <input aria-label="提醒内容" value={message} onChange={(event) => setMessage(event.target.value)} />
            </label>
            <label className="calendar-input-group">
              <span>分类</span>
              <input aria-label="提醒分类" value={category} onChange={(event) => setCategory(event.target.value)} />
            </label>
            <label className="calendar-input-group">
              <span>时间</span>
              <input aria-label="模板时间" type="time" step="1" value={time} onChange={(event) => setTime(event.target.value)} />
            </label>
            <label className="calendar-input-group">
              <span>重复方式</span>
              <select className="inline-select" aria-label="重复方式" value={repeatType} onChange={(event) => setRepeatType(event.target.value as 'none' | 'daily' | 'workdays')} style={{ width: '100%', marginTop: 0 }}>
                <option value="none">单次</option>
                <option value="daily">每天</option>
                <option value="workdays">工作日</option>
              </select>
            </label>
            <label className="calendar-input-group">
              <span>宽容分钟</span>
              <input aria-label="模板宽容时间" type="number" min="0" value={graceMinutes} onChange={(event) => setGraceMinutes(event.target.value)} />
            </label>
            <label className="calendar-input-group reminder-form-full">
              <span>备注</span>
              <input aria-label="提醒备注" value={note} onChange={(event) => setNote(event.target.value)} />
            </label>
          </div>
          <div className="action-row">
            <button
              className="primary-button"
              disabled={isSubmitting}
              type="button"
              onClick={() => void (editingTemplateId ? handleEditTemplate(editingTemplateId, editingTemplateEnabled) : handleCreateTemplate())}
            >
              {isSubmitting ? '正在保存...' : editingTemplateId ? '保存修改' : '保存模板'}
            </button>
            {editingTemplateId ? (
              <button className="secondary-button" type="button" onClick={resetForm}>
                取消编辑
              </button>
            ) : null}
            <button className="secondary-button" type="button" onClick={() => navigate('/calendar')}>
              去日历查看
            </button>
          </div>
        </article>

        <article className="panel">
          <span className="panel-label">提醒模板列表</span>
          <strong className="panel-title">全部提醒</strong>
          <p className="panel-text">支持启停、复制与查看规则</p>

          <div className="reminders-panel" style={{ marginTop: 0 }}>
            {isLoading ? <p className="template-row panel-text">正在加载提醒模板...</p> : null}

            {templates.map((template) => (
              <div className="template-card" key={template.id}>
                <div className="template-card-header">
                  <div>
                    <strong>{template.title}</strong>
                    <p className="panel-text" style={{ marginBottom: 0 }}>
                      {template.scheduleSummary} | {template.eventTypeLabel}
                    </p>
                  </div>
                  <span className={`status-chip ${template.enabled ? 'status-已完成' : 'status-已跳过'}`}>
                    {template.enabled ? '运行中' : '已暂停'}
                  </span>
                </div>
                <div className="action-row" style={{ marginTop: 16 }}>
                  <button className="secondary-button" style={{ minHeight: 36, padding: '0 12px', fontSize: 13 }} type="button" onClick={() => handleToggle(template)}>
                    {template.enabled ? '停用' : '启用'}
                  </button>
                  <button aria-label={`编辑 ${template.title}`} className="secondary-button" style={{ minHeight: 36, padding: '0 12px', fontSize: 13 }} type="button" onClick={() => startEditing(template)}>
                    编辑
                  </button>
                  <button
                    aria-label={`复制 ${template.title}`}
                    className="secondary-button"
                    style={{ minHeight: 36, padding: '0 12px', fontSize: 13 }}
                    type="button"
                    onClick={() => handleDuplicate(template.id)}
                  >
                    复制
                  </button>
                </div>
              </div>
            ))}
          </div>
        </article>
      </div>
    </section>
  )
}

function getCurrentTimeWithSeconds() {
  const d = new Date()
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

function extractRepeatType(repeatRuleJson: string): 'none' | 'daily' | 'workdays' {
  if (repeatRuleJson.includes('workdays')) {
    return 'workdays'
  }

  if (repeatRuleJson.includes('none')) {
    return 'none'
  }

  return 'daily'
}

function extractTimeFromRepeatRule(repeatRuleJson: string, fallback = getCurrentTimeWithSeconds()) {
  const match = repeatRuleJson.match(/"time":"(\d{2}:\d{2}(:\d{2})?)"/)
  return match?.[1] ?? fallback
}
