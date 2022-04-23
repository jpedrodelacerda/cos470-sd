use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::convert::TryFrom;
use std::env;
use std::process::exit;
use sysinfo::{Pid as SysInfoPid, System, SystemExt};

fn main() {
    let (signal, pid) = receive_instructions();

    println!("Sending {} to pid {}", signal, pid);
    let _ = kill(pid, signal).ok();
}

fn receive_instructions() -> (Signal, Pid) {
    let args = env::args().collect::<Vec<_>>();
    let signal = Signal::try_from(args[1].parse::<i32>().expect("failed to parse signal"))
        .expect("Invalid signal");
    let pid = Pid::from_raw(args[2].parse::<i32>().unwrap());
    if !pid_exists(pid.as_raw()) {
        println!("PID {} does not exist", pid);
        exit(1);
    }

    (signal, pid)
}

fn pid_exists(pid: i32) -> bool {
    let sys_pid = SysInfoPid::from(pid);
    let s = System::new_all();
    let procs = s.processes();
    procs.contains_key(&sys_pid)
}
