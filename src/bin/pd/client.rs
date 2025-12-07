use anyhow::Result;
use parallel_downloader::ipc::{Command, Response};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn send_command_raw(cmd: Command) -> Result<Response> {
    let mut stream = TcpStream::connect("127.0.0.1:9090".to_string())
        .await
        .map_err(|_| anyhow::anyhow!("Could not connect to daemon. Is it running?"))?;

    let json_req = serde_json::to_string(&cmd)?;
    stream.write_all(json_req.as_bytes()).await?;

    let mut buf = [0; 1024];
    let n = stream.read(&mut buf).await?;
    let json_resp = String::from_utf8_lossy(&buf[..n]);

    let response: Response = serde_json::from_str(&json_resp)?;

    Ok(response)
}
