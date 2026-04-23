import { useEffect } from 'react'

import { type TodayDashboardData } from '../services/dashboard'

type ReminderOverlayProps = {
  reminder: {
    occurrenceId: string
    title: string
    message: string
    scheduledTime: string
    graceDeadline: string
  } | null
  isSubmitting: boolean
  onClose: () => void
  onComplete: () => void
  onGraceTenMinutes: () => void
  onSnooze: (minutes: number) => void
  onSkip: () => void
  dashboard: TodayDashboardData | null
}

const snoozeOptions = [5, 10, 15, 30] as const

export function ReminderOverlay(props: ReminderOverlayProps) {
  useEffect(() => {
    if (!props.reminder) {
      return
    }

    function handleKeydown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        props.onClose()
      }
    }

    window.addEventListener('keydown', handleKeydown)

    return () => {
      window.removeEventListener('keydown', handleKeydown)
    }
  }, [props.onClose, props.reminder])

  if (!props.reminder) {
    return null
  }

  return (
    <div className="reminder-overlay-backdrop" role="presentation">
      <section aria-label="触发提醒" className="reminder-overlay-panel" style={{ maxWidth: 480, textAlign: 'center', padding: '48px 32px' }}>
        <div style={{ marginBottom: 32 }}>
          <span className="panel-label" style={{ fontSize: 14 }}>{props.reminder.scheduledTime} 提醒</span>
          <strong className="panel-title" style={{ fontSize: 32, marginBottom: 16 }}>{props.reminder.title}</strong>
          <p className="panel-text" style={{ fontSize: 16 }}>{props.reminder.message}</p>
          <p className="panel-text" style={{ fontSize: 14, marginTop: 12 }}>宽容截止至：{props.reminder.graceDeadline}</p>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          <button 
            className="primary-button" 
            disabled={props.isSubmitting} 
            type="button" 
            onClick={props.onComplete}
            style={{ height: 56, fontSize: 16, borderRadius: 100 }}
          >
            完成
          </button>
          
          <button 
            className="secondary-button" 
            disabled={props.isSubmitting} 
            type="button" 
            onClick={props.onGraceTenMinutes}
            style={{ height: 56, fontSize: 16, borderRadius: 100, border: '1px solid rgba(199, 110, 54, 0.3)', color: 'var(--accent-orange)' }}
          >
            宽容 10 分钟
          </button>

          <div className="action-row" style={{ justifyContent: 'center', flexWrap: 'wrap', marginTop: 4 }}>
            {snoozeOptions.map((minutes) => (
              <button
                key={minutes}
                className="secondary-button"
                disabled={props.isSubmitting}
                type="button"
                onClick={() => props.onSnooze(minutes)}
              >
                稍后 {minutes} 分钟
              </button>
            ))}
          </div>
        </div>
        
        <div style={{ display: 'flex', justifyContent: 'center', gap: 24, marginTop: 32 }}>
          <button 
            className="secondary-button" 
            disabled={props.isSubmitting} 
            type="button" 
            onClick={props.onSkip}
            style={{ background: 'transparent', border: 'none', boxShadow: 'none', color: 'var(--text-secondary)' }}
          >
            跳过今天
          </button>
          <button 
            className="secondary-button" 
            disabled={props.isSubmitting} 
            type="button" 
            onClick={props.onClose}
            style={{ background: 'transparent', border: 'none', boxShadow: 'none', color: 'var(--text-secondary)' }}
          >
            关闭窗口
          </button>
        </div>
      </section>
    </div>
  )
}
