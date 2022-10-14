//! Defines information for Bullet Force's RPC calls

use std::borrow::Cow;

use photon_lib::highlevel::structs::RpcCall;

pub const METHOD_NAMES: [&str; 80] = [
    "AcknowledgeDamageDoneRPC",
    "AnotherRPCMethod",
    "BecomeNewMasterClient",
    "ChangeCrouchState",
    "Chat",
    "CmdGetTeamNumber",
    "ColorRpc",
    "DestroyRpc",
    "DisplayVoteData",
    "DoJump",
    "FetchCheaters",
    "FetchVoteData",
    "FlagOwnerTeamUpdated",
    "FlagTakenValueUpdated",
    "Flash",
    "GetBestSpawnPointForPlayer",
    "GotKillAssist",
    "HealthUpdated",
    "InstantiateRpc",
    "JSNow",
    "KickPlayer",
    "LatencyReceive",
    "LatencySend",
    "localCreateGrenade",
    "localHurt",
    "localReload",
    "localSpawnThrowingWeapon",
    "MapVotedFor",
    "Marco",
    "MatchOverChanged",
    "mpMeleeAnimation",
    "mpThrowGrenadeAnimation",
    "MyRPCMethod",
    "NukeKill",
    "PickupItemInit",
    "PlayerHitPlayer",
    "PlayerKickedForPing",
    "Polo",
    "PunPickup",
    "PunPickupSimple",
    "PunRespawn",
    "ReliabilityMessageReceived",
    "ReliabilityMessageSent",
    "RequestForPickupItems",
    "RequestForPickupTimes",
    "RequestVipsOnMasterFromSubordinate",
    "RestartHardcoreModeRound",
    "RestartMatch",
    "RpcDie",
    "RPCElevatorButtonPressed",
    "RpcSendChatMessage",
    "RpcShoot",
    "RpcShowHitmarker",
    "RpcShowPerkMessage",
    "SetElevatorsClosed",
    "SetMaps",
    "SetNextMap",
    "SetPing",
    "SetRank",
    "SetSpawnPoint",
    "SetTimeScale",
    "ShowAnnouncement",
    "ShowDebugCapsule",
    "SpawnFailed",
    "TaggedPlayer",
    "TeleportToPosition",
    "UpdateAlivePlayers",
    "UpdateHMFFARounds",
    "UpdateMPDeaths",
    "UpdateMPKills",
    "UpdateMPRounds",
    "UpdateTeamNumber",
    "UpdateTeamPoints",
    "UpdateTimeInMatch",
    "UpdateVIPsOnSubordinates",
    "UsernameChanged",
    "WeaponCamoChanged",
    "WeaponTypeChanged",
    "RpcACKill",
    "RpcForceKillstreak",
];

/// Get the method name of an RPC call.
///
/// This function gets the string method name if it is present, or otherwise resolves the method index using the
/// hardcoded [METHOD_NAMES] list.
pub fn get_rpc_method_name(data: &RpcCall) -> anyhow::Result<Cow<str>> {
    if let Some(idx) = data.rpc_index {
        Ok(Cow::Borrowed(METHOD_NAMES[idx as usize]))
    } else if let Some(method_name) = &data.method_name {
        Ok(Cow::Owned(method_name.clone()))
    } else {
        anyhow::bail!("malformatted call, neither method name nor index was present")
    }
}
