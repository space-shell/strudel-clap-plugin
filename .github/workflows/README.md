# GitHub Actions Workflows

This directory contains CI/CD workflows for the Strudel project.

## Available Workflows

### `clap-plugin.yml` - CLAP Plugin Build

Automatically builds the Strudel CLAP plugin for multiple platforms.

**Triggers:**
- Push to `main` or `feat/clap-daw-plugin` branches
- Pull requests to `main`
- Manual trigger via GitHub UI (workflow_dispatch)
- Only runs when files in `packages/clap-plugin/` change

**Build Targets:**
- ✅ **Linux x86_64** - Native build on Ubuntu
- ✅ **Windows x86_64** - Cross-compiled from Linux using MinGW
- 🚧 **macOS x86_64** - Commented out (requires macOS runner)
- 🚧 **macOS ARM64** - Commented out (requires macOS runner)

**Artifacts:**
Each successful build uploads artifacts containing:
- The compiled plugin binary (`.so`, `.dll`, or `.dylib`)
- README with installation instructions
- Artifacts are retained for 30 days

**Caching:**
Uses GitHub Actions cache for:
- Cargo dependencies
- Cargo registry
- Build artifacts

This significantly speeds up subsequent builds (from ~5-10 minutes to ~1-2 minutes).

## Downloading Build Artifacts

### From GitHub UI:
1. Go to Actions tab in GitHub
2. Click on the workflow run
3. Scroll to "Artifacts" section
4. Download the platform-specific zip file

### Using GitHub CLI:
```bash
# List recent runs
gh run list --workflow=clap-plugin.yml

# Download artifacts from latest run
gh run download --name strudel-clap-x86_64-pc-windows-gnu
```

## Creating Releases

### Automatic Releases:
Push a git tag starting with `v`:
```bash
git tag v0.1.0
git push origin v0.1.0
```

This triggers the workflow which:
1. Builds all platforms
2. Creates zip archives
3. Creates a GitHub Release
4. Uploads all artifacts to the release

### Manual Release:
You can also create releases manually and attach the downloaded artifacts.

## Local Testing

To test the workflow locally, you can use [act](https://github.com/nektos/act):

```bash
# Install act
# https://github.com/nektos/act#installation

# Run the workflow locally (Linux build only)
act push --workflows .github/workflows/clap-plugin.yml

# Run specific job
act --job build --matrix platform.name:"Linux x86_64"
```

Note: Cross-compilation jobs (Windows from Linux) should work in act, but native macOS builds won't work without a macOS runner.

## Troubleshooting

### Build fails with "linker not found"
Ensure the workflow installs the required dependencies:
- For Linux: `libasound2-dev`, `libjack-jackd2-dev`
- For Windows cross-compilation: `mingw-w64`

### Artifacts not uploaded
Check that:
1. The build succeeded
2. The artifact path exists
3. The artifact name doesn't conflict with existing artifacts

### Cache issues
If you suspect cache corruption:
1. Go to Actions → Caches
2. Delete the problematic cache
3. Re-run the workflow

## Enabling macOS Builds

To enable macOS builds:
1. Uncomment the macOS entries in the matrix
2. Push to trigger the workflow
3. Note: macOS runners are slower and more expensive on GitHub Actions

## Workflow Badges

Add these to your README.md to show build status:

```markdown
![CLAP Plugin Build](https://github.com/tidalcycles/strudel/actions/workflows/clap-plugin.yml/badge.svg)
```

## Cost Considerations

GitHub Actions provides:
- **Public repos**: Unlimited minutes
- **Private repos**: 2,000 minutes/month free

Build times (approximate):
- Linux: ~3-5 minutes
- Windows (cross-compile): ~3-5 minutes
- macOS: ~10-15 minutes

Caching reduces build times by 50-70% for subsequent builds.
