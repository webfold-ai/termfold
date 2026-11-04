# Security policy

## Supported versions

termfold is pre-release (`0.1.x`, not published to crates.io). Only the `main` branch receives
security fixes. There are no backports.

## Reporting a vulnerability

Please report suspected vulnerabilities privately, not in a public issue or pull request:

**https://github.com/Ptyktos/termfold/security/advisories/new** (GitHub private vulnerability reporting), or email the security team at security@twn.systems

Include the affected commit, the platform and terminal emulator, and a minimal input that
reproduces the problem (for the input parser, the raw bytes are usually enough).

We will acknowledge reports as soon as a maintainer sees them. This is a best-effort project with no
response-time guarantee. Once a report is confirmed, we will fix it on `main`, publish a GitHub
security advisory, and credit the reporter unless they ask not to be named.

## Scope

In scope:

- The byte-level input parser (`termfold::terminal::parse_event`): any panic, overflow, unbounded
  allocation or hang on attacker-controlled bytes.
- Terminal state handling: raw-mode enable/restore (`stty` invocations), mouse-capture toggling.
- Dependency advisories that affect termfold's build or runtime (`cargo audit` and `cargo deny` run in CI).

Out of scope: the terminal emulator itself, the operating system's `stty` implementation, and
issues that require an attacker who already runs code as the user.

## Verifying releases

Release archives are signed with Sigstore (cosign keyless) and carry SLSA build provenance. See
`docs/SUPPLY-CHAIN.md` for verification commands.
