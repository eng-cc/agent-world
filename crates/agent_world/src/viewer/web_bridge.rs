use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
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
            let stream = match incoming {
                Ok(stream) => stream,
                Err(err) => {
                    eprintln!("viewer web bridge accept error: {err:?}");
                    continue;
                }
            };
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
        let upstream_shutdown = upstream.try_clone()?;
        let mut upstream_writer = BufWriter::new(upstream);

        let (tx_from_upstream, rx_from_upstream) = mpsc::channel::<String>();
        let upstream_reader_thread = thread::spawn(move || {
            read_upstream_lines(upstream_reader, tx_from_upstream);
        });

        let session_result = (|| -> Result<(), ViewerWebBridgeError> {
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
        })();

        // Ensure the cloned reader side is also released; otherwise upstream may keep the
        // first session alive and block subsequent reconnects.
        let _ = upstream_shutdown.shutdown(Shutdown::Both);
        let _ = upstream_reader_thread.join();

        session_result
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
            match websocket.close(frame) {
                Ok(()) | Err(WsError::ConnectionClosed) | Err(WsError::AlreadyClosed) => {}
                Err(err) => return Err(err.into()),
            }
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
    use std::time::Instant;
    use tungstenite::connect;

    #[test]
    fn bridge_config_new_sets_defaults() {
        let config = ViewerWebBridgeConfig::new("127.0.0.1:5011", "127.0.0.1:5010");
        assert_eq!(config.bind_addr, "127.0.0.1:5011");
        assert_eq!(config.upstream_addr, "127.0.0.1:5010");
        assert_eq!(config.poll_interval, Duration::from_millis(10));
    }

    #[test]
    fn bridge_allows_reconnect_after_websocket_refresh() {
        let upstream_listener = TcpListener::bind("127.0.0.1:0").expect("bind upstream");
        let upstream_addr = upstream_listener.local_addr().expect("upstream addr");
        let (upstream_tx, upstream_rx) = mpsc::channel::<String>();

        let upstream_thread = thread::spawn(move || {
            let stream_one = accept_with_timeout(&upstream_listener, Duration::from_secs(2))
                .expect("accept first upstream session");
            let mut reader_one = BufReader::new(stream_one);
            let mut line = String::new();
            reader_one
                .read_line(&mut line)
                .expect("read first line from first session");
            upstream_tx
                .send(format!("session1:{}", line.trim()))
                .expect("send session1 line");

            reader_one
                .get_mut()
                .set_read_timeout(Some(Duration::from_millis(50)))
                .expect("set timeout on first session");
            let close_wait_start = Instant::now();
            let first_closed = loop {
                line.clear();
                match reader_one.read_line(&mut line) {
                    Ok(0) => break true,
                    Ok(_) => break false,
                    Err(err)
                        if err.kind() == io::ErrorKind::WouldBlock
                            || err.kind() == io::ErrorKind::TimedOut =>
                    {
                        if close_wait_start.elapsed() >= Duration::from_secs(2) {
                            break false;
                        }
                    }
                    Err(_) => break false,
                }
            };
            upstream_tx
                .send(format!("session1_closed:{first_closed}"))
                .expect("send close state");

            let stream_two = accept_with_timeout(&upstream_listener, Duration::from_secs(2))
                .expect("accept second upstream session");
            let mut reader_two = BufReader::new(stream_two);
            line.clear();
            reader_two
                .read_line(&mut line)
                .expect("read first line from second session");
            upstream_tx
                .send(format!("session2:{}", line.trim()))
                .expect("send session2 line");
        });

        let bridge = ViewerWebBridge::new(
            ViewerWebBridgeConfig::new("127.0.0.1:0", upstream_addr.to_string())
                .with_poll_interval(Duration::from_millis(1)),
        );

        run_ws_session(&bridge, "first-request");
        assert_eq!(
            upstream_rx
                .recv_timeout(Duration::from_secs(1))
                .expect("session1 message"),
            "session1:first-request"
        );
        assert_eq!(
            upstream_rx
                .recv_timeout(Duration::from_secs(1))
                .expect("session1 close state"),
            "session1_closed:true"
        );

        run_ws_session(&bridge, "second-request");
        assert_eq!(
            upstream_rx
                .recv_timeout(Duration::from_secs(1))
                .expect("session2 message"),
            "session2:second-request"
        );

        upstream_thread.join().expect("join upstream thread");
    }

    fn run_ws_session(bridge: &ViewerWebBridge, payload: &str) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ws listener");
        let ws_addr = listener.local_addr().expect("ws addr");
        let payload = payload.to_string();

        let client_thread = thread::spawn(move || {
            let url = format!("ws://{ws_addr}");
            let (mut client, _) = connect(url.as_str()).expect("connect ws client");
            client
                .send(Message::Text(payload))
                .expect("send ws payload");
            client.close(None).expect("close ws client");
        });

        let stream = accept_with_timeout(&listener, Duration::from_secs(2))
            .expect("accept websocket stream");
        bridge.serve_stream(stream).expect("serve websocket stream");
        client_thread.join().expect("join ws client thread");
    }

    fn accept_with_timeout(listener: &TcpListener, timeout: Duration) -> io::Result<TcpStream> {
        listener.set_nonblocking(true)?;
        let start = Instant::now();
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    stream.set_nonblocking(false)?;
                    listener.set_nonblocking(false)?;
                    return Ok(stream);
                }
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                    if start.elapsed() >= timeout {
                        listener.set_nonblocking(false)?;
                        return Err(io::Error::new(io::ErrorKind::TimedOut, "accept timed out"));
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                Err(err) => {
                    listener.set_nonblocking(false)?;
                    return Err(err);
                }
            }
        }
    }
}
