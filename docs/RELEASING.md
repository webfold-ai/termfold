# Releasing termfold

termfold is not published to crates.io (`publish = false`). A release is a signed source archive on
GitHub.

1. Make sure `main` is green in CI and that `Cargo.lock` is current.
2. Set `version` in `Cargo.toml`. The release workflow rejects a tag that does not match it.
3. Move the `[Unreleased]` entries in `CHANGELOG.md` under a new heading, and add the date.
4. Commit that change and merge it through a reviewed pull request.
5. Create an annotated tag on that commit and push it:

   ```bash
   git tag -a v0.1.0 -m "termfold v0.1.0"
   git push origin v0.1.0
   ```

6. `release.yml` runs on the tag. It checks the version, builds and tests the tagged tree, packages
   `termfold-vX.Y.Z.tar.gz` with `git archive`, signs it, attests provenance, and creates the GitHub
   release with generated notes.
7. Check the release assets against the commands in `docs/SUPPLY-CHAIN.md`.

Tags and releases are created by maintainers. Nothing in this repository creates them automatically.
