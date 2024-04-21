const COMMANDS: &[&str] = &["event", "breadcrumb"];

fn main() {
  tauri_plugin::Builder::new(COMMANDS).build();
}