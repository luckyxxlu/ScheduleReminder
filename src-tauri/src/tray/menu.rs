#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayMenuItem {
    pub id: String,
    pub label: String,
}

pub fn build_tray_menu() -> Vec<TrayMenuItem> {
    vec![
        TrayMenuItem {
            id: "open_today".to_string(),
            label: "打开今天页".to_string(),
        },
        TrayMenuItem {
            id: "quick_create".to_string(),
            label: "快速新建提醒".to_string(),
        },
        TrayMenuItem {
            id: "pause_one_hour".to_string(),
            label: "暂停全部提醒 1 小时".to_string(),
        },
        TrayMenuItem {
            id: "open_settings".to_string(),
            label: "打开设置".to_string(),
        },
        TrayMenuItem {
            id: "exit_app".to_string(),
            label: "退出应用".to_string(),
        },
    ]
}

pub fn handle_close_window(minimize_to_tray: bool) -> &'static str {
    if minimize_to_tray {
        "minimize_to_tray"
    } else {
        "exit_app"
    }
}

#[cfg(test)]
mod tests {
    use super::{build_tray_menu, handle_close_window};

    #[test]
    fn builds_expected_tray_menu_actions() {
        let menu = build_tray_menu();

        assert_eq!(menu.len(), 5);
        assert_eq!(menu[0].label, "打开今天页");
        assert_eq!(menu[2].label, "暂停全部提醒 1 小时");
    }

    #[test]
    fn closes_window_to_tray_when_enabled() {
        assert_eq!(handle_close_window(true), "minimize_to_tray");
    }

    #[test]
    fn exits_app_when_minimize_to_tray_is_disabled() {
        assert_eq!(handle_close_window(false), "exit_app");
    }
}
