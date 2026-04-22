import { useState } from 'react'

export function CalendarPage() {
  const [selectedDate, setSelectedDate] = useState('2026-04-21')

  return (
    <section className="page-section">
      <header className="page-header">
        <h2>日历</h2>
        <button className="secondary-button" type="button">
          回到今天
        </button>
      </header>

      <article className="panel">
        <span className="panel-label">本月提醒分布</span>
        <strong className="panel-title">月视图骨架</strong>
        <p className="panel-text">后续将在这里渲染提醒分布、日期详情和当日状态。</p>
        <div className="calendar-grid">
          <button className="calendar-day" type="button" onClick={() => setSelectedDate('2026-04-22')}>
            22日
          </button>
          <button className="calendar-day" type="button" onClick={() => setSelectedDate('2026-04-23')}>
            23日
          </button>
        </div>
        <div className="date-detail">
          <strong>{selectedDate}</strong>
          <p className="panel-text">喝水提醒</p>
        </div>
      </article>
    </section>
  )
}
