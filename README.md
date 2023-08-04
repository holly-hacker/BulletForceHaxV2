> [!WARNING]
> You do not want to use this (yet). While the underlying networking library is solid, the GUI interface is very bare-bones and doesn't offer any useful features. I may implement some more useful features in the future, but UI development is boring and motivation is low.

# BulletForceHaxV2

[![Build history](https://buildstats.info/github/chart/holly-hacker/bulletforcehaxv2/?branch=main)](https://github.com/HoLLy-HaCKeR/BulletForceHaxV2/actions)

[![Dependency status](https://deps.rs/repo/github/holly-hacker/bulletforcehaxv2/status.svg)](https://deps.rs/repo/github/holly-hacker/bulletforcehaxv2/)

This repository holds a launcher and MITM-based cheat for Bullet Force. It allows you both a convenient way to launch
the game without bloated, ad-ridden websites that redownload the entire game every time, and cheats. I'm making this
public because I've mostly lost interest and want to work on other projects, but I may continue working on it in the
future.

This is a successor of [BulletForceHax](https://github.com/HoLLy-HaCKeR/bulletforcehax).

## How does it work?
`bulletforcehax2_server` will build an executable that runs a webserver which hosts hosts a copy of the game and a web
UI. Alongside the game, some javascript is shipped  that will hook Unity's websocket and webrequest functions to modify
all requests to go through the server. It will in turn forward it to the original destination and modify the in-transit
packets where it's beneficial.

For more info, see [Development.md](Development.md) and [Photon.md](Photon.md).

## Important notes

If hax is enabled, only a single client may be connected at once. This is because the hax state is tied to the lifetime
of the proxy, so multiple simultaneous connections will cause the state to go mayham.

## FAQ

### I just get a black window with text, but nothing else happens!
In your terminal you should see a link to http://localhost:48897. Click that link and you will see the web UI. This page
contains another link to the game itself. You'll want to keep both windows open.

### The game launches but I see no cheats!
By default, hax are disabled. Create a `config.toml` file that contains `hax = true` next to the exe.

You can check the `--help` output of the executable for more info. All cli arguments can also be entered in the config
file.

### BulletForceHaxV2 creates massive log files!
Make sure to run the game in release mode. This lowers the verbosity of file logs.

### You're ruining the game!
Bullet Force uses an extremely client-authoritive networking model. I'd be surprised if the game wasn't filled with
cheaters already. See this as a gentle prod to the developers to fix their shit.
