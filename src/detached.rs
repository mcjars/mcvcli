use ipipe::Pipe;

pub fn status(pid: Option<usize>) -> bool {
    if pid.is_none() {
        return false;
    }

    let pid = sysinfo::Pid::from(pid.unwrap());
    let sys = sysinfo::System::new_all();
    let process = match sys.process(pid) {
        Some(proc) => proc,
        None => return false,
    };

    process
        .exe()
        .and_then(|exe| exe.to_str())
        .is_some_and(|s| s.contains("java"))
}

pub fn get_pipes(identifier: &str) -> [Pipe; 3] {
    let stdin = Pipe::with_name(&format!("{identifier}_stdin")).unwrap();
    let stdout = Pipe::with_name(&format!("{identifier}_stdout")).unwrap();
    let stderr = Pipe::with_name(&format!("{identifier}_stderr")).unwrap();

    [stdin, stdout, stderr]
}
