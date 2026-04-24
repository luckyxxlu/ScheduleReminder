#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSettings {
    pub default_grace_minutes: i32,
    pub startup_with_windows: bool,
    pub tray_enabled: bool,
    pub close_to_tray_on_close: bool,
    pub theme: String,
    pub quiet_hours_enabled: bool,
    pub quiet_hours_start: Option<String>,
    pub quiet_hours_end: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsError {
    InvalidGraceMinutes,
    IncompleteQuietHours,
}

pub fn validate_settings(settings: &AppSettings) -> Result<(), SettingsError> {
    if settings.default_grace_minutes < 0 {
        return Err(SettingsError::InvalidGraceMinutes);
    }

    if settings.quiet_hours_enabled {
        let start = settings.quiet_hours_start.as_ref();
        let end = settings.quiet_hours_end.as_ref();
        if start.is_none() || end.is_none() {
            return Err(SettingsError::IncompleteQuietHours);
        }
    }

    Ok(())
}

pub fn set_launch_on_startup(settings: &mut AppSettings, enabled: bool) {
    settings.startup_with_windows = enabled;
}

pub fn quiet_hours_active(settings: &AppSettings, current_time: &str) -> bool {
    if !settings.quiet_hours_enabled {
        return false;
    }

    let (Some(start), Some(end)) = (
        settings.quiet_hours_start.as_ref(),
        settings.quiet_hours_end.as_ref(),
    ) else {
        return false;
    };

    let start = start.as_str();
    let end = end.as_str();

    if start <= end {
        current_time >= start && current_time <= end
    } else {
        current_time >= start || current_time <= end
    }
}

#[cfg(test)]
mod tests {
    use super::{
        quiet_hours_active, set_launch_on_startup, validate_settings, AppSettings, SettingsError,
    };

    fn sample_settings() -> AppSettings {
        AppSettings {
            default_grace_minutes: 10,
            startup_with_windows: false,
            tray_enabled: true,
            close_to_tray_on_close: true,
            theme: "system".to_string(),
            quiet_hours_enabled: false,
            quiet_hours_start: None,
            quiet_hours_end: None,
        }
    }

    #[test]
    fn validates_startup_setting_toggle() {
        let mut settings = sample_settings();

        set_launch_on_startup(&mut settings, true);

        assert!(settings.startup_with_windows);
    }

    #[test]
    fn validates_quiet_hours_when_fully_configured() {
        let mut settings = sample_settings();
        settings.quiet_hours_enabled = true;
        settings.quiet_hours_start = Some("22:00".to_string());
        settings.quiet_hours_end = Some("07:00".to_string());

        let result = validate_settings(&settings);

        assert!(result.is_ok());
    }

    #[test]
    fn rejects_incomplete_quiet_hours_configuration() {
        let mut settings = sample_settings();
        settings.quiet_hours_enabled = true;
        settings.quiet_hours_start = Some("22:00".to_string());

        let result = validate_settings(&settings);

        assert_eq!(result, Err(SettingsError::IncompleteQuietHours));
    }

    #[test]
    fn detects_quiet_hours_when_current_time_is_in_range() {
        let mut settings = sample_settings();
        settings.quiet_hours_enabled = true;
        settings.quiet_hours_start = Some("22:00".to_string());
        settings.quiet_hours_end = Some("07:00".to_string());

        assert!(quiet_hours_active(&settings, "23:00"));
        assert!(quiet_hours_active(&settings, "06:30"));
        assert!(!quiet_hours_active(&settings, "12:00"));
    }
}
