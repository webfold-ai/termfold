# Supply chain

## What is pinned

- **Rust dependencies.** `Cargo.lock` is committed, and CI builds with `--locked`. Lockfile changes
  come with the dependency change.
- **The one git dependency.** `ruler` is pinned to a full commit with `rev = "a37d96c…"`. The
  repository (`https://github.com/webfold-ai/ruler`) is the only git source allowed in `deny.toml`.
  Changing the rev is a deliberate review point.
- **GitHub Actions.** Every `uses:` is pinned to a 40-hex commit SHA, and the tag is kept in a comment.
  `dtolnay/rust-toolchain` is pinned to the head of its `1.88.0` branch.
- **Toolchain.** CI uses Rust 1.88.0, which is the `rust-version` in `Cargo.toml`.

## Checks on every change

| Check | Workflow | What it enforces |
|---|---|---|
| Format, build, test, clippy, docs | `ci.yml` | Warnings are errors. The lockfile must be current. |
| RustSec advisories | `supply-chain.yml` (`cargo audit --deny warnings`) | Fails on any advisory or warning. Runs daily. |
| Dependency policy | `supply-chain.yml` (`cargo deny check advisories bans licenses sources`) | Licence allow-list, no unknown registries or git sources, no yanked crates. |
| SBOM | `supply-chain.yml` (`cargo cyclonedx`) | A CycloneDX SBOM is uploaded as a 90-day artifact. |
| Dependency review | `dependency-review.yml` | Fails a PR that introduces a high-severity advisory. |
| Static analysis | `codeql.yml` | CodeQL for Rust, `security-extended`. |
| Fuzzing | `fuzz.yml` | libFuzzer on `parse_event`: 3 minutes per PR, daily longer runs. |
| Scorecard | `scorecard.yml` | Weekly. Publishes once the repository is public. |

The advisory and policy checks were run locally on 1.88.0 at this commit: `cargo audit` reported no
advisories, and `cargo deny` passed advisories, bans, licenses and sources.

## Licence policy

`deny.toml` allows MIT, Apache-2.0 and Unicode-3.0. Adding a licence is a review point.

The `wildcards = "warn"` setting is deliberate. A `rev`-pinned git dependency has no version requirement,
which cargo-deny reports as a wildcard. The source is restricted by `allow-git`, so the warning is
accepted rather than suppressed.

## Releases

Release archives are built from the tagged commit. They carry three kinds of evidence:

1. A cosign keyless signature (`.sig` and `.cert`) on the archive and on `SHA256SUMS`. The certificate
   identity is the release workflow at the tag.
2. SLSA build provenance, as a GitHub artifact attestation and as an `.intoto.jsonl` file.
3. A SHA-256 checksum file.

### Verifying a release

These commands have not been run against a real release, because no tag has been pushed yet.

```bash
# Checksums
sha256sum -c SHA256SUMS

# Sigstore signature on the archive
cosign verify-blob \
  --certificate termfold-v0.1.0.tar.gz.cert \
  --signature termfold-v0.1.0.tar.gz.sig \
  --certificate-identity 'https://github.com/Ptyktos/termfold/.github/workflows/release.yml@refs/tags/v0.1.0' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  termfold-v0.1.0.tar.gz

# GitHub build-provenance attestation
gh attestation verify termfold-v0.1.0.tar.gz --repo Ptyktos/termfold
```

## Known gaps

- The GitHub Actions versions are pinned by SHA. Their upstream code is not reviewed here.
- `cargo-audit` and `cargo-deny` are installed from crates.io at fixed versions, `cargo-deny` 0.19.4.
  `cargo-audit` is not pinned.
- The `ruler` dependency is in another repository. Its own supply chain is out of scope here.
