# memd Authority Deploy Contract

The shared memd authority is a memd-owned service. ClawControl may consume it,
but ClawControl does not own the memd runtime, image, database, or deploy
lifecycle.

## Names

- Runtime container: `memd-authority`
- Image repo: `memd-authority`
- Data volume: `memd_authority_data`
- Public authority port: `8787`

Do not deploy memd into a `clawcontrol-*` container or image name. That makes a
memd update look like a ClawControl launch and hides the true owner from agents.

## Guarded Flow

Use:

```bash
scripts/deploy-memd-authority-openclaw.sh build-only
```

This builds the remote image and mutates no running service.

Use activation only when the port is already owned by the memd authority:

```bash
scripts/deploy-memd-authority-openclaw.sh activate
```

Activation refuses to remove or replace `clawcontrol-*` containers. Migrating
from a legacy `clawcontrol-memd` container is an explicit infra step: create the
memd-owned authority service, then point ClawControl at it as a consumer.

## Agent Rule

Agents may build memd authority images. Agents must not kill, start, or replace
ClawControl-prefixed services while updating memd unless the user explicitly
asks for that specific infra migration.
