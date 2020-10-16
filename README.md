# LiveBudsCli
A cli tool to control your Galaxy live buds

# Features
- [x] Equalizer, touchpad-lock and anc control
- [x] Buds status (battery, anc, current equalizer setting, ...)
- [x] Auto music play/pause on bud remove (via mpris)
- [x] Desktop notification for low battery
- [x] Multiple device support
- [x] Individual device configs
- [x] Json output for scripts (via `jq`)
- [x] Bash completion (for every shell)
- [ ] Change touchpad tap action
- [ ] Change config options from cli

# Install

## AUR
`yay -S earbuds-git`

## Compilation
Run following command:
```
cargo install earbuds
```

# Polybar
![Polybar](/.imgs/polybar.png)
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
earbuds set eq bass
```

Toggle noise reduction or the touchpad lock
```
earbuds toggle anc/touchpad
```

Get Status in json format
```
earbuds status -o json
```
