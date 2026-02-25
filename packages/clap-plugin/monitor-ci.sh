#!/bin/bash
# Monitor GitHub Actions CI for Strudel CLAP plugin

echo "🔍 Monitoring GitHub Actions CI"
echo "Repository: https://github.com/space-shell/strudel-clap-plugin"
echo ""

# Check if we're in nix environment
if ! command -v gh &> /dev/null; then
    echo "⚠️  GitHub CLI not found. Run from nix develop environment:"
    echo "   nix develop"
    exit 1
fi

# List recent runs
echo "=== Recent Workflow Runs ==="
gh run list --limit 10

echo ""
echo "=== Watching latest run (if any) ==="
LATEST_RUN=$(gh run list --limit 1 --json databaseId --jq '.[0].databaseId')

if [ -n "$LATEST_RUN" ] && [ "$LATEST_RUN" != "null" ]; then
    echo "Found run ID: $LATEST_RUN"
    echo "Watching..."
    gh run watch "$LATEST_RUN" --exit-status
else
    echo "❌ No workflow runs found yet."
    echo ""
    echo "Possible reasons:"
    echo "1. GitHub Actions may not be enabled"
    echo "2. Workflow hasn't started yet (can take 30-60 seconds)"
    echo "3. There may be an error in the workflow file"
    echo ""
    echo "To enable Actions:"
    echo "1. Go to: https://github.com/space-shell/strudel-clap-plugin/settings/actions"
    echo "2. Enable 'Allow all actions and reusable workflows'"
    echo ""
    echo "To manually trigger:"
    echo "1. Go to: https://github.com/space-shell/strudel-clap-plugin/actions"
    echo "2. Click 'Build CLAP Plugin' workflow"
    echo "3. Click 'Run workflow'"
fi
