![PRs](https://img.shields.io/badge/PRs-welcome-56cc14?style=flat-square)
[![ci/cd](https://img.shields.io/github/workflow/status/JojiiOfficial/LiveBudsCli/Rust?style=flat-square)](https://github.com/JojiiOfficial/LiveBudsCli/actions?query=workflow%3ARust)
[![crates](https://img.shields.io/crates/dv/earbuds?style=flat-square)](https://crates.io/crates/earbuds)
[![earbuds](https://img.shields.io/aur/version/earbuds?style=flat-square&label=earbuds&logo=arch-linux)](https://aur.archlinux.org/packages/earbuds/)
[![earbuds-git](https://img.shields.io/aur/version/earbuds-git?style=flat-square&label=earbuds-git&logo=arch-linux)](https://aur.archlinux.org/packages/earbuds-git/)

# LiveBudsCli
A free cli tool to control your [Galaxy buds live](https://www.samsung.com/us/mobile-audio/galaxy-buds-live)

<table>
<tr>
    <td>
        <img src=".imgs/status.png" width="auto" height="auto"/>
    </td>
    <td>
        <img src=".imgs/status_extended.png" width="auto" height="auto"/>
    </td>
</tr>
</table>
<br>


# Features
- [x] Equalizer, touchpad-lock and anc control
- [x] Basic Buds status (battery, anc, current equalizer setting, ...)
- [x] Advanced status informations (battery voltage/current, temperature)
- [x] Auto music play/pause on bud remove (via mpris)
- [x] Desktop notification for low battery
- [x] Connect/Disconnect your earbuds easily with a subcommand
- [x] Multiple device support
- [x] Individual device configs
- [x] Automatic sink switch (pulseaudio) [feature: `pulse-sink`, used by default]
- [x] Bash completion (for every shell)
- [x] Change touchpad tap action
- [x] Change config options from cli
- [x] Json output for scripts (via `jq`)

# Install


## Pacman repo
You can get it precompiled from my [pacman repository](https://repo.jojii.de)

## AUR
`yay -S earbuds`

## Compilation
Run following command:
```
cargo install earbuds
```


# Polybar
![Polybar](.imgs/polybar.png)
<br>
You can display the status of your buds live in your polybar with [this script](https://github.com/JojiiOfficial/LiveBudsCli/tree/master/scripts/polybar.sh)
<br>
To achieve this, you have to add following to your polybar config and move the script into your polybar script folder. Don't forget to add `buds` to the modules section.
```
[module/buds]
type = custom/script
interval = 8
label = %output%
exec = ~/.config/polybar/scripts/polybar.sh
click-middle = earbuds toggle anc
click-right = earbuds toggle touchpadlock
```


# Usage
To get most of the features listed above, you need to have a daemon instance running (`earbuds -d`). If you run one of the commands 
listed below, the daemon automatically gets started.

Status informations:
```
earbuds status
```

Set equalizer to Bass boost
```
earbuds set equalizer bass
```

Enable ANC
```
earbuds enable anc
```

Toggle noise reduction or the touchpad lock
```
earbuds toggle anc/touchpad
```

Get Status in json format
```
earbuds status -o json
```

Set the left touchpad tap command to spotify
```
earbuds set tap-action spotify left
```

Connect/Disconnect:
```
earbuds connect/disconnect
```

To debug the daemon run:
```
earbuds -k -d --no-fork
```
This kills the currently running daemon, creates a new one and starts it in the foreground.
