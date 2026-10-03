#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    if std::env::args().any(|a| a == "--diagnose") {
        match guanqi_lib::diagnose() {
            Ok(report) => println!("{report}"),
            Err(error) => {eprintln!("{error}"); std::process::exit(1);}
        }
        return;
    }
    guanqi_lib::run()
}
