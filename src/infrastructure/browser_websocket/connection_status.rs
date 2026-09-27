#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionStatus {
    Idle,
    Connecting,
    Connected,
    Disconnected,
}
