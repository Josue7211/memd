# memd Mac Bridge

Mac Bridge is the bundled macOS services bridge for memd and OpenClaw-family
apps. It ships in-tree because Apple Reminders, Contacts, Notes, Find My, and
local Messages data are essential local context on a Mac, not an optional
afterthought.

The bridge stays platform-specific and lives under `integrations/` so the memd
core remains portable across Linux, macOS, and Windows.

## Services

| Endpoint | Source | Purpose |
| --- | --- | --- |
| `GET /health` | bridge | readiness and service list |
| `GET /reminders` | `remindctl` | list Apple Reminders |
| `POST /reminders` | `remindctl` | create reminders |
| `POST /reminders/complete` | `remindctl` | complete reminders |
| `GET /notes` | JXA | list/search Apple Notes |
| `GET /contacts` | JXA | list/search Contacts |
| `GET /findmy/devices` | Find My cache | device location metadata |
| `POST /messages/mark-read` | Messages SQLite | mark conversations read |
| `GET /messages/attachment-raw` | Messages attachments | serve local attachments |

## Install

From the memd checkout on macOS:

```bash
integrations/mac-bridge/install.sh
```

The main memd installer also runs this automatically on macOS unless disabled:

```bash
MEMD_INSTALL_MAC_BRIDGE=0 scripts/install-memd.sh
```

The installer:

- installs Node dependencies
- installs `remindctl` to `~/.local/bin` when missing
- creates `integrations/mac-bridge/.env` with `BRIDGE_PORT` and a generated `BRIDGE_API_KEY`
- installs a user LaunchAgent named `com.memd.mac-bridge`
- writes logs to `/tmp/memd-mac-bridge.log`

## Configure Consumers

Use this URL from apps on the same Mac:

```text
http://127.0.0.1:4100
```

Use this URL from other machines over Tailscale:

```bash
tailscale ip -4
```

Then configure the consumer with:

```text
MAC_BRIDGE_HOST=http://<mac-tailscale-ip>:4100
MAC_BRIDGE_API_KEY=<value from integrations/mac-bridge/.env>
```

For ClawControl, these map to `MAC_BRIDGE_HOST` and `MAC_BRIDGE_API_KEY`.

## Permissions

macOS will prompt when the bridge first touches protected data. Grant access to
the terminal or background process that runs Node:

- Reminders
- Contacts
- Automation for Notes/Reminders where prompted
- Full Disk Access for Messages database and attachments

You can preflight Reminders with:

```bash
~/.local/bin/remindctl status
~/.local/bin/remindctl authorize
```

## Manage

```bash
launchctl print "gui/$(id -u)/com.memd.mac-bridge"
tail -f /tmp/memd-mac-bridge.log
launchctl bootout "gui/$(id -u)" "$HOME/Library/LaunchAgents/com.memd.mac-bridge.plist"
launchctl bootstrap "gui/$(id -u)" "$HOME/Library/LaunchAgents/com.memd.mac-bridge.plist"
```
