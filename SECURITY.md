# Security

If you find a security issue in `memd`, do not file it in a public issue with exploit details.

## Report Process

- use GitHub Security Advisories if enabled for the repository
- otherwise contact the maintainers through the private channel configured by the project owner

## What To Include

- what component is affected
- what version or commit you tested
- the minimal reproduction
- whether the issue leaks memory, secrets, or access control

## Scope

Security issues include:

- data exposure
- auth or access control mistakes
- unsafe file handling
- injection or deserialization issues
- privilege boundary violations

Memory correctness bugs are not always security bugs, but they can become one fast if they leak private source or cross-project state.
