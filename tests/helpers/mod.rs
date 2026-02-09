use std::net::TcpListener;

/// Binds to a TCP port and holds it open until dropped.
pub struct ListenerGuard {
    _listener: TcpListener,
    port: u16,
}

impl ListenerGuard {
    pub fn new(port: u16) -> Self {
        let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
            .unwrap_or_else(|_| panic!("failed to bind to port {port}"));
        Self {
            _listener: listener,
            port,
        }
    }

    /// Bind to port 0 and let the OS assign an available port.
    pub fn random() -> Self {
        let listener =
            TcpListener::bind("127.0.0.1:0").expect("failed to bind to random port");
        let port = listener.local_addr().unwrap().port();
        Self {
            _listener: listener,
            port,
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}
