use clap::Parser;
use std::process::{Command, Stdio};
#[cfg(windows)]
use std::os::windows::process::CommandExt;

mod deamon;
mod cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = cli::Args::parse();
    match args.cmd {
        cli::Cmd::Run(opts) => {
            if opts.foreground {
                deamon::run().await?
            } else {
                let exe = std::env::current_exe()?;
                let mut cmd = Command::new(exe);
                cmd.arg("run")
                    .arg("--foreground")
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null());
                #[cfg(windows)]
                {
                    const DETACHED_PROCESS: u32 = 0x00000008;
                    const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
                    cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
                }
                let _ = cmd.spawn()?;
            }
        }
        cli::Cmd::Status => {
            cli::exec(cli::Cmd::Status).await?
        }
        cli::Cmd::Stop => {
            cli::exec(cli::Cmd::Stop).await?
        }
    }
    Ok(())
}