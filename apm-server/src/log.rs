use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
};
use std::convert::Infallible;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::AppState;

pub async fn stream_handler(State(state): State<Arc<AppState>>) -> Response {
    let log_path: PathBuf = match &state.log_file {
        Some(p) => p.clone(),
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    if !log_path.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let content = match std::fs::read_to_string(&log_path) {
        Ok(c) => c,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    let all_lines: Vec<&str> = content.lines().collect();
    let start = all_lines.len().saturating_sub(100);
    let initial: Vec<String> = all_lines[start..].iter().map(|l| l.to_string()).collect();
    let mut offset = content.len() as u64;

    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(256);

    for line in initial {
        if tx.send(Ok(Event::default().data(line))).await.is_err() {
            return Sse::new(ReceiverStream::new(rx)).into_response();
        }
    }

    let log_path_clone = log_path.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(250)).await;

            if tx.is_closed() {
                break;
            }

            let new_len = match std::fs::metadata(&log_path_clone) {
                Ok(m) => m.len(),
                Err(_) => continue,
            };

            if new_len <= offset {
                continue;
            }

            let mut file = match std::fs::File::open(&log_path_clone) {
                Ok(f) => f,
                Err(_) => continue,
            };
            if file.seek(SeekFrom::Start(offset)).is_err() {
                continue;
            }
            let mut buf = String::new();
            if file.read_to_string(&mut buf).is_err() {
                continue;
            }
            offset = new_len;

            for line in buf.lines() {
                if line.is_empty() {
                    continue;
                }
                if tx.send(Ok(Event::default().data(line.to_string()))).await.is_err() {
                    return;
                }
            }
        }
    });

    Sse::new(ReceiverStream::new(rx))
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keepalive"),
        )
        .into_response()
}
