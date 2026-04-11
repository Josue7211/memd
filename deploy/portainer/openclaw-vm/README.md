# OpenClaw VM Portainer Deploy

This directory is the deployment path for running `memd-server` on `openclaw-vm`
under Portainer management.

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

1. Build the image on the target VM:
   - `docker build -f deploy/docker/Dockerfile.memd-server -t memd-server:local .`
2. Bootstrap the Portainer agent on `openclaw-vm`:
   - `bash deploy/portainer/openclaw-vm/bootstrap-portainer-agent.sh`
3. Add `openclaw-vm` as a Portainer endpoint in the services Portainer instance.
4. Create the stack in Portainer using:
   - `deploy/portainer/openclaw-vm/memd-server.compose.yml`

## Runtime defaults

- container name: `memd-server`
- exposed port: `8787`
- database volume: `memd_server_data`
- bind address inside container: `0.0.0.0:8787`
