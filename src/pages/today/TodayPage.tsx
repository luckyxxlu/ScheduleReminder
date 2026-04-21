export function TodayPage() {
  return (
    <section className="page-section">
      <header className="page-header">
        <h2>今天</h2>
        <button className="primary-button" type="button">
          新建提醒
        </button>
      </header>

      <div className="page-grid page-grid-two">
        <article className="panel panel-hero">
          <span className="panel-label">下一条提醒</span>
          <strong className="panel-title">22:30 准备休息</strong>
          <p className="panel-text">宽容 15 分钟，支持稍后提醒与跳过今天。</p>
        </article>

        <article className="panel">
          <span className="panel-label">今日状态</span>
          <strong className="panel-title">你的节奏很稳定</strong>
          <p className="panel-text">今日时间线、宽容中的提醒和完成数会显示在这里。</p>
        </article>
      </div>
    </section>
  )
}
