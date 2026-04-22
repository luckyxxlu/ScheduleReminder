import { useEffect, useMemo, useState } from 'react'

import { createCalendarEvent, getCalendarOverview, type CalendarOverviewData } from '../../services/dashboard'

const weekdayLabels = ['一', '二', '三', '四', '五', '六', '日']
const initialSelectedDate = '2026-04-22'

type CalendarCell = {
  date: string
  day: number
  inCurrentMonth: boolean
  reminderCount: number
}

export function CalendarPage() {
  const [selectedDate, setSelectedDate] = useState(initialSelectedDate)
  const [overview, setOverview] = useState<CalendarOverviewData | null>(null)
  const [draftTitle, setDraftTitle] = useState('')
  const [draftTime, setDraftTime] = useState('19:30')

  useEffect(() => {
    void getCalendarOverview(selectedDate).then(setOverview)
  }, [selectedDate])

  const calendarCells = useMemo(() => {
    return buildCalendarCells(overview?.monthKey ?? monthKeyFromDate(selectedDate), overview?.monthEntries ?? [])
  }, [overview?.monthEntries, overview?.monthKey, selectedDate])

  async function handleCreateEvent() {
    if (!draftTitle.trim()) {
      return
    }

    const nextOverview = await createCalendarEvent({
      title: draftTitle,
      selectedDate,
      time: draftTime,
    })

    setOverview(nextOverview)
    setDraftTitle('')
  }

  return (
    <section className="page-section">
      <header className="page-header">
        <div>
          <h2>日历</h2>
          <p className="page-subtitle">完整查看提醒分布，并为任意一天快速补充事件。</p>
        </div>
        <button className="secondary-button" type="button" onClick={() => setSelectedDate(initialSelectedDate)}>
          回到今天
        </button>
      </header>

      <div className="calendar-layout">
        <article className="panel calendar-panel">
          <div className="calendar-topbar">
            <div>
              <span className="panel-label">本月提醒分布</span>
              <strong className="panel-title">{formatMonthLabel(overview?.monthKey ?? monthKeyFromDate(selectedDate))}</strong>
            </div>
            <span className="calendar-summary">本月已安排 {(overview?.monthEntries ?? []).reduce((sum, item) => sum + item.reminderCount, 0)} 条提醒</span>
          </div>

          <div className="calendar-weekdays">
            {weekdayLabels.map((label) => (
              <span className="calendar-weekday" key={label}>
                {label}
              </span>
            ))}
          </div>

          <div className="calendar-month-grid">
            {calendarCells.map((cell) => {
              const isSelected = cell.date === selectedDate
              const isToday = cell.date === initialSelectedDate

              return (
                <button
                  key={cell.date}
                  className={`calendar-month-cell${isSelected ? ' calendar-month-cell-selected' : ''}${
                    !cell.inCurrentMonth ? ' calendar-month-cell-muted' : ''
                  }${isToday ? ' calendar-month-cell-today' : ''}`}
                  aria-label={`选择 ${cell.date}`}
                  type="button"
                  onClick={() => setSelectedDate(cell.date)}
                >
                  <span className="calendar-month-day">{cell.day}</span>
                  <span className="calendar-month-meta">{cell.reminderCount > 0 ? `${cell.reminderCount} 条提醒` : '空闲'}</span>
                </button>
              )
            })}
          </div>
        </article>

        <aside className="panel calendar-detail-panel">
          <span className="panel-label">选中日期</span>
          <strong className="panel-title">{selectedDate}</strong>
          <p className="panel-text">像日历 App 一样查看当日安排，并直接补充新的提醒事件。</p>

          <div className="calendar-event-list">
            {(overview?.entries ?? []).length === 0 ? (
              <div className="calendar-event-empty">
                <strong>今天还没有安排</strong>
                <p className="panel-text">为这一天添加一个提醒事件，稍后会出现在今天页与提醒管理中。</p>
              </div>
            ) : (
              overview?.entries.map((entry) => (
                <article className="calendar-event-card" key={entry.id}>
                  <div>
                    <strong>{entry.title}</strong>
                    <p className="panel-text">{entry.time} · {entry.status}</p>
                  </div>
                </article>
              ))
            )}
          </div>

          <div className="calendar-create-box">
            <span className="panel-label">添加当天事件</span>
            <label className="calendar-input-group">
              <span>事件标题</span>
              <input aria-label="事件标题" value={draftTitle} onChange={(event) => setDraftTitle(event.target.value)} />
            </label>
            <label className="calendar-input-group">
              <span>提醒时间</span>
              <input
                aria-label="提醒时间"
                type="time"
                value={draftTime}
                onChange={(event) => setDraftTime(event.target.value)}
              />
            </label>
            <button className="primary-button calendar-submit-button" type="button" onClick={() => void handleCreateEvent()}>
              添加到 {selectedDate}
            </button>
          </div>
        </aside>
      </div>
    </section>
  )
}

function buildCalendarCells(monthKey: string, monthEntries: CalendarOverviewData['monthEntries']): CalendarCell[] {
  const [year, month] = monthKey.split('-').map(Number)
  const entryMap = new Map(monthEntries.map((item) => [item.date, item.reminderCount]))
  const firstDay = new Date(year, month - 1, 1)
  const firstWeekday = normalizeWeekday(firstDay.getDay())
  const daysInMonth = new Date(year, month, 0).getDate()
  const daysInPrevMonth = new Date(year, month - 1, 0).getDate()
  const cells: CalendarCell[] = []

  for (let index = firstWeekday - 1; index > 0; index -= 1) {
    const day = daysInPrevMonth - index + 1
    const date = formatDate(year, month - 1, day)
    cells.push({ date, day, inCurrentMonth: false, reminderCount: entryMap.get(date) ?? 0 })
  }

  for (let day = 1; day <= daysInMonth; day += 1) {
    const date = formatDate(year, month, day)
    cells.push({ date, day, inCurrentMonth: true, reminderCount: entryMap.get(date) ?? 0 })
  }

  while (cells.length < 42) {
    const day = cells.length - (firstWeekday - 1) - daysInMonth + 1
    const date = formatDate(year, month + 1, day)
    cells.push({ date, day, inCurrentMonth: false, reminderCount: entryMap.get(date) ?? 0 })
  }

  return cells
}

function normalizeWeekday(day: number) {
  return day === 0 ? 7 : day
}

function formatDate(year: number, month: number, day: number) {
  const date = new Date(year, month - 1, day)
  const normalizedYear = date.getFullYear()
  const normalizedMonth = String(date.getMonth() + 1).padStart(2, '0')
  const normalizedDay = String(date.getDate()).padStart(2, '0')

  return `${normalizedYear}-${normalizedMonth}-${normalizedDay}`
}

function formatMonthLabel(monthKey: string) {
  const [year, month] = monthKey.split('-')
  return `${year} 年 ${Number(month)} 月`
}

function monthKeyFromDate(date: string) {
  return date.split('-').slice(0, 2).join('-')
}
