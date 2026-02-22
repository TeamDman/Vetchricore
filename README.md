# Vetchricore

Learning Veilid by creating a simple terminal-based chat application.

Not an official Veilid project.

## Veilid chat commands

`vetchricore veilid` now includes the same core command flow as `veilid-python-demo`:

- `state` - start/attach/get-state/shutdown summary
- `keygen` - generate and store your local keypair
- `add-friend <name> <pubkey>` - store a friend's public key
- `dump-keystore` - print your keypair + friend keys
- `delete-keystore` - delete local keystore table
- `start <name>` - create a chat DHT key and begin chatting
- `respond <name> <record-key>` - join an existing chat key
- `clean <record-key>` - delete a DHT record key

### Quick usage

```powershell
# Generate your local keypair
vetchricore veilid keygen

# Add a friend
vetchricore veilid add-friend friend1 VLD0:...

# Start a chat (share the generated key)
vetchricore veilid start friend1

# Respond to a shared key
vetchricore veilid respond friend1 VLD0:...
```