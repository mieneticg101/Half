//! WebSocket support for real-time bidirectional communication
//!
//! Provides high-performance WebSocket connections with clean API.

use crate::error::{Error, Result};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::protocol::Message};

/// WebSocket connection handle
///
/// Provides a safe, ergonomic API for WebSocket communication.
///
/// # Features
/// - Send and receive text and binary messages
/// - Automatic ping/pong handling
/// - Clean connection closure
/// - Thread-safe message sending
///
/// # Example
/// ```no_run
/// use half_core::websocket::WebSocket;
///
/// async fn handle_websocket(ws: WebSocket) {
///     while let Some(msg) = ws.receive().await {
///         match msg {
///             Ok(text) => {
///                 println!("Received: {}", text);
///                 ws.send_text("Echo: ".to_string() + &text).await.ok();
///             }
///             Err(e) => {
///                 eprintln!("Error: {}", e);
///                 break;
///             }
///         }
///     }
/// }
/// ```
pub struct WebSocket {
    sender: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
    receiver: Arc<Mutex<SplitStream<WebSocketStream<TcpStream>>>>,
}

impl WebSocket {
    /// Create a new WebSocket from a TCP stream
    ///
    /// This performs the WebSocket handshake and returns a WebSocket handle.
    pub async fn from_tcp_stream(stream: TcpStream) -> Result<Self> {
        let ws_stream = accept_async(stream)
            .await
            .map_err(|e| Error::InternalError(format!("WebSocket handshake failed: {}", e)))?;

        let (sender, receiver) = ws_stream.split();

        Ok(Self {
            sender: Arc::new(Mutex::new(sender)),
            receiver: Arc::new(Mutex::new(receiver)),
        })
    }

    /// Send a text message
    pub async fn send_text(&self, text: impl Into<String>) -> Result<()> {
        let mut sender = self.sender.lock().await;
        sender
            .send(Message::Text(text.into().into()))
            .await
            .map_err(|e| Error::InternalError(format!("Failed to send message: {}", e)))?;
        Ok(())
    }

    /// Send a binary message
    pub async fn send_binary(&self, data: impl Into<Vec<u8>>) -> Result<()> {
        let mut sender = self.sender.lock().await;
        sender
            .send(Message::Binary(data.into().into()))
            .await
            .map_err(|e| Error::InternalError(format!("Failed to send message: {}", e)))?;
        Ok(())
    }

    /// Send a ping message
    pub async fn send_ping(&self, data: Vec<u8>) -> Result<()> {
        let mut sender = self.sender.lock().await;
        sender
            .send(Message::Ping(data.into()))
            .await
            .map_err(|e| Error::InternalError(format!("Failed to send ping: {}", e)))?;
        Ok(())
    }

    /// Receive the next message
    ///
    /// Returns `None` when the connection is closed.
    /// Automatically handles ping/pong messages.
    pub async fn receive(&self) -> Option<Result<String>> {
        let mut receiver = self.receiver.lock().await;

        loop {
            match receiver.next().await {
                Some(Ok(msg)) => match msg {
                    Message::Text(text) => return Some(Ok(text.to_string())),
                    Message::Binary(data) => {
                        // Convert binary to string (UTF-8)
                        match String::from_utf8(data.to_vec()) {
                            Ok(text) => return Some(Ok(text)),
                            Err(e) => {
                                return Some(Err(Error::InternalError(format!(
                                    "Invalid UTF-8 in binary message: {}",
                                    e
                                ))));
                            }
                        }
                    }
                    Message::Ping(data) => {
                        // Automatically respond to ping with pong
                        if let Err(e) = self.send_pong(data.to_vec()).await {
                            return Some(Err(e));
                        }
                        // Continue to next message
                        continue;
                    }
                    Message::Pong(_) => {
                        // Ignore pong messages
                        continue;
                    }
                    Message::Close(_) => return None,
                    Message::Frame(_) => {
                        // Ignore raw frames
                        continue;
                    }
                },
                Some(Err(e)) => {
                    return Some(Err(Error::InternalError(format!("WebSocket error: {}", e))));
                }
                None => return None,
            }
        }
    }

    /// Receive the next message as binary
    pub async fn receive_binary(&self) -> Option<Result<Vec<u8>>> {
        let mut receiver = self.receiver.lock().await;

        loop {
            match receiver.next().await {
                Some(Ok(msg)) => match msg {
                    Message::Binary(data) => return Some(Ok(data.to_vec())),
                    Message::Text(text) => return Some(Ok(text.as_bytes().to_vec())),
                    Message::Ping(data) => {
                        // Automatically respond to ping with pong
                        if let Err(e) = self.send_pong(data.to_vec()).await {
                            return Some(Err(e));
                        }
                        continue;
                    }
                    Message::Pong(_) => continue,
                    Message::Close(_) => return None,
                    Message::Frame(_) => continue,
                },
                Some(Err(e)) => {
                    return Some(Err(Error::InternalError(format!("WebSocket error: {}", e))));
                }
                None => return None,
            }
        }
    }

    /// Send a pong message (usually in response to ping)
    async fn send_pong(&self, data: Vec<u8>) -> Result<()> {
        let mut sender = self.sender.lock().await;
        sender
            .send(Message::Pong(data.into()))
            .await
            .map_err(|e| Error::InternalError(format!("Failed to send pong: {}", e)))?;
        Ok(())
    }

    /// Close the WebSocket connection gracefully
    pub async fn close(&self) -> Result<()> {
        let mut sender = self.sender.lock().await;
        sender
            .send(Message::Close(None))
            .await
            .map_err(|e| Error::InternalError(format!("Failed to close connection: {}", e)))?;
        Ok(())
    }

    /// Clone the sender for use in multiple tasks
    ///
    /// This allows you to send messages from multiple async tasks.
    pub fn clone_sender(&self) -> Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>> {
        Arc::clone(&self.sender)
    }
}

/// WebSocket message type for easier handling
#[derive(Debug, Clone)]
pub enum WsMessage {
    /// Text message
    Text(String),
    /// Binary message
    Binary(Vec<u8>),
    /// Close message
    Close,
}

impl From<Message> for WsMessage {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Text(text) => WsMessage::Text(text.to_string()),
            Message::Binary(data) => WsMessage::Binary(data.to_vec()),
            Message::Close(_) => WsMessage::Close,
            _ => WsMessage::Close, // Treat ping/pong/frame as close
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_from_text() {
        let msg = Message::Text("Hello".to_string().into());
        let ws_msg = WsMessage::from(msg);
        match ws_msg {
            WsMessage::Text(text) => assert_eq!(text, "Hello"),
            _ => panic!("Expected Text message"),
        }
    }

    #[test]
    fn test_ws_message_from_binary() {
        let msg = Message::Binary(vec![1, 2, 3].into());
        let ws_msg = WsMessage::from(msg);
        match ws_msg {
            WsMessage::Binary(data) => assert_eq!(data, vec![1, 2, 3]),
            _ => panic!("Expected Binary message"),
        }
    }

    #[test]
    fn test_ws_message_from_close() {
        let msg = Message::Close(None);
        let ws_msg = WsMessage::from(msg);
        match ws_msg {
            WsMessage::Close => {}
            _ => panic!("Expected Close message"),
        }
    }
}
