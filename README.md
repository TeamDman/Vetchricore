# Vetchricore

Learning Veilid by creating a simple terminal-based chat application.

Not an official Veilid project.

## Veilid chat commands

`vetchricore` now uses a profile-first command model:

- Global `--profile <name>` override for all commands.
- `profile add|list|use|remove|show`
- `known-user list|add <name> <pubkey>|rename <old> <new>|remove <name>`
- `key gen|show [--reveal]|remove`
- `route create [--listen]`
- `route add --known-user <name> --record-key <key>`
- `send chat to <known-user> [--message <text>]`
- `media player list [--output-format auto|text|json]` (configured preferences only)
- `media player add|new|set|update|create <player-key> <path-to-exe>`
- `media player show <player-key>`
- `media player default set <player-key>`
- `media player default show`
- `media player detect|discover now [--output-format auto|text|json] [--walk yes|no|true|false|ask] [--walk-timeout 25s] [--walk-roots "C:\\;D:\\Apps"]`

### Quick usage

```powershell
# Create and switch profiles
vetchricore profile add profile2
vetchricore profile use profile2

# Generate your local keypair for the active profile
vetchricore key gen

# Add a known user
vetchricore known-user add user1 VLD0:...

# Start listening by publishing a private-route blob under a DHT record key
vetchricore route create --listen

# Register one of user1's route record keys then send a message
vetchricore route add --known-user user1 --record-key VLD0:...
vetchricore send chat to user1 --message "hello"

# Detect media players available on PATH and persist results
vetchricore media player detect now
vetchricore media player discover now

# Optionally walk filesystem recursively (timed)
vetchricore media player detect now --walk true --walk-timeout 25s

# Restrict walk to specific roots
vetchricore media player detect now --walk true --walk-roots "C:\\Program Files;D:\\MediaTools"

# Configure and inspect media players
vetchricore media player add vlc "D:\\programs\\vlc.exe"
vetchricore media player set vlc "D:\\programs\\vlc.exe"
vetchricore media player create vlc "D:\\programs\\vlc.exe"
vetchricore media player show vlc
vetchricore media player default set vlc
vetchricore media player list --output-format json
```