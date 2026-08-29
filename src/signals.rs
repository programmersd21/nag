use nix::sys::signal::{self, SigAction, SigHandler, SigSet, Signal};
use nix::unistd::Pid;
use std::sync::atomic::{AtomicI32, Ordering};

static FORWARD_PGID: AtomicI32 = AtomicI32::new(0);

pub fn setup_signal_handlers(pgid: i32) {
    FORWARD_PGID.store(pgid, Ordering::SeqCst);

    let sa = SigAction::new(
        SigHandler::Handler(handle_signal),
        signal::SaFlags::SA_RESTART,
        SigSet::empty(),
    );

    unsafe {
        let _ = signal::sigaction(Signal::SIGINT, &sa);
        let _ = signal::sigaction(Signal::SIGTERM, &sa);
        let _ = signal::sigaction(Signal::SIGHUP, &sa);
        let _ = signal::sigaction(Signal::SIGQUIT, &sa);
    }
}

pub fn restore_signal_handlers() {
    FORWARD_PGID.store(0, Ordering::SeqCst);

    let sa = SigAction::new(
        SigHandler::SigDfl,
        signal::SaFlags::empty(),
        SigSet::empty(),
    );

    unsafe {
        let _ = signal::sigaction(Signal::SIGINT, &sa);
        let _ = signal::sigaction(Signal::SIGTERM, &sa);
        let _ = signal::sigaction(Signal::SIGHUP, &sa);
        let _ = signal::sigaction(Signal::SIGQUIT, &sa);
    }
}

extern "C" fn handle_signal(sig: i32) {
    let pgid = FORWARD_PGID.load(Ordering::SeqCst);
    if pgid > 0 {
        if let Ok(signal) = Signal::try_from(sig) {
            // Forward signal to the child process group
            let _ = signal::kill(Pid::from_raw(-pgid), signal);
        }
    }
}

pub fn map_signal_to_exit_code(sig: i32) -> i32 {
    128 + sig
}
