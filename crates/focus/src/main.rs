mod appkiller;

fn main() {
    let process = "Spotify";
    appkiller::kill_process(process);
}
