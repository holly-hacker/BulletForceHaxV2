# BulletForceHaxV2

[![Build history](https://buildstats.info/github/chart/holly-hacker/bulletforcehaxv2/?branch=main)](https://github.com/HoLLy-HaCKeR/BulletForceHaxV2/actions)

[![Dependency status](https://deps.rs/repo/github/holly-hacker/bulletforcehaxv2/status.svg)](https://deps.rs/repo/github/holly-hacker/bulletforcehaxv2/)

This repository holds a launcher and MITM-based cheat for Bullet Force. It allows you both a convenient way to launch
the game without bloated, ad-ridden websites that redownload the entire game every time, and cheats. I'm making this
public because I've mostly lost interest and want to work on other projects, but I may continue working on it in the
future.

This is a successor of [BulletForceHax](https://github.com/HoLLy-HaCKeR/bulletforcehax).

## How does it work?
`bulletforcehax2_app` will build an executable that runs a webserver and opens a webview that's pointed to it. This
webserver hosts the game files and contains some javascript that will hook Unity's websocket and webrequest functions
to modify all requests to go through that webserver. It will in turn forward it to the original destination and modify
the in-transit packets where it's beneficial.

For more info, see [Development.md](Development.md) and [Photon.md](Photon.md).

## Important notes

If hax is enabled, only a single client may be connected at once. This is because the hax state is tied to the lifetime
of the proxy, so multiple simultaneous connections will cause the state to go mayham.

## Compilation on linux
You will need the required packages to build projects that use webview.

On Ubuntu/Debian:
```bash
sudo apt install -y libwebkit2gtk-4.0-dev libgtk-3-dev
```

On Arch:
```bash
sudo pacman -S webkit2gtk
```

On Fedora:
```bash
sudo dnf install gtk3-devel webkit2gtk3-devel
```

Please note that wayland is currently not supported (see [tauri-egui#7](https://github.com/tauri-apps/tauri-egui/issues/7)).

## FAQ

### The game launches but I see no cheats!
By default, hax are disabled. Create a `config.toml` file that contains `hax = true` next to the exe.

You can check the `--help` output of the executable for more info. All cli arguments can also be entered in the config
file.

### BulletForceHaxV2 creates massive log files!
Make sure to run the game in release mode. This lowers the verbosity of file logs.

### You're ruining the game!
Bullet Force uses an extremely client-authoritive networking model. I'd be surprised if the game wasn't filled with
cheaters already. See this as a gentle prod to the developers to fix their shit.
