//! State of the link to ACT, shown on the start screen.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// No address configured yet.
    Idle,
    Connecting,
    Connected,
    Disconnected,
}
