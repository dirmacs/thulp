# Vendor Dependencies

This document explains why Thulp vendors certain dependencies and how to manage them.

## Current Vendored Dependencies

### ares-server

**Location**: `vendor/ares-server`  
**Source**: `github.com/dirmacs/ares` (commit `cd9d697`)  
**Reason**: Contains necessary patches not yet published to crates.io

#### Why Vendored?

The `ares-server` crate from crates.io (version 0.3.0 and earlier) has feature-gate compilation errors that prevent building with certain feature combinations. Our fork at `dirmacs/ares` includes fixes for:

1. Feature-gate conditional compilation issues
2. Missing feature flags for optional dependencies
3. Compatibility improvements with `rs-utcp` v0.3.0

#### Patches Applied

The vendored version includes the following patches (commit `cd9d697`):

- Fixed `#[cfg(feature = "...")]` conditional compilation
- Added proper feature flags to `Cargo.toml`
- Updated dependencies to match `rs-utcp` requirements

## When to Remove Vendor

The vendor folder can be removed once:

1. ✅ Patches are merged upstream to `rs-utcp/ares-server`
2. ✅ A new version (≥0.3.1) is published to crates.io
3. ✅ The published version includes all necessary fixes

## How to Remove Vendor

Once the above conditions are met:

### Step 1: Update Cargo.toml

Remove the patch section from the root `Cargo.toml`:

```toml
# DELETE THIS SECTION
[patch.crates-io]
ares-server = { path = "vendor/ares-server" }
```

### Step 2: Update Version

Update the dependency version in `Cargo.toml`:

```toml
[workspace.dependencies]
# Change from:
ares-server = { path = "vendor/ares-server", optional = true }

# To:
ares-server = { version = "0.3.1", optional = true }  # Use the latest published version
```

### Step 3: Remove Vendor Directory

```bash
rm -rf vendor/ares-server
```

### Step 4: Test

```bash
# Clean build to ensure no residual artifacts
cargo clean

# Build with ares feature
cargo build -p thulp-mcp --features ares

# Run tests
cargo test -p thulp-mcp --features ares
```

### Step 5: Update Documentation

Remove or update references to the vendor folder in:

- `README.md` (root)
- `crates/thulp-mcp/README.md`
- This file (`VENDOR.md`)

## How to Update Vendored Dependencies

If you need to update the vendored `ares-server` before it's published:

### Step 1: Pull Latest Changes

```bash
cd vendor/ares-server
git fetch origin
git merge origin/main  # or specific branch/commit
cd ../..
```

### Step 2: Test Compatibility

```bash
cargo test -p thulp-mcp --features ares
```

### Step 3: Document the Update

Update this file with:
- New commit hash
- Date of update
- Reason for update
- Any new patches applied

## Alternative: Using Git Dependencies

Instead of vendoring, you can use git dependencies (not recommended for production):

```toml
[workspace.dependencies]
ares-server = { git = "https://github.com/dirmacs/ares", rev = "cd9d697", optional = true }
```

**Pros**: No vendor folder, easier updates  
**Cons**: Requires internet connection, less reproducible builds, slower CI

## Checking Upstream Status

To check if upstream has published a new version:

```bash
# Check crates.io
cargo search ares-server

# Check git repository
git ls-remote https://github.com/rs-utcp/ares-server

# Check for published versions
curl https://crates.io/api/v1/crates/ares-server | jq '.versions[0].num'
```

## History

### 2025-01-15

- **Action**: Vendored `ares-server` from `dirmacs/ares` commit `cd9d697`
- **Reason**: Feature-gate compilation errors in crates.io version 0.3.0
- **Status**: Waiting for upstream fixes to be published

## Questions?

If you have questions about vendored dependencies, please:

1. Check if patches have been merged upstream
2. Check if a new version is available on crates.io
3. Open an issue in the Thulp repository
4. Contact the maintainers

## Related Files

- `Cargo.toml` - Workspace dependencies and patches
- `crates/thulp-mcp/Cargo.toml` - MCP crate dependencies
- `README.md` - Project overview mentioning vendor situation
