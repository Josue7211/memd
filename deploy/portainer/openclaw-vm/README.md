# memd Portainer Deployment

Deploy memd from the memd repo, not from ClawControl.

Canonical Portainer stack file:

- `deploy/portainer/memd-authority.stack.yml`

Scripted deployment path:

- `scripts/deploy-memd-authority.sh`

Runtime ownership:

- stack: `memd-authority-stack`
- container: `memd-authority`
- image repo: `memd-authority`
- network: `memd-authority-network`
- volume: `memd_authority_data`
- host port: `${MEMD_AUTHORITY_PORT:-8788}` -> container `8787`

ClawControl should consume this service through `MEMD_BASE_URL`; it must not
define, build, start, stop, or migrate memd services.
