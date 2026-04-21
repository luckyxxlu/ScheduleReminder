export function CalendarPage() {
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
      </article>
    </section>
  )
}
