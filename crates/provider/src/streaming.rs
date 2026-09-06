use tokio_stream::StreamExt;

/// 流式处理 SSE 响应
pub async fn process_stream(
    response: reqwest::Response,
) -> anyhow::Result<tokio::sync::mpsc::Receiver<String>> {
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(100);

    tokio::spawn(async move {
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    for line in text.lines() {
                        if line.starts_with("data: ") {
                            let _ = tx.send(line.to_string()).await;
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });

    Ok(rx)
}
