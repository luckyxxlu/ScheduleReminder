import { invoke } from '@tauri-apps/api/core'

export type TodayDashboardData = {
  activeReminderId: string
  nextReminderTitle: string
  nextReminderTime: string
  nextReminderMessage: string
  nextReminderStatus: string
  nextReminderNotificationState: string
  nextReminderGraceDeadline: string | null
  nextReminderAvailableActions: string[]
  highlightedStatus: string
  todayTimeline: TodayTimelineItem[]
  recentActions: TodayActionItem[]
}

export type TodayTimelineItem = {
  id: string
  time: string
  title: string
  message: string
  status: string
  isActive: boolean
}

export type TodayActionItem = {
  id: string
  actionLabel: string
  actionAt: string
}

export type CalendarEntry = {
  id: string
  date: string
  time: string
  title: string
  message: string
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
  recentActions: TodayActionItem[]
}

export async function getTodayDashboard(): Promise<TodayDashboardData> {
  return invoke<TodayDashboardData>('get_today_dashboard')
}

export async function markNextReminderCompleted(): Promise<TodayDashboardData> {
  return invoke<TodayDashboardData>('mark_next_reminder_completed')
}

export async function graceNextReminderTenMinutes(): Promise<TodayDashboardData> {
  return invoke<TodayDashboardData>('grace_next_reminder_ten_minutes')
}

export async function snoozeNextReminder(minutes: number): Promise<TodayDashboardData> {
  return invoke<TodayDashboardData>('snooze_next_reminder', { minutes })
}

export async function skipNextReminder(): Promise<TodayDashboardData> {
  return invoke<TodayDashboardData>('skip_next_reminder')
}

export async function getCalendarOverview(selectedDate: string): Promise<CalendarOverviewData> {
  return invoke<CalendarOverviewData>('get_calendar_overview', { selected_date: selectedDate })
}

export async function createCalendarEvent(input: {
  title: string
  message: string
  selectedDate: string
  time: string
}): Promise<CalendarOverviewData> {
  return invoke<CalendarOverviewData>('create_calendar_event', {
    title: input.title,
    message: input.message,
    selected_date: input.selectedDate,
    time: input.time,
  })
}
