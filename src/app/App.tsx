import { HashRouter, NavLink, Route, Routes } from 'react-router-dom'

import { CalendarPage } from '../pages/calendar/CalendarPage'
import { RemindersPage } from '../pages/reminders/RemindersPage'
import { SettingsPage } from '../pages/settings/SettingsPage'
import { TodayPage } from '../pages/today/TodayPage'

const navItems = [
  { to: '/', label: '今天', end: true },
  { to: '/calendar', label: '日历' },
  { to: '/reminders', label: '提醒' },
  { to: '/settings', label: '设置' },
]

export function App() {
  return (
    <HashRouter>
      <div className="app-shell">
        <aside className="sidebar">
          <div className="brand-block">
            <p className="brand-eyebrow">ScheduleReminder</p>
            <h1 className="brand-title">桌面时间提醒器</h1>
          </div>

          <nav aria-label="主导航" className="main-nav">
            {navItems.map((item) => (
              <NavLink
                key={item.to}
                className={({ isActive }) =>
                  isActive ? 'nav-link nav-link-active' : 'nav-link'
                }
                end={item.end}
                to={item.to}
              >
                {item.label}
              </NavLink>
            ))}
          </nav>
        </aside>

        <main className="page-container">
          <Routes>
            <Route element={<TodayPage />} path="/" />
            <Route element={<CalendarPage />} path="/calendar" />
            <Route element={<RemindersPage />} path="/reminders" />
            <Route element={<SettingsPage />} path="/settings" />
          </Routes>
        </main>
      </div>
    </HashRouter>
  )
}
