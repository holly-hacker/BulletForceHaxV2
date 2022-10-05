/// <summary>(237) A bool parameter for creating games. If set to true, no room events are sent to the clients on join and leave. Default: false (and not sent).</summary>
pub const SUPPRESS_ROOM_EVENTS: u8 = 237;

/// <summary>(236) Time To Live (TTL) for a room when the last player leaves. Keeps room in memory for case a player re-joins soon. In milliseconds.</summary>
pub const EMPTY_ROOM_TTL: u8 = 236;

/// <summary>(235) Time To Live (TTL) for an 'actor' in a room. If a client disconnects, this actor is inactive first and removed after this timeout. In milliseconds.</summary>
pub const PLAYER_TTL: u8 = 235;

/// <summary>(234) Optional parameter of OpRaiseEvent and OpSetCustomProperties to forward the event/operation to a web-service.</summary>
pub const EVENT_FORWARD: u8 = 234;

/// <summary>(233) Optional parameter of OpLeave in async games. If false, the player does abandons the game (forever). By default players become inactive and can re-join.</summary>
/// Obsolete: Use: IsInactive
pub const IS_COMING_BACK: u8 = 233;

/// <summary>(233) Used in EvLeave to describe if a user is inactive (and might come back) or not. In rooms with PlayerTTL, becoming inactive is the default case.</summary>
pub const IS_INACTIVE: u8 = 233;

/// <summary>(232) Used when creating rooms to define if any userid can join the room only once.</summary>
pub const CHECK_USER_ON_JOIN: u8 = 232;

/// <summary>(231) Code for "Check And Swap" (CAS) when changing properties.</summary>
pub const EXPECTED_VALUES: u8 = 231;

/// <summary>(230) Address of a (game) server to use.</summary>
pub const ADDRESS: u8 = 230;

/// <summary>(229) Count of players in this application in a rooms (used in stats event)</summary>
pub const PEER_COUNT: u8 = 229;

/// <summary>(228) Count of games in this application (used in stats event)</summary>
pub const GAME_COUNT: u8 = 228;

/// <summary>(227) Count of players on the master server (in this app, looking for rooms)</summary>
pub const MASTER_PEER_COUNT: u8 = 227;

/// <summary>(225) User's ID</summary>
pub const USER_ID: u8 = 225;

/// <summary>(224) Your application's ID: a name on your own Photon or a GUID on the Photon Cloud</summary>
pub const APPLICATION_ID: u8 = 224;

/// <summary>(223) Not used currently (as "Position"). If you get queued before connect, this is your position</summary>
pub const POSITION: u8 = 223;

/// <summary>(223) Modifies the matchmaking algorithm used for OpJoinRandom. Allowed parameter values are defined in enum MatchmakingMode.</summary>
pub const MATCH_MAKING_TYPE: u8 = 223;

/// <summary>(222) List of RoomInfos about open / listed rooms</summary>
pub const GAME_LIST: u8 = 222;

/// <summary>(221) Internally used to establish encryption</summary>
pub const TOKEN: u8 = 221;

/// <summary>(220) Version of your application</summary>
pub const APP_VERSION: u8 = 220;

/// <summary>(210) Internally used in case of hosting by Azure</summary>
/// Obsolete: TCP routing was removed after becoming obsolete.
pub const AZURE_NODE_INFO: u8 = 210; // only used within events, so use: EventCode.AzureNodeInfo

/// <summary>(209) Internally used in case of hosting by Azure</summary>
/// Obsolete: TCP routing was removed after becoming obsolete.
pub const AZURE_LOCAL_NODE_ID: u8 = 209;

/// <summary>(208) Internally used in case of hosting by Azure</summary>
/// Obsolete: TCP routing was removed after becoming obsolete.
pub const AZURE_MASTER_NODE_ID: u8 = 208;

/// <summary>(255) Code for the gameId/roomName (a unique name per room). Used in OpJoin and similar.</summary>
pub const ROOM_NAME: u8 = 255;

/// <summary>(250) Code for broadcast parameter of OpSetProperties method.</summary>
pub const BROADCAST: u8 = 250;

/// <summary>(252) Code for list of players in a room.</summary>
pub const ACTOR_LIST: u8 = 252;

/// <summary>(254) Code of the Actor of an operation. Used for property get and set.</summary>
pub const ACTOR_NR: u8 = 254;

/// <summary>(249) Code for property set (Hashtable).</summary>
pub const PLAYER_PROPERTIES: u8 = 249;

/// <summary>(245) Code of data/custom content of an event. Used in OpRaiseEvent.</summary>
pub const CUSTOM_EVENT_CONTENT: u8 = 245;

/// <summary>(245) Code of data of an event. Used in OpRaiseEvent.</summary>
pub const DATA: u8 = 245;

/// <summary>(244) Code used when sending some code-related parameter, like OpRaiseEvent's event-code.</summary>
/// <remarks>This is not the same as the Operation's code, which is no longer sent as part of the parameter Dictionary in Photon 3.</remarks>
pub const CODE: u8 = 244;

/// <summary>(248) Code for property set (Hashtable).</summary>
pub const GAME_PROPERTIES: u8 = 248;

/// <summary>
/// (251) Code for property-set (Hashtable). This key is used when sending only one set of properties.
/// If either ActorProperties or GameProperties are used (or both), check those keys.
/// </summary>
pub const PROPERTIES: u8 = 251;

/// <summary>(253) Code of the target Actor of an operation. Used for property set. Is 0 for game</summary>
pub const TARGET_ACTOR_NR: u8 = 253;

/// <summary>(246) Code to select the receivers of events (used in Lite, Operation RaiseEvent).</summary>
pub const RECEIVER_GROUP: u8 = 246;

/// <summary>(247) Code for caching events while raising them.</summary>
pub const CACHE: u8 = 247;

/// <summary>(241) Bool parameter of CreateGame Operation. If true, server cleans up roomcache of leaving players (their cached events get removed).</summary>
pub const CLEANUP_CACHE_ON_LEAVE: u8 = 241;

/// <summary>(240) Code for "group" operation-parameter (as used in Op RaiseEvent).</summary>
pub const GROUP: u8 = 240;

/// <summary>(239) The "Remove" operation-parameter can be used to remove something from a list. E.g. remove groups from player's interest groups.</summary>
pub const REMOVE: u8 = 239;

/// <summary>(239) Used in Op Join to define if UserIds of the players are broadcast in the room. Useful for FindFriends and reserving slots for expected users.</summary>
pub const PUBLISH_USER_ID: u8 = 239;

/// <summary>(238) The "Add" operation-parameter can be used to add something to some list or set. E.g. add groups to player's interest groups.</summary>
pub const ADD: u8 = 238;

/// <summary>(218) Content for EventCode.ErrorInfo and internal debug operations.</summary>
pub const INFO: u8 = 218;

/// <summary>(217) This key's (byte) value defines the target custom authentication type/service the client connects with. Used in OpAuthenticate</summary>
pub const CLIENT_AUTHENTICATION_TYPE: u8 = 217;

/// <summary>(216) This key's (string) value provides parameters sent to the custom authentication type/service the client connects with. Used in OpAuthenticate</summary>
pub const CLIENT_AUTHENTICATION_PARAMS: u8 = 216;

/// <summary>(215) Makes the server create a room if it doesn't exist. OpJoin uses this to always enter a room, unless it exists and is full/closed.</summary>
// const pub CreateIfNotExists: u8 = 215;

/// <summary>(215) The JoinMode enum defines which variant of joining a room will be executed: Join only if available, create if not exists or re-join.</summary>
/// <remarks>Replaces CreateIfNotExists which was only a bool-value.</remarks>
pub const JOIN_MODE: u8 = 215;

/// <summary>(214) This key's (string or byte[]) value provides parameters sent to the custom authentication service setup in Photon Dashboard. Used in OpAuthenticate</summary>
pub const CLIENT_AUTHENTICATION_DATA: u8 = 214;

/// <summary>(203) Code for MasterClientId, which is synced by server. When sent as op-parameter this is code 203.</summary>
/// <remarks>Tightly related to GamePropertyKey.MasterClientId.</remarks>
pub const MASTER_CLIENT_ID: u8 = 203;

/// <summary>(1) Used in Op FindFriends request. Value must be string[] of friends to look up.</summary>
pub const FIND_FRIENDS_REQUEST_LIST: u8 = 1;

/// <summary>(2) Used in Op FindFriends request. An integer containing option-flags to filter the results.</summary>
pub const FIND_FRIENDS_OPTIONS: u8 = 2;

/// <summary>(1) Used in Op FindFriends response. Contains bool[] list of online states (false if not online).</summary>
pub const FIND_FRIENDS_RESPONSE_ONLINE_LIST: u8 = 1;

/// <summary>(2) Used in Op FindFriends response. Contains string[] of room names ("" where not known or no room joined).</summary>
pub const FIND_FRIENDS_RESPONSE_ROOM_ID_LIST: u8 = 2;

/// <summary>(213) Used in matchmaking-related methods and when creating a room to name a lobby (to join or to attach a room to).</summary>
pub const LOBBY_NAME: u8 = 213;

/// <summary>(212) Used in matchmaking-related methods and when creating a room to define the type of a lobby. Combined with the lobby name this identifies the lobby.</summary>
pub const LOBBY_TYPE: u8 = 212;

/// <summary>(211) This (optional) parameter can be sent in Op Authenticate to turn on Lobby Stats (info about lobby names and their user- and game-counts).</summary>
pub const LOBBY_STATS: u8 = 211;

/// <summary>(210) Used for region values in OpAuth and OpGetRegions.</summary>
pub const REGION: u8 = 210;

/// <summary>(209) Path of the WebRPC that got called. Also known as "WebRpc Name". Type: string.</summary>
pub const URI_PATH: u8 = 209;

/// <summary>(208) Parameters for a WebRPC as: Dictionary&lt;string, object&gt;. This will get serialized to JSon.</summary>
pub const WEB_RPC_PARAMETERS: u8 = 208;

/// <summary>(207) ReturnCode for the WebRPC, as sent by the web service (not by Photon, which uses ErrorCode). Type: byte.</summary>
pub const WEB_RPC_RETURN_CODE: u8 = 207;

/// <summary>(206) Message returned by WebRPC server. Analog to Photon's debug message. Type: string.</summary>
pub const WEB_RPC_RETURN_MESSAGE: u8 = 206;

/// <summary>(205) Used to define a "slice" for cached events. Slices can easily be removed from cache. Type: int.</summary>
pub const CACHE_SLICE_INDEX: u8 = 205;

/// <summary>(204) Informs the server of the expected plugin setup.</summary>
/// <remarks>
/// The operation will fail in case of a plugin mismatch returning error code PluginMismatch 32751(0x7FFF - 16).
/// Setting string[]{} means the client expects no plugin to be setup.
/// Note: for backwards compatibility null omits any check.
/// </remarks>
pub const PLUGINS: u8 = 204;

/// <summary>(202) Used by the server in Operation Responses, when it sends the nickname of the client (the user's nickname).</summary>
pub const NICK_NAME: u8 = 202;

/// <summary>(201) Informs user about name of plugin load to game</summary>
pub const PLUGIN_NAME: u8 = 201;

/// <summary>(200) Informs user about version of plugin load to game</summary>
pub const PLUGIN_VERSION: u8 = 200;

/// <summary>(196) Cluster info provided in OpAuthenticate/OpAuthenticateOnce responses.</summary>
pub const CLUSTER: u8 = 196;

/// <summary>(195) Protocol which will be used by client to connect master/game servers. Used for nameserver.</summary>
pub const EXPECTED_PROTOCOL: u8 = 195;

/// <summary>(194) Set of custom parameters which are sent in auth request.</summary>
pub const CUSTOM_INIT_DATA: u8 = 194;

/// <summary>(193) How are we going to encrypt data.</summary>
pub const ENCRYPTION_MODE: u8 = 193;

/// <summary>(192) Parameter of Authentication, which contains encryption keys (depends on AuthMode and EncryptionMode).</summary>
pub const ENCRYPTION_DATA: u8 = 192;

/// <summary>(191) An int parameter summarizing several boolean room-options with bit-flags.</summary>
pub const ROOM_OPTION_FLAGS: u8 = 191;
