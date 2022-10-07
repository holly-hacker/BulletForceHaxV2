//! Event codes for [EventData].

/// (230) Initial list of RoomInfos (in lobby on Master)
pub const GAME_LIST: u8 = 230;

/// (229) Update of RoomInfos to be merged into "initial" list (in lobby on Master)
pub const GAME_LIST_UPDATE: u8 = 229;

/// (228) Currently not used. State of queueing in case of server-full
pub const QUEUE_STATE: u8 = 228;

/// (227) Currently not used. Event for matchmaking
pub const MATCH: u8 = 227;

/// (226) Event with stats about this application (players, rooms, etc)
pub const APP_STATS: u8 = 226;

/// (224) This event provides a list of lobbies with their player and game counts.
pub const LOBBY_STATS: u8 = 224;

/// (210) Internally used in case of hosting by Azure
/// Obsolete: TCP routing was removed after becoming obsolete.
pub const AZURE_NODE_INFO: u8 = 210;

/// (255) Event Join: someone joined the game. The new actorNumber is provided as well as the properties of that actor (if set in OpJoin).
pub const JOIN: u8 = 255;

/// (254) Event Leave: The player who left the game can be identified by the actorNumber.
pub const LEAVE: u8 = 254;

/// (253) When you call OpSetProperties with the broadcast option "on", this event is fired. It contains the properties being set.
pub const PROPERTIES_CHANGED: u8 = 253;

/// (253) When you call OpSetProperties with the broadcast option "on", this event is fired. It contains the properties being set.
/// Obsolete: Use PropertiesChanged now.
pub const SET_PROPERTIES: u8 = 253;

/// (252) When player left game unexpected and the room has a playerTtl != 0, this event is fired to let everyone know about the timeout.
/// Obsolete. Replaced by Leave.
pub const DISCONNECT: u8 = 252;

/// (251) Sent by Photon Cloud when a plugin-call or webhook-call failed or events cache limit exceeded. Usually, the execution on the server continues, despite the issue. Contains: ParameterCode.Info.
/// <seealso cref="https://doc.photonengine.com/en-us/realtime/current/reference/webhooks#options"/>
pub const ERROR_INFO: u8 = 251;

/// (250) Sent by Photon whent he event cache slice was changed. Done by OpRaiseEvent.
pub const CACHE_SLICE_CHANGED: u8 = 250;

/// (223) Sent by Photon to update a token before it times out.
pub const AUTH_EVENT: u8 = 223;
