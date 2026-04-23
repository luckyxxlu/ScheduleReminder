import { useEffect, useEffectEvent, useState } from 'react'
import { listen, TauriEvent } from '@tauri-apps/api/event'
import { useNavigate } from 'react-router-dom'

import { todayDashboardRefreshEvent } from '../../app/events'
import {
  getTodayDashboard,
  graceNextReminderTenMinutes,
  markNextReminderCompleted,
  skipNextReminder,
  snoozeNextReminder,
  type TodayDashboardData,
} from '../../services/dashboard'
import { extractErrorMessage } from '../../utils/errors'

const snoozeOptions = [5, 10, 15, 30] as const

export function TodayPage() {
  const navigate = useNavigate()
  const [dashboard, setDashboard] = useState<TodayDashboardData | null>(null)
  const [snoozeMinutes, setSnoozeMinutes] = useState<number>(15)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const [successMessage, setSuccessMessage] = useState<string | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)

  const loadDashboard = useEffectEvent((isMounted: () => boolean) => {
    getTodayDashboard()
      .then((nextDashboard) => {
        if (!isMounted()) {
          return
        }

        setDashboard(nextDashboard)
        setErrorMessage(null)
      })
      .catch((error: unknown) => {
        if (!isMounted()) {
          return
        }

        setErrorMessage(extractErrorMessage(error, '今天页加载失败'))
      })
  })

  useEffect(() => {
    let mounted = true
    const unlistenFns: Array<() => void> = []
    const isMounted = () => mounted

    loadDashboard(isMounted)
    const timer = window.setInterval(() => loadDashboard(isMounted), 15000)

    void Promise.all([
      listen(todayDashboardRefreshEvent, () => {
        loadDashboard(isMounted)
      }),
      listen(TauriEvent.WINDOW_FOCUS, () => {
        loadDashboard(isMounted)
      }),
    ]).then((disposeFns) => {
      if (!mounted) {
        disposeFns.forEach((dispose) => dispose())
        return
      }

      unlistenFns.push(...disposeFns)
    })

    return () => {
      mounted = false
      window.clearInterval(timer)
      unlistenFns.forEach((dispose) => dispose())
    }
  }, [])

  async function runAction(action: () => Promise<TodayDashboardData>, successText: string) {
    setIsSubmitting(true)

    try {
      const nextDashboard = await action()
      setDashboard(nextDashboard)
      setSuccessMessage(successText)
      setErrorMessage(null)
    } catch (error: unknown) {
      setErrorMessage(extractErrorMessage(error, '提醒处理失败'))
      setSuccessMessage(null)
    } finally {
      setIsSubmitting(false)
    }
  }

  async function handleGraceTenMinutes() {
    await runAction(graceNextReminderTenMinutes, '已将当前提醒宽容 10 分钟')
  }

  async function handleSnooze() {
    await runAction(() => snoozeNextReminder(snoozeMinutes), `已稍后提醒 ${snoozeMinutes} 分钟`)
  }

  async function handleSkip() {
    await runAction(skipNextReminder, '已跳过今天这条提醒')
  }

  async function handleComplete() {
    await runAction(markNextReminderCompleted, '已完成当前提醒')
  }

  const completedCount = dashboard?.todayTimeline.filter((item) => item.status === '已完成').length ?? 0
  const graceCount = dashboard?.todayTimeline.filter((item) => item.status === '宽容中').length ?? 0
  const pendingCount = dashboard?.todayTimeline.filter((item) => item.status === '待处理').length ?? 0
  const actionsEnabled = (dashboard?.nextReminderAvailableActions.length ?? 0) > 0 && !isSubmitting
  const statusSummary = buildStatusSummary(dashboard?.nextReminderStatus ?? '待处理', {
    pendingCount,
    graceCount,
    completedCount,
  })

  return (
    <section className="page-section">
      <header className="page-header">
        <div>
          <h2>今天</h2>
          <p className="page-subtitle">先看下一条提醒，再决定今天的节奏要不要调整。</p>
        </div>
        <button className="primary-button" type="button" onClick={() => navigate('/reminders')}>
          快速新建
        </button>
      </header>

      <div className="bento-grid">
        <article className="panel panel-hero bento-hero">
          <span className="panel-label">下一条提醒</span>
          {errorMessage ? <p className="error-text">{errorMessage}</p> : null}
          {successMessage ? <p className="success-text">{successMessage}</p> : null}
          {dashboard ? (
            <>
              <strong className="panel-title">
                {dashboard.nextReminderTime} {dashboard.nextReminderTitle}
              </strong>
              <p className="panel-text">{dashboard.nextReminderMessage}</p>
              <p className="panel-text">当前状态：{dashboard.nextReminderStatus}</p>
              <p className="panel-text">{dashboard.nextReminderNotificationState}</p>
              {dashboard.nextReminderGraceDeadline ? (
                <p className="panel-text">宽容截止：{dashboard.nextReminderGraceDeadline}</p>
              ) : null}
            </>
          ) : (
            <p className="panel-text">正在加载今日提醒...</p>
          )}
          <div className="action-row">
            <button className="primary-button" disabled={!actionsEnabled} type="button" onClick={() => void handleComplete()}>
              完成
            </button>
            <button className="secondary-button" disabled={!actionsEnabled} type="button" onClick={() => void handleGraceTenMinutes()}>
              宽容 10 分钟
            </button>
            <select
              aria-label="稍后提醒时长"
              className="inline-select"
              disabled={!actionsEnabled}
              value={snoozeMinutes}
              onChange={(event) => setSnoozeMinutes(Number(event.target.value))}
            >
              {snoozeOptions.map((minutes) => (
                <option key={minutes} value={minutes}>
                  {minutes} 分钟
                </option>
              ))}
            </select>
            <button className="secondary-button" disabled={!actionsEnabled} type="button" onClick={() => void handleSnooze()}>
              稍后提醒
            </button>
            <button className="secondary-button" disabled={!actionsEnabled} type="button" onClick={() => void handleSkip()}>
              跳过今天
            </button>
          </div>
          <div className="today-timeline-preview">
            {(dashboard?.todayTimeline ?? []).length === 0 ? (
              <p className="panel-text">今天还没有提醒时间线。</p>
            ) : (
              dashboard?.todayTimeline.map((item) => (
                <div className={`timeline-item${item.isActive ? ' timeline-item-active' : ''}`} key={item.id}>
                  <div>
                    <span className="time">{item.time}</span>
                    <strong>{item.title}</strong>
                    <p className="panel-text" style={{margin: 0}}>{item.message}</p>
                  </div>
                  <span className={`status-chip status-${item.status}`}>{item.status}</span>
                </div>
              ))
            )}
          </div>
        </article>

        <article className="panel bento-metrics">
          <span className="panel-label">今日状态</span>
          <strong className="panel-title">{statusSummary.title}</strong>
          <p className="panel-text">{statusSummary.description}</p>
          <div className="today-metrics">
            <div className="metric-card">
              <span>待处理</span>
              <strong>{pendingCount}</strong>
            </div>
            <div className="metric-card">
              <span>宽容中</span>
              <strong>{graceCount}</strong>
            </div>
            <div className="metric-card">
              <span>已完成</span>
              <strong>{completedCount}</strong>
            </div>
          </div>
        </article>

        <article className="panel bento-logs">
          <span className="panel-label">宽容中的提醒</span>
          <div style={{ marginBottom: 24 }}>
            <strong className={`status-chip status-${dashboard?.highlightedStatus ?? '宽容中'}`}>
              {dashboard?.highlightedStatus ?? '宽容中'}
            </strong>
          </div>
          
          <span className="panel-label">最近操作</span>
          <div className="today-log-list">
            {(dashboard?.recentActions ?? []).length === 0 ? (
              <p className="panel-text">今天还没有提醒处理记录。</p>
            ) : (
              dashboard?.recentActions.map((item) => (
                <div className="today-log-item" key={item.id}>
                  <strong>{item.actionLabel}</strong>
                  <span>{item.actionAt}</span>
                </div>
              ))
            )}
          </div>
        </article>
      </div>
    </section>
  )
}

function buildStatusSummary(
  nextReminderStatus: string,
  counts: { pendingCount: number; graceCount: number; completedCount: number },
) {
  if (nextReminderStatus === '宽容中') {
    return {
      title: '当前有提醒正在等待处理',
      description: `还有 ${counts.graceCount} 条提醒处于宽容时间，建议优先处理这条已触发通知的提醒。`,
    }
  }

  if (nextReminderStatus === '待处理') {
    return {
      title: '下一条提醒尚未触发通知',
      description: `今天还有 ${counts.pendingCount} 条待处理提醒，到点后会按计划发送 Windows 通知。`,
    }
  }

  if (counts.completedCount > 0) {
    return {
      title: '今天已经开始推进了',
      description: `已完成 ${counts.completedCount} 条提醒，你可以继续在日历页补充新的安排。`,
    }
  }

  return {
    title: '今天节奏比较轻',
    description: '你可以直接在日历页补充事件，也可以在提醒页启停固定模板。',
  }
}
