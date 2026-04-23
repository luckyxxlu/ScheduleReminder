import { useEffect, useEffectEvent, useState } from 'react'
import { emit, listen, TauriEvent } from '@tauri-apps/api/event'
import { HashRouter, NavLink, Route, Routes } from 'react-router-dom'

import { todayDashboardRefreshEvent } from './events'
import { ReminderOverlay } from './ReminderOverlay'
import { CalendarPage } from '../pages/calendar/CalendarPage'
import { RemindersPage } from '../pages/reminders/RemindersPage'
import { SettingsPage } from '../pages/settings/SettingsPage'
import { TodayPage } from '../pages/today/TodayPage'
import {
  getTodayDashboard,
  graceNextReminderTenMinutes,
  markNextReminderCompleted,
  skipNextReminder,
  snoozeNextReminder,
  type TodayDashboardData,
} from '../services/dashboard'

const navItems = [
  { to: '/', label: '今天', end: true },
  { to: '/calendar', label: '日历' },
  { to: '/reminders', label: '提醒' },
  { to: '/settings', label: '设置' },
]

export function App() {
  const [triggeredReminder, setTriggeredReminder] = useState<{
    occurrenceId: string
    title: string
    message: string
    scheduledTime: string
    graceDeadline: string
  } | null>(null)
  const [overlayDashboard, setOverlayDashboard] = useState<TodayDashboardData | null>(null)
  const [isOverlaySubmitting, setIsOverlaySubmitting] = useState(false)

  const syncOverlayDashboard = useEffectEvent(async () => {
    try {
      const dashboard = await getTodayDashboard()
      setOverlayDashboard(dashboard)
      setTriggeredReminder((currentReminder) =>
        currentReminder && dashboard.nextReminderStatus === '宽容中' ? currentReminder : null,
      )
    } catch (error) {
      console.error('同步提醒状态失败', error)
    }
  })

  useEffect(() => {
    let mounted = true
    const unlistenFns: Array<() => void> = []

    void Promise.all([
      listen<{
        occurrenceId: string
        title: string
        message: string
        scheduledTime: string
        graceDeadline: string
      }>('reminder-triggered', async (event) => {
        setTriggeredReminder(event.payload)
        await syncOverlayDashboard()
        await emit(todayDashboardRefreshEvent, { source: 'reminder-triggered' })
      }),
      listen(todayDashboardRefreshEvent, async () => {
        await syncOverlayDashboard()
      }),
      listen(TauriEvent.WINDOW_FOCUS, async () => {
        await syncOverlayDashboard()
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
      unlistenFns.forEach((dispose) => dispose())
    }
  }, [])

  async function runOverlayAction(action: () => Promise<TodayDashboardData>) {
    setIsOverlaySubmitting(true)

    try {
      const nextDashboard = await action()
      setOverlayDashboard(nextDashboard)
      setTriggeredReminder(null)
      await emit(todayDashboardRefreshEvent, { source: 'overlay-action' })

    } finally {
      setIsOverlaySubmitting(false)
    }
  }

  return (
    <HashRouter>
      <div className="app-shell">
        <aside className="sidebar">
          <div className="brand-block">
            <p className="brand-eyebrow">时间助手</p>
            <h1 className="brand-title">桌面作息提醒</h1>
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
      <ReminderOverlay
        dashboard={overlayDashboard}
        isSubmitting={isOverlaySubmitting}
        reminder={triggeredReminder}
        onClose={() => setTriggeredReminder(null)}
        onComplete={() => void runOverlayAction(markNextReminderCompleted)}
        onGraceTenMinutes={() => void runOverlayAction(graceNextReminderTenMinutes)}
        onSkip={() => void runOverlayAction(skipNextReminder)}
        onSnooze={(minutes) => void runOverlayAction(() => snoozeNextReminder(minutes))}
      />
    </HashRouter>
  )
}
