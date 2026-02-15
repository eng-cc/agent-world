use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use tungstenite::handshake::HandshakeError;
use tungstenite::protocol::Message;
use tungstenite::{accept, Error as WsError};

#[derive(Debug, Clone)]
pub struct ViewerWebBridgeConfig {
    pub bind_addr: String,
    pub upstream_addr: String,
    pub poll_interval: Duration,
}

impl ViewerWebBridgeConfig {
    pub fn new(bind_addr: impl Into<String>, upstream_addr: impl Into<String>) -> Self {
        Self {
            bind_addr: bind_addr.into(),
            upstream_addr: upstream_addr.into(),
            poll_interval: Duration::from_millis(10),
        }
    }

    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }
}

#[derive(Debug)]
pub enum ViewerWebBridgeError {
    Io(io::Error),
    WebSocket(WsError),
}

impl From<io::Error> for ViewerWebBridgeError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<WsError> for ViewerWebBridgeError {
    fn from(err: WsError) -> Self {
        Self::WebSocket(err)
    }
}

pub struct ViewerWebBridge {
    config: ViewerWebBridgeConfig,
}

impl ViewerWebBridge {
    pub fn new(config: ViewerWebBridgeConfig) -> Self {
        Self { config }
    }

    pub fn run(&self) -> Result<(), ViewerWebBridgeError> {
        let listener = TcpListener::bind(&self.config.bind_addr)?;
        for incoming in listener.incoming() {
            let stream = incoming?;
            if let Err(err) = self.serve_stream(stream) {
                eprintln!("viewer web bridge error: {err:?}");
            }
        }
        Ok(())
    }

    fn serve_stream(&self, stream: TcpStream) -> Result<(), ViewerWebBridgeError> {
        let mut websocket = accept(stream).map_err(map_handshake_error)?;
        websocket.get_mut().set_nonblocking(true)?;

        let upstream = TcpStream::connect(&self.config.upstream_addr)?;
        upstream.set_nodelay(true)?;
        let upstream_reader = upstream.try_clone()?;
        let mut upstream_writer = BufWriter::new(upstream);

        let (tx_from_upstream, rx_from_upstream) = mpsc::channel::<String>();
        thread::spawn(move || {
            read_upstream_lines(upstream_reader, tx_from_upstream);
        });

        loop {
            match websocket.read() {
                Ok(message) => {
                    if !handle_ws_message(message, &mut upstream_writer, &mut websocket)? {
                        break;
                    }
                }
                Err(WsError::Io(err)) if err.kind() == io::ErrorKind::WouldBlock => {}
                Err(WsError::ConnectionClosed) | Err(WsError::AlreadyClosed) => break,
                Err(err) => return Err(err.into()),
            }

            loop {
                match rx_from_upstream.try_recv() {
                    Ok(line) => {
                        websocket.send(Message::Text(line))?;
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => return Ok(()),
                }
            }

            thread::sleep(self.config.poll_interval);
        }

        Ok(())
    }
}

fn handle_ws_message(
    message: Message,
    upstream_writer: &mut BufWriter<TcpStream>,
    websocket: &mut tungstenite::WebSocket<TcpStream>,
) -> Result<bool, ViewerWebBridgeError> {
    match message {
        Message::Text(text) => {
            upstream_writer.write_all(text.as_bytes())?;
            upstream_writer.write_all(b"\n")?;
            upstream_writer.flush()?;
            Ok(true)
        }
        Message::Binary(binary) => {
            if let Ok(text) = String::from_utf8(binary) {
                upstream_writer.write_all(text.as_bytes())?;
                upstream_writer.write_all(b"\n")?;
                upstream_writer.flush()?;
            }
            Ok(true)
        }
        Message::Ping(payload) => {
            websocket.send(Message::Pong(payload))?;
            Ok(true)
        }
        Message::Close(frame) => {
            websocket.close(frame)?;
            Ok(false)
        }
        Message::Pong(_) => Ok(true),
        Message::Frame(_) => Ok(true),
    }
}

fn map_handshake_error(
    err: HandshakeError<
        tungstenite::ServerHandshake<TcpStream, tungstenite::handshake::server::NoCallback>,
    >,
) -> ViewerWebBridgeError {
    match err {
        HandshakeError::Failure(error) => ViewerWebBridgeError::WebSocket(error),
        HandshakeError::Interrupted(_) => ViewerWebBridgeError::Io(io::Error::new(
            io::ErrorKind::Interrupted,
            "websocket handshake interrupted",
        )),
    }
}

fn read_upstream_lines(stream: TcpStream, tx: mpsc::Sender<String>) {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let text = line.trim();
                if text.is_empty() {
                    continue;
                }
                if tx.send(text.to_string()).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_config_new_sets_defaults() {
        let config = ViewerWebBridgeConfig::new("127.0.0.1:5011", "127.0.0.1:5010");
        assert_eq!(config.bind_addr, "127.0.0.1:5011");
        assert_eq!(config.upstream_addr, "127.0.0.1:5010");
        assert_eq!(config.poll_interval, Duration::from_millis(10));
    }
}
