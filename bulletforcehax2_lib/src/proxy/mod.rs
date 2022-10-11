use std::fmt::Display;

pub mod webrequest_proxy;
pub mod websocket_proxy;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    ClientToServer,
    ServerToClient,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::ClientToServer => write!(f, "c->s"),
            Direction::ServerToClient => write!(f, "s->c"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// Indicates what kind of server a websocket is connected to.
pub enum WebSocketServer {
    /// The server where clients connect to to find a game. Called the master server by Photon.
    LobbyServer,
    /// The game server where actual matches happen.
    GameServer,
}

impl WebSocketServer {
    pub fn from_port(port: u16) -> Option<Self> {
        match port {
            2053 => Some(Self::LobbyServer),
            2083 => Some(Self::GameServer),
            _ => None,
        }
    }
}

impl Display for WebSocketServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebSocketServer::LobbyServer => write!(f, "lobby"),
            WebSocketServer::GameServer => write!(f, "game"),
        }
    }
}
