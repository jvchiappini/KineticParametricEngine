# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability in KPE, please do **not** open a public issue.

Instead, report it privately via email:

**security@kpe.dev**

If you do not receive a response within 72 hours, please follow up with a public issue tagged `security` to ensure visibility.

### What to include

- A clear description of the vulnerability
- Steps to reproduce (proof of concept preferred)
- The affected version(s)
- Any potential impact or exploit scenarios

### What to expect

- We will acknowledge receipt within 24 hours
- We will investigate and provide an estimated timeline for a fix
- We will notify you when the fix is released
- We will credit you in the release notes (if desired)

## Scope

The following components are in scope:

- `kpe-core` and all `kpe-*` crates
- WASM bindings (`kpe-wasm`)
- CLI tool (`kpe-cli`)
- Web and desktop applications (`apps/`)

Out of scope:

- Third-party dependencies (report to their respective maintainers)
- Development dependencies
- Example/test code

## Preferred Languages

English or Spanish.
