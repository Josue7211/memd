#!/usr/bin/env bash
set -euo pipefail

docker volume create portainer_agent_data >/dev/null
docker rm -f portainer_agent >/dev/null 2>&1 || true
docker run -d \
  --name portainer_agent \
  --restart=unless-stopped \
  -p 9001:9001 \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v /var/lib/docker/volumes:/var/lib/docker/volumes \
  portainer/agent:2.39.0

echo "Portainer agent listening on :9001"
