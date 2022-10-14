# Photon

Photon is a networking framework for Unity used by BulletForce.

## Photon Libraries

You can get a copy of these files by downloading them from the Unity Asset Store or by finding them hosted on GitHub by third parties. See [Development.md](./Development.md) for more info.

### Plugins/Photon3Unity3D.dll

A .NET DLL that is referenced by the game. It handles all low level networking. This means reading and writing bytes to a network stream and and converting it to one of the basic types such as OperationRequest, OperationResponse, EventData, etc. It is not obfuscated so it can be analyzed with dnSpy.

This DLL is versioned differently to the rest of PUN.

Notable methods:
- `PeerBase.DeserializeMessageAndCallback(StreamBuffer)`: reads from a stream and deserialize

### Plugins/WebSocket

A small amount of plumbing code in the form of .cs files so the WebGL version of Unity can connect over WebSocket or WebTcp.

### Photon Unity Networking/Plugins/PhotonNetwork

A library in the form of .cs files that implements a layer on top of Photon3Unity3D.dll. It provides a more high level interface for operations such as connecting to a lobby and keeping a list of actors ingame.

Of note is LoadbalancingPeer.cs, which implements `PhotonPeer` and contains a bunch of useful constants, and LoadBalancingClient.cs, which implements `IPhotonPeerListener` and maps Operation{Request,Response} and EventData to more high-level objects.

### PhotonChatApi

TODO

## Object transformation

Before Photon can send a message over the wire, it needs to be converted into an OperationRequest, OperationResponse or EventData (other types exist but are only used internally). These types look roughly like this:

```cs
class EventData {
    byte Code;
    Dictionary<byte, object> Parameters;
}
class OperationRequest {
    byte OperationCode;
    Dictionary<byte, object> Parameters;
}
class OperationResponse {
    byte OperationCode;
    short ReturnCode;
    string? DebugMessage;
    Dictionary<byte, object> Parameters;
}
```

Whereas a high-level object from PhotonRealtime may look like this:

```cs
class RoomInfo {
    bool RemovedFromList;
    byte MaxPlayers;
    byte PlayerCount;
    bool IsOpen;
    bool IsVisible;
    bool IsOpen;
    // ...
    
    Hashtable CustomProperties;
}
```

Photon will convert this object to a lower level type (in the case of RoomInfo, this is EventData) and then into a byte stream.
