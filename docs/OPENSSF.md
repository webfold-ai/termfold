# OpenSSF Scorecard and Best Practices status

This records what the repository does for each OpenSSF check. "Config" means the file exists in
this tree. Nothing here has been observed on GitHub yet: the repository is private, and the
Scorecard result and the Best Practices badge depend on the repository being public and on
maintainer action.

## Scorecard checks

| Check | Status | Evidence |
|---|---|---|
| Binary-Artifacts | Config | No binaries are tracked (checked with `git grep -I`). |
| Branch-Protection | **TODO (maintainers)** | Not configured. Rulesets are not applied from this repository. |
| CI-Tests | Config | `.github/workflows/ci.yml` runs fmt, build, test, clippy and doc on each PR. |
| CII-Best-Practices | **TODO (maintainers)** | No badge registered. See the Best Practices section below. |
| Code-Review | **TODO (maintainers)** | Depends on branch protection requiring reviews. |
| Contributors | Not met | There is one contributor in git history. This is not padded with made-up names. |
| Dangerous-Workflow | Config | No `pull_request_target` or `workflow_run` triggers. |
| Dependency-Update-Tool | Config | `.github/dependabot.yml`: cargo (root and `fuzz/`) and github-actions, weekly. |
| Fuzzing | Config | `fuzz/` with a libFuzzer target (`libfuzzer-sys`), run by `.github/workflows/fuzz.yml`. |
| License | Config | `LICENSE-MIT` and `LICENSE-APACHE`, matching `license` in `Cargo.toml`. |
| Maintained | Time-gated | Scores up once the repository is more than 90 days old and has recent activity. |
| Packaging | Partial | `publish = false`. The release workflow builds a signed source archive, not a registry package. |
| Pinned-Dependencies | Config | Every `uses:` is a 40-hex SHA, with the tag in a comment. `Cargo.lock` is committed. There is no Dockerfile. |
| SAST | Config | `.github/workflows/codeql.yml` (Rust, `security-extended`). |
| Security-Policy | Config | `SECURITY.md` points to GitHub private vulnerability reporting. |
| Signed-Releases | Config, not yet exercised | `release.yml` signs with cosign (keyless) and attaches SLSA provenance. No tag has been pushed. |
| Token-Permissions | Config | Top-level `permissions` in every workflow (`contents: read`, or `read-all` for Scorecard). Write scopes are set per job only where needed. |
| Vulnerabilities | Met at this commit | `cargo audit` reports no advisories. Re-checked daily by `supply-chain.yml`. |
| Webhooks | N/A | No webhooks are configured by this repository. |

## Open items for maintainers

1. **CODEOWNERS.** Resolved: `.github/CODEOWNERS` lists `@twn-lloyd @yokoszn`, two human owners.
2. **Branch protection.** Require pull requests, at least one approving review (two if you want the
   stricter caretaker tier), code-owner review, and the `Rust checks` status check. Block force-pushes
   and deletions. Do not add admin bypass.
3. **Best Practices badge.** Register the project at https://www.bestpractices.dev, then add the badge
   URL to the README. The badge needs a project ID that does not exist yet, so none is written here.
4. **Scorecard publication.** `scorecard.yml` sets `publish_results` from the repository's visibility,
   so results publish automatically once the repository is public.
5. **Dependency review.** `dependency-review.yml` needs the dependency graph enabled. On private
   repositories it also needs GitHub Advanced Security.

## Caretaker

The workflows and policy files were adapted by hand from the templates in `caretaker-ossf`
(`caretaker/caretaker-ossf/templates/`). `caretakerctl` could not be built in this environment, because it
depends on private git repositories, so `caretakerctl ossf render` was not run. The invariants the
caretaker tests enforce were checked directly instead: every action is SHA-pinned, the CI job is named
`Rust checks`, and CODEOWNERS lists at least two owners (placeholders for now).

Changes from the templates:

- `fuzz_http` and the GHCR container job are removed. termfold has no HTTP surface and no image.
- `release.yml` builds a source archive, because termfold has no binary.
- The fuzz target is `termfold::terminal::parse_event`. The template's `fuzz_import` entry does not apply.
- The Rust toolchain is pinned to 1.88.0, the `rust-version`, instead of 1.94.
- The `dtolnay/rust-toolchain` pin from the templates (`6bed0761…`) is not a current commit of
  that action's branches, so this repository pins the `1.88.0` branch head `360a4e93…`. The stale pin
  should be checked in caretaker before its templates are reused elsewhere.

Caretaker's fleet tooling can take over the rest. Not run here, and not without maintainer input:

```bash
caretakerctl ossf apply --repo Ptyktos/termfold --crate-name termfold \
  --import-entry termfold::terminal::parse_event \
  --code-owners '@real-owner-1 @real-owner-2' --skip-ruleset
```

`--skip-ruleset` avoids changing branch protection on an existing repository until the owners are set.
The ruleset is applied in a separate step, after review.

## Best Practices (bestpractices.dev)

Met by the repository as it stands: public-facing documentation, a license, a vulnerability reporting
process, automated tests in CI, static analysis (CodeQL), a fuzz target, a dependency update tool,
and a warning-free build with `-D warnings`.

Not met: a registered project, code review by a second human, and the owner list above.
