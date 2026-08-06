# Releasing txt2data

## Prerequisites

- Push access to the repository
- `CARGO_REGISTRY_TOKEN` secret configured in GitHub (Settings > Secrets and variables > Actions)


## Release process

### 1. Raise a prepare-release PR

Bump the version in `Cargo.toml`:

```
[package]
version = "0.7.0"
```

Review and merge.

### 2. Tag and push

```
git tag v0.7.0
git push origin main v0.7.0
```

The tag must match `v[0-9]+.[0-9]+.[0-9]+` (e.g. `v0.7.0`).
Pre-release tags like `v0.7.0-rc1` are also supported.

### 3. What happens automatically

Push the tag triggers `release.yml`, which runs jobs in order:

| Job | What it does |
|-----|-------------|
| `init` | Extracts version from tag, detects pre-release |
| `check-version` | Verifies `Cargo.toml` version matches tag |
| `build` | Builds release binaries for 5 targets (linux gnu/musl, linux aarch64, macOS x86_64/arm64) |
| `build-rpm` | Builds RPM packages for x86_64 and aarch64 |
| `release` | Generates changelog, attests provenance (SLSA), creates GitHub Release with all artifacts |
| `publish` | Runs `cargo publish` to crates.io (stable releases only, skipped for pre-releases) |

The publish job is part of the release workflow (not a separate workflow)
because GitHub Actions does not trigger workflows from events created by
`GITHUB_TOKEN`.

## Pre-release

Tag with a pre-release suffix to create a GitHub pre-release:

```
git tag v0.7.0-rc1
git push origin v0.7.0-rc1
```

Pre-releases build the same artifacts but are marked as pre-release on
GitHub and skip `cargo publish`.

## Secrets

| Secret | Where to set | How to get |
|--------|-------------|------------|
| `CARGO_REGISTRY_TOKEN` | Repository > Settings > Secrets | https://crates.io/settings/tokens -- scope to `publish-update` for `txt2data` |
| `GITHUB_TOKEN` | Automatic | Provided by GitHub Actions |
