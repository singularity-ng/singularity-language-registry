# Test Auto-Approve Workflow

This is a test to verify the auto-approve workflow with the new ORG_GITHUB_TOKEN.

## Expected Behavior:
1. ✅ Claude Auto Review should run automatically
2. ✅ PR should get auto-approved if checks pass
3. ✅ "Require Claude Review for Main" should pass (now checks for actual review text)
4. ✅ Auto-merge should be enabled

## Token Setup:
- `ORG_GITHUB_TOKEN` is configured and working
- Token has `pull_requests: read and write` permission
- Token is scoped to this repository only

