# Vetchricore

Learning Veilid by creating a simple terminal-based chat application.

Not an official Veilid project.

## Veilid chat commands

`vetchricore` now uses a profile-first command model:

- Global `--profile <name>` override for all commands.
- `profile add|list|use|remove|show`
- `friend list|add <name> <pubkey>|rename <old> <new>|remove <name>`
- `key gen|show [--reveal]|remove`
- `route create [--listen]`
- `route add --friend <name> --record-key <key>`
- `send chat to <friend> [--message <text>]`

### Quick usage

```powershell
# Create and switch profiles
vetchricore profile add profile2
vetchricore profile use profile2

# Generate your local keypair for the active profile
vetchricore key gen

# Add a friend
vetchricore friend add friend1 VLD0:...

# Start listening by publishing a private-route blob under a DHT record key
vetchricore route create --listen

# Register one of friend1's route record keys then send a message
vetchricore route add --friend friend1 --record-key VLD0:...
vetchricore send chat to friend1 --message "hello"
```