use ipipe::Pipe;

pub fn status(pid: Option<usize>) -> bool {
    if pid.is_none() {
        return false;
    }

    let pid = sysinfo::Pid::from(pid.unwrap());
    let sys = sysinfo::System::new_all();
    let process = sys.process(pid);

    if process.is_none() {
        return false;
    }

    let process = process.unwrap();
    for value in process.environ() {
        if value.to_str().unwrap().contains("/.mcvcli/java/") {
            return true;
        }
    }

    false
}

pub fn get_pipes(identifier: &str) -> (Pipe, Pipe, Pipe) {
    let stdin = Pipe::with_name(&format!("{}_stdin", identifier)).unwrap();
    let stdout = Pipe::with_name(&format!("{}_stdout", identifier)).unwrap();
    let stderr = Pipe::with_name(&format!("{}_stderr", identifier)).unwrap();

    (stdin, stdout, stderr)
}
