const COMMANDS: &[&str] = &["envelope", "breadcrumb"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
