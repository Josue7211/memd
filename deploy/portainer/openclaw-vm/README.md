# Deprecated Path

Do not deploy memd from this directory.

memd is its own stack. The canonical deployment path is:

- `deploy/memd-authority/openclaw-vm/memd-authority.compose.yml`
- `scripts/deploy-memd-authority.sh`

Canonical runtime ownership:

- stack: `memd-authority-stack`
- container: `memd-authority`
- image repo: `memd-authority`
- network: `memd-authority-network`
- volume: `memd_authority_data`

This path remains only so old references fail visibly instead of silently
recreating a mixed app stack. Do not add a compose file here.
