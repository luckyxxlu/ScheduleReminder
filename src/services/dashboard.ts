import { invoke } from '@tauri-apps/api/core'

export type TodayDashboardData = {
  nextReminderTitle: string
  nextReminderTime: string
  nextReminderMessage: string
  highlightedStatus: string
}

export type CalendarEntry = {
  id: string
  date: string
  time: string
  title: string
  status: string
}

export type CalendarDaySummary = {
  date: string
  reminderCount: number
}

export type CalendarOverviewData = {
  selectedDate: string
  monthKey: string
  monthEntries: CalendarDaySummary[]
  entries: CalendarEntry[]
}

export async function getTodayDashboard(): Promise<TodayDashboardData> {
  return invoke<TodayDashboardData>('get_today_dashboard')
}

export async function markNextReminderCompleted(): Promise<TodayDashboardData> {
  return invoke<TodayDashboardData>('mark_next_reminder_completed')
}

export async function getCalendarOverview(selectedDate: string): Promise<CalendarOverviewData> {
  return invoke<CalendarOverviewData>('get_calendar_overview', { selected_date: selectedDate })
}

export async function createCalendarEvent(input: {
  title: string
  selectedDate: string
  time: string
}): Promise<CalendarOverviewData> {
  return invoke<CalendarOverviewData>('create_calendar_event', {
    title: input.title,
    selected_date: input.selectedDate,
    time: input.time,
  })
}
