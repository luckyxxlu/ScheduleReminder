import { useEffect, useState } from 'react'

import { getTodayDashboard, markNextReminderCompleted, type TodayDashboardData } from '../../services/dashboard'

export function TodayPage() {
  const [dashboard, setDashboard] = useState<TodayDashboardData | null>(null)

  useEffect(() => {
    void getTodayDashboard().then(setDashboard)
  }, [])

  async function handleComplete() {
    const nextDashboard = await markNextReminderCompleted()
    setDashboard(nextDashboard)
  }

  return (
    <section className="page-section">
      <header className="page-header">
        <div>
          <h2>今天</h2>
          <p className="page-subtitle">先看下一条提醒，再决定今天的节奏要不要调整。</p>
        </div>
        <button className="primary-button" type="button">快速新建</button>
      </header>

      <div className="page-grid page-grid-two">
        <article className="panel panel-hero">
          <span className="panel-label">下一条提醒</span>
          {dashboard ? (
            <>
              <strong className="panel-title">
                {dashboard.nextReminderTime} {dashboard.nextReminderTitle}
              </strong>
              <p className="panel-text">{dashboard.nextReminderMessage}</p>
            </>
          ) : (
            <p className="panel-text">正在加载今日提醒...</p>
          )}
          <div className="action-row">
            <button className="secondary-button" type="button" onClick={() => void handleComplete()}>
              完成
            </button>
            <button className="secondary-button" type="button">
              宽容 10 分钟
            </button>
            <button className="secondary-button" type="button">
              稍后提醒
            </button>
          </div>
          <div className="today-timeline-preview">
            <div className="timeline-item timeline-item-active">
              <span>08:00</span>
              <strong>喝水提醒</strong>
            </div>
            <div className="timeline-item">
              <span>14:30</span>
              <strong>深度工作</strong>
            </div>
            <div className="timeline-item">
              <span>22:30</span>
              <strong>准备休息</strong>
            </div>
          </div>
        </article>

        <article className="panel">
          <span className="panel-label">今日状态</span>
          <strong className="panel-title">今天安排不多，适合稳步推进</strong>
          <p className="panel-text">你可以直接在日历页补充事件，也可以在提醒页启停固定模板。</p>
          <div className="today-metrics">
            <article className="metric-card">
              <span>待处理</span>
              <strong>2</strong>
            </article>
            <article className="metric-card">
              <span>宽容中</span>
              <strong>1</strong>
            </article>
            <article className="metric-card">
              <span>已完成</span>
              <strong>{dashboard?.highlightedStatus === '已完成' ? 1 : 0}</strong>
            </article>
          </div>
          <div className="status-block">
            <span className="panel-label">宽容中的提醒</span>
            <strong className={`status-chip status-${dashboard?.highlightedStatus ?? '宽容中'}`}>
              {dashboard?.highlightedStatus ?? '宽容中'}
            </strong>
          </div>
        </article>
      </div>
    </section>
  )
}
