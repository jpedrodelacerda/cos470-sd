use std::env;

use nix::sys::signal::{self, SaFlags, SigAction, SigHandler, SigSet};
use nix::unistd::Pid;
use nix::{libc, unistd};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("No waiting method provided. Choose BUSY or BLOCKING.");
        std::process::exit(1);
    }
    let handlers = &[
        (SigHandler::Handler(sigint_handler), signal::SIGINT),
        (SigHandler::Handler(sigquit_handler), signal::SIGQUIT),
        (SigHandler::Handler(sigterm_handler), signal::SIGTERM),
    ];
    for (sig_handler, signal) in handlers {
        let sa = SigAction::new(*sig_handler, SaFlags::empty(), SigSet::empty());
        unsafe {
            let _ = signal::sigaction(*signal, &sa);
        }
    }

    println!("Send signals to PID {}.", Pid::this());

    match args[1].as_str() {
        "BUSY" => busy_wait(),
        "BLOCKING" => blocking_wait(),
        _ => {
            println!("Please choose BUSY or BLOCKING.");
            std::process::exit(1);
        }
    }
}

fn busy_wait() {
    println!("Running BUSY WAIT");
    loop {}
}

fn blocking_wait() {
    println!("Running BLOCKING WAIT");
    loop {
        unistd::pause();
    }
}

extern "C" fn sigint_handler(sig: libc::c_int) {
    println!("I received a SIGINT ({}). Should I finish?", sig)
}

extern "C" fn sigquit_handler(sig: libc::c_int) {
    println!("I received SIGQUIT ({}). Should I quit?", sig)
}

extern "C" fn sigterm_handler(sig: libc::c_int) {
    println!("I received SIGTERM ({}). Bye bye!", sig);
    std::process::exit(0)
}
