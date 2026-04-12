> Secondary/reference doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]].

# Infrastructure Facts

This file is the local truth source for environment-specific infrastructure
facts that should not be guessed from context.

If a fact here is stale, update this file first and treat older statements as
unverified until they are checked again.

## Current Known Facts

- Cloudflare tunnel runs via `systemd`
- Cloudflare tunnel lives on the `plex` VM
- domain: `aparcedo.org`
- shared `memd-server` lives on `openclaw-vm` behind Portainer
- `memd-server` is reachable over Tailscale at `http://100.104.154.24:8787`
- OpenClaw VM Tailscale DNS name is `openclaw.tail8fd5f4.ts.net`
- Portainer on `openclaw-vm` is reachable at `https://100.104.154.24:9443`
- shared `memd-server` is intended to be accessed over Tailscale or an
  equivalent private VPN/private network, not exposed publicly

## Verification Rule

Do not state tunnel, VM, domain, public accessibility, LAN reachability,
Tailscale reachability, or deployment facts unless they were verified locally
from this machine or from the host that owns the service.

If you have not verified them, say `unverified`.

## Required Checks Before Claiming Infra Facts

Run the relevant local checks first:

```bash
systemctl status cloudflared
systemctl cat cloudflared
cloudflared tunnel list
cloudflared tunnel info <name-or-id>
```

If DNS or routing is part of the claim, also verify the configured hostname and
the actual service endpoint before describing it as public.

## What Not To Do

- do not invent public URLs
- do not assume a Cloudflare quick tunnel exists
- do not assume a named tunnel exists on the current machine
- do not assume a service is reachable outside Tailscale
- do not assume the current repo owns the tunnel being discussed

## Operator Note

This file exists to stop false-confidence infra answers. For deployment and
networking claims, local evidence wins over conversational context.
