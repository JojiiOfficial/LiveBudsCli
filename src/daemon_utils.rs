use std::{
    env,
    path::Path,
    process::{exit, Command, Stdio},
};

use nix::{
    sys::signal::{self, SIGTERM},
    unistd::Pid,
};

/// Start the daemon detached from the current cli
pub fn start() -> bool {
    let curr_exe = env::current_exe().expect("Couldn't get current executable!");
    let mut cmd = Command::new("nohup");
    let cmd = cmd.arg(curr_exe).arg("-d").arg("--no-fork").arg("-q");
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    let status = cmd.spawn();
    status.is_ok()
}

/// Returns an error with a human friendly message if a daemon is already running
pub fn check_running<P: AsRef<Path>>(p: P) -> Result<(), String> {
    let p = p.as_ref();

    if !p.exists() {
        return Ok(());
    }

    // Check if the socket file is used by a running program
    if let Ok(files) = ofiles::opath(&p) {
        if files.is_empty() {
            // Cleanup old socket file
            try_delete_socket(p)?;
        }

        return Err(format!(
            "A daemon is already running: {}",
            files
                .into_iter()
                .map(|i| format!("{:?} ", i))
                .collect::<String>()
        ));
    }

    // Cleanup old socket file
    try_delete_socket(p)?;

    Ok(())
}

/// Try to delete the socket file
pub fn try_delete_socket<P: AsRef<Path>>(p: P) -> Result<(), String> {
    std::fs::remove_file(p.as_ref()).map_err(|e| {
        format!(
            "Can't delete old socket file {}: {:?}",
            p.as_ref().display(),
            e
        )
    })?;
    Ok(())
}

// Kill a daemon
pub fn kill<P: AsRef<Path>>(quiet: bool, daemon_path: P) -> bool {
    let daemon_path = daemon_path.as_ref();

    let pids = ofiles::opath(daemon_path);
    println!("pids: {pids:?}");
    if let Ok(pids) = pids {
        let u: u32 = (*pids.get(0).unwrap()).into();
        if let Err(err) = signal::kill(Pid::from_raw(u as i32), SIGTERM) {
            eprintln!("Error killing process: {:?}", err);
            exit(1);
        } else if !quiet {
            println!("Daemon exited!");
        }

        // Hacky way not to display annoying cargo warnings
        try_delete_socket(daemon_path).unwrap();
        return true;
    }

    false
}
