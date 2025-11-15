//! Server-Sent Events (SSE) support for real-time server-to-client streaming
//!
//! SSE provides one-way communication from server to client over HTTP.

use crate::{Response, Result};
use bytes::Bytes;
use futures_util::stream::Stream;
use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Interval;

/// Server-Sent Event
///
/// Represents a single SSE event that can be sent to the client.
///
/// # Format
/// SSE messages follow this format:
/// ```text
/// event: event_name
/// data: event_data
/// id: event_id
/// retry: 5000
/// ```
#[derive(Debug, Clone)]
pub struct SseEvent {
    /// Event name (optional)
    pub event: Option<String>,
    /// Event data (required)
    pub data: String,
    /// Event ID (optional, for client reconnection)
    pub id: Option<String>,
    /// Retry interval in milliseconds (optional)
    pub retry: Option<u64>,
    /// Comment (optional, keeps connection alive)
    pub comment: Option<String>,
}

impl SseEvent {
    /// Create a new SSE event with data
    pub fn new(data: impl Into<String>) -> Self {
        Self {
            event: None,
            data: data.into(),
            id: None,
            retry: None,
            comment: None,
        }
    }

    /// Set the event name
    pub fn event(mut self, name: impl Into<String>) -> Self {
        self.event = Some(name.into());
        self
    }

    /// Set the event ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the retry interval in milliseconds
    pub fn retry(mut self, milliseconds: u64) -> Self {
        self.retry = Some(milliseconds);
        self
    }

    /// Set a comment (for keeping connection alive)
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Convert the event to SSE format
    pub fn to_sse_format(&self) -> String {
        let mut output = String::new();

        if let Some(ref comment) = self.comment {
            output.push_str(&format!(": {}\n", comment));
        }

        if let Some(ref event) = self.event {
            output.push_str(&format!("event: {}\n", event));
        }

        if let Some(ref id) = self.id {
            output.push_str(&format!("id: {}\n", id));
        }

        if let Some(retry) = self.retry {
            output.push_str(&format!("retry: {}\n", retry));
        }

        // Data can be multi-line
        for line in self.data.lines() {
            output.push_str(&format!("data: {}\n", line));
        }

        output.push('\n');
        output
    }
}

impl fmt::Display for SseEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_sse_format())
    }
}

/// SSE Channel for sending events
///
/// Provides a channel for sending Server-Sent Events to connected clients.
///
/// # Example
/// ```no_run
/// use half_core::sse::{SseEvent, SseChannel};
///
/// async fn sse_handler() -> half_core::Response {
///     let (channel, stream) = SseChannel::new();
///
///     // Spawn task to send events
///     tokio::spawn(async move {
///         let mut counter = 0;
///         loop {
///             counter += 1;
///             let event = SseEvent::new(format!("Count: {}", counter))
///                 .event("counter")
///                 .id(counter.to_string());
///
///             if channel.send(event).await.is_err() {
///                 break;
///             }
///
///             tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
///         }
///     });
///
///     // Return SSE response
///     stream.into_response()
/// }
/// ```
pub struct SseChannel {
    sender: mpsc::UnboundedSender<SseEvent>,
}

impl SseChannel {
    /// Create a new SSE channel
    ///
    /// Returns a tuple of (SseChannel, SseStream).
    /// Use the channel to send events and convert the stream to a response.
    pub fn new() -> (Self, SseStream) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self { sender: tx },
            SseStream {
                receiver: rx,
                keep_alive: None,
            },
        )
    }

    /// Send an SSE event
    ///
    /// Returns `Ok(())` if the event was sent successfully.
    /// Returns `Err(())` if the client disconnected.
    pub async fn send(&self, event: SseEvent) -> std::result::Result<(), ()> {
        self.sender.send(event).map_err(|_| ())
    }

    /// Send a simple message event
    pub async fn send_message(&self, data: impl Into<String>) -> std::result::Result<(), ()> {
        self.send(SseEvent::new(data)).await
    }

    /// Send a keep-alive comment
    pub async fn send_keepalive(&self) -> std::result::Result<(), ()> {
        self.send(SseEvent::new("").comment("keepalive")).await
    }
}

impl Default for SseChannel {
    fn default() -> Self {
        Self::new().0
    }
}

/// SSE Stream for converting events to HTTP response
pub struct SseStream {
    receiver: mpsc::UnboundedReceiver<SseEvent>,
    keep_alive: Option<Interval>,
}

impl SseStream {
    /// Enable keep-alive with a specified interval
    ///
    /// Sends periodic comment lines to keep the connection alive.
    pub fn with_keep_alive(mut self, interval: Duration) -> Self {
        self.keep_alive = Some(tokio::time::interval(interval));
        self
    }

    /// Convert the stream to an HTTP response
    ///
    /// This sets the appropriate headers for SSE:
    /// - Content-Type: text/event-stream
    /// - Cache-Control: no-cache
    /// - Connection: keep-alive
    pub fn into_response(self) -> Response {
        // Create response with SSE headers
        let response = Response::new()
            .header_str("Content-Type", "text/event-stream")
            .header_str("Cache-Control", "no-cache")
            .header_str("Connection", "keep-alive")
            .header_str("X-Accel-Buffering", "no"); // Disable nginx buffering

        // TODO: Implement streaming body support
        // For now, return an empty response with proper headers
        response.body("")
    }
}

impl Stream for SseStream {
    type Item = Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Check for keep-alive first
        if let Some(ref mut interval) = self.keep_alive {
            if interval.poll_tick(cx).is_ready() {
                let event = SseEvent::new("").comment("keepalive");
                return Poll::Ready(Some(Ok(Bytes::from(event.to_sse_format()))));
            }
        }

        // Poll for events from the channel
        match self.receiver.poll_recv(cx) {
            Poll::Ready(Some(event)) => Poll::Ready(Some(Ok(Bytes::from(event.to_sse_format())))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_event_simple() {
        let event = SseEvent::new("Hello, World!");
        let formatted = event.to_sse_format();
        assert_eq!(formatted, "data: Hello, World!\n\n");
    }

    #[test]
    fn test_sse_event_with_type() {
        let event = SseEvent::new("Message content").event("message");
        let formatted = event.to_sse_format();
        assert!(formatted.contains("event: message\n"));
        assert!(formatted.contains("data: Message content\n"));
    }

    #[test]
    fn test_sse_event_with_id() {
        let event = SseEvent::new("Data").id("123");
        let formatted = event.to_sse_format();
        assert!(formatted.contains("id: 123\n"));
    }

    #[test]
    fn test_sse_event_with_retry() {
        let event = SseEvent::new("Data").retry(5000);
        let formatted = event.to_sse_format();
        assert!(formatted.contains("retry: 5000\n"));
    }

    #[test]
    fn test_sse_event_multiline() {
        let event = SseEvent::new("Line 1\nLine 2\nLine 3");
        let formatted = event.to_sse_format();
        assert!(formatted.contains("data: Line 1\n"));
        assert!(formatted.contains("data: Line 2\n"));
        assert!(formatted.contains("data: Line 3\n"));
    }

    #[test]
    fn test_sse_event_comment() {
        let event = SseEvent::new("Data").comment("This is a comment");
        let formatted = event.to_sse_format();
        assert!(formatted.contains(": This is a comment\n"));
    }

    #[tokio::test]
    async fn test_sse_channel_send() {
        let (channel, _stream) = SseChannel::new();

        // Send an event
        let event = SseEvent::new("Test message");
        assert!(channel.send(event).await.is_ok());

        // Try to receive (this would need proper stream impl)
        drop(channel);
        // Stream should be closed
    }

    #[tokio::test]
    async fn test_sse_channel_send_message() {
        let (channel, _stream) = SseChannel::new();
        assert!(channel.send_message("Hello").await.is_ok());
    }
}
