use std::process::Command;

pub fn issue_bg_update(path: String) {
    let output = Command::new("swww")
        .args(["img", path.as_str()])
        .output()
        .expect("Failed to execute command");
    println!("Set BG status update: {}", output.status);
    println!("Output: {}", String::from_utf8_lossy(&output.stdout));
    println!("Error: {}", String::from_utf8_lossy(&output.stderr));
}
