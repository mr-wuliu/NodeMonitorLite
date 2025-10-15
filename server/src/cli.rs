use clap::{Parser, Subcommand, Args as ClapArgs};
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};

#[derive(Parser, Debug)]
#[command(name = "nml", about = "Node Monitor Lite")] 
pub struct Args {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    Run(RunOpts),
    Status,
    Stop,
}

#[derive(ClapArgs, Debug, Default, Clone, Copy)]
pub struct RunOpts {
    #[arg(long, help = "在前台运行（不分离）")]
    pub foreground: bool,
}

pub async fn exec(cmd: Cmd) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match cmd {
        Cmd::Run(_opts) => {
            // 在 main 中处理，避免循环调用
            Ok(())
        }
        Cmd::Status => {
            let mut stream = TcpStream::connect("127.0.0.1:5052").await?;
            stream.write_all(b"status\n").await?;
            let (reader, _) = stream.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            if reader.read_line(&mut line).await? > 0 {
                print!("{}", line);
            }
            Ok(())
        }
        Cmd::Stop => {
            let mut stream = TcpStream::connect("127.0.0.1:5052").await?;
            stream.write_all(b"stop\n").await?;
            let (reader, _) = stream.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            if reader.read_line(&mut line).await? > 0 {
                print!("{}", line);
            }
            Ok(())
        }
    }
}


