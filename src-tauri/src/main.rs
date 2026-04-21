#[tauri::command]
fn greet(name: &str) -> String {
    schedule_reminder::commands::app::greet(name)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("failed to run ScheduleReminder")
}
