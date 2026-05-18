# OpenClaw VM Portainer Deploy

This directory is the deployment path for running `memd-server` on `openclaw-vm`
under Portainer management.

This is memd-owned infrastructure. ClawControl may use this authority endpoint,
but memd deploys must not create, start, stop, or replace `clawcontrol-*`
containers. See `docs/contracts/memd-authority-deploy.md`.

## Current reality

- `openclaw-vm` is reachable and has Docker.
- It is not currently exposed as a Portainer endpoint through the local
  `portainer` CLI workflow.
- The local `portainer` helper also depends on a Vault-backed API key, and the
  current shell has Bitwarden locked.

## What is ready now

- `deploy/docker/Dockerfile.memd-server` builds a deployable `memd-server` container.
- `memd-server.compose.yml` is the Portainer stack file.
- `bootstrap-portainer-agent.sh` bootstraps the Portainer agent on
  `openclaw-vm` so the services Portainer instance can manage it.

## Expected flow

1. Generate deploy identity from a clean checkout:
   - `scripts/deploy-memd-server-preflight.sh`
   - Source the output into the shell or copy the three env values into the
     Portainer stack env: `MEMD_GIT_BRANCH`, `MEMD_GIT_COMMIT`,
     `MEMD_GIT_DIRTY`.
2. Build the image on the target VM with identity args:
   - `docker build -f deploy/docker/Dockerfile.memd-server --build-arg MEMD_GIT_BRANCH="$MEMD_GIT_BRANCH" --build-arg MEMD_GIT_COMMIT="$MEMD_GIT_COMMIT" --build-arg MEMD_GIT_DIRTY="$MEMD_GIT_DIRTY" -t memd-server:"$MEMD_GIT_COMMIT" .`
3. Bootstrap the Portainer agent on `openclaw-vm`:
   - `bash deploy/portainer/openclaw-vm/bootstrap-portainer-agent.sh`
4. Add `openclaw-vm` as a Portainer endpoint in the services Portainer instance.
5. Create the stack in Portainer using:
   - `deploy/portainer/openclaw-vm/memd-server.compose.yml`

For direct SSH deploy work, prefer the guarded memd-owned path:

- `scripts/deploy-memd-authority-openclaw.sh build-only`
- `scripts/deploy-memd-authority-openclaw.sh activate`

The guarded script refuses ClawControl-prefixed container/image names and refuses
activation when port `8787` is still owned by a `clawcontrol-*` service.

`GET /api/status` must show the deployed commit, `git_dirty=clean`, and an
acceptable `benchmark_gate` before `server_authority` can be considered ready.

## Runtime defaults

- container name: `memd-authority` for direct guarded deploys,
  `memd-server` for this Portainer stack
- exposed port: `8787`
- database volume: `memd_authority_data` for direct guarded deploys,
  `memd_server_data` for this Portainer stack
- bind address inside container: `0.0.0.0:8787`
