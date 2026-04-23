pub mod commands;
pub mod db;
pub mod models;
pub mod notification;
pub mod scheduler;
pub mod settings;
pub mod state;
pub mod tray;

#[cfg(test)]
mod tests {
    use crate::commands::app::greet;
    use crate::state::app_identity::app_name;

    #[test]
    fn exposes_expected_application_name() {
        assert_eq!(app_name(), "时间助手");
    }

    #[test]
    fn greets_from_the_backend_command() {
        assert_eq!(greet("开发者"), "你好，开发者。欢迎使用时间助手。");
    }
}
