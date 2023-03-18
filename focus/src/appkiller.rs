use std::process::Command;

pub fn kill_process(process: &str) {
    let mut command = Command::new("pkill");
    command.arg("-QUIT").arg("-x").arg(process);
    let output = command.output().expect("Failed to kill process: {process}");

    if output.status.success() {
        println!("{process} killed!");
    } else {
        println!("{:?}", command);
        println!("Status: {}", output.status);
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
}
