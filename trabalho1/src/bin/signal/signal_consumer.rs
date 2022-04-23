use nix::libc;
use nix::sys::signal::{self, SaFlags, SigAction, SigHandler, SigSet};

fn main() {
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
    loop {}
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
