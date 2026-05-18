# memd Authority Deploy Contract

The shared memd authority is a memd-owned service. ClawControl may consume it,
but ClawControl does not own the memd runtime, image, database, or deploy
lifecycle.

## Names

- Runtime container: `memd-authority`
- Image repo: `memd-authority`
- Data volume: `memd_authority_data`
- Migration public authority port: `8788`
- Final public authority port after explicit cutover: `8787`

Do not deploy memd into a `clawcontrol-*` container or image name. That makes a
memd update look like a ClawControl launch and hides the true owner from agents.

## Guarded Flow

Use:

```bash
scripts/deploy-memd-authority.sh build-only
```

This builds the remote image and mutates no running service.

Use activation only when the port is already owned by the memd authority:

```bash
scripts/deploy-memd-authority.sh activate
```

Activation defaults to port `8788`, creating a side-by-side memd-owned authority
without stopping the legacy `clawcontrol-memd` service on `8787`.

Port `8787` cutover is a separate explicit infra migration. Do not bind `8787`
for memd authority while a `clawcontrol-*` container owns it.

## Agent Rule

Agents may build memd authority images. Agents must not kill, start, or replace
ClawControl-prefixed services while updating memd unless the user explicitly
asks for that specific infra migration.

Live-state sync follows the same boundary. The memd-owned path is
`scripts/live-state-sync-memd.sh`. The ClawControl source importer refuses by
default and requires `MEMD_ALLOW_CLAWCONTROL_SYNC=1`; it may only read an
already-running ClawControl source and must never launch it.

To migrate agents before final cutover, set their shared base URL to the
side-by-side authority after `/healthz` and `/api/status` pass:

```text
http://100.104.154.24:8788
```
