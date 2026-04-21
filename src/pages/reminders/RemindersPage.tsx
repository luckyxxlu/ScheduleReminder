export function RemindersPage() {
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
      </article>
    </section>
  )
}
