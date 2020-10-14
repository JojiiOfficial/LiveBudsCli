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

# Install

## AUR
/soon/

## Compilation
Run following command:
```
cargo install earbuds
```

# Usage

Status informations:
```
earbuds status
```

Set equalizer to Bass boost
```
earbuds set eq bass
```

Get Status in json format
```
earbuds status -o json
```
