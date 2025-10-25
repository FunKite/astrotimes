# GitHub Repository Settings for Maintainer-Only Development

This document outlines the recommended GitHub repository settings for Solunatus, configured for maintainer-only development while welcoming user feedback.

## ✅ Settings Applied via API

The following settings have been configured automatically:

### General Repository Settings
- ✅ **Delete branch on merge**: Enabled - Automatically cleans up merged branches
- ✅ **Issues**: Enabled - Users can report bugs and request features
- ✅ **Projects**: Enabled - For internal project management

### Merge Settings
- ✅ **Allow squash merging**: Enabled
- ✅ **Allow merge commits**: Enabled
- ✅ **Allow rebase merging**: Enabled
- ✅ **Auto-merge**: Disabled

## ⚙️ Manual Settings (Web Interface Required)

Some settings require configuration through the GitHub web interface. Navigate to:
**Settings** → **General** / **Branches** / **Actions** as indicated below.

### Repository Visibility (Settings → General → Danger Zone)

**Current**: Private
**Recommendation**:
- If keeping **Private**: Forking is automatically allowed for private repos (can't be disabled for user-owned repos)
- If making **Public**: Consider the forking implications below

**Note**: For user-owned repositories (not organization-owned), the `allow_forking` setting cannot be changed via API. GitHub automatically allows forking for private repos to enable collaboration, but since we're not accepting PRs, this is less of a concern.

### Pull Requests (Settings → General → Pull Requests)

Configure these settings manually:

- ☑️ **Allow merge commits** - Keep enabled (already set)
- ☑️ **Allow squash merging** - Keep enabled (already set)
- ☑️ **Allow rebase merging** - Keep enabled (already set)
- ☑️ **Automatically delete head branches** - Enabled (already set)

Optional (recommended):
- ☐ **Allow auto-merge** - Keep disabled (already set)
- ☐ **Always suggest updating pull request branches** - Disabled (default)

### Branch Protection Rules (Settings → Branches)

**Note**: Branch protection for private repositories requires GitHub Pro. If you have Pro, configure these settings:

**For the `main` branch**:

Navigate to: **Settings → Branches → Add rule**

Rule name: `main`

**Recommended settings for maintainer-only workflow**:

- ☐ **Require a pull request before merging** - UNCHECKED (you're pushing directly)
- ☐ **Require status checks to pass before merging** - UNCHECKED (optional: enable if using CI)
- ☐ **Require conversation resolution before merging** - UNCHECKED
- ☐ **Require signed commits** - OPTIONAL (good security practice)
- ☐ **Require linear history** - OPTIONAL (keeps history clean)
- ☐ **Require deployments to succeed before merging** - UNCHECKED
- ☐ **Lock branch** - UNCHECKED (you need to push)
- ☐ **Do not allow bypassing the above settings** - UNCHECKED (you need full access)
- ☑️ **Allow force pushes** - CHECKED (you may need to rewrite history)
- ☐ **Allow deletions** - UNCHECKED (prevent accidental deletion)

**If you DON'T have GitHub Pro**: Branch protection is not available for private repos on the free tier. This is acceptable for maintainer-only development since you have full control.

### GitHub Actions (Settings → Actions → General)

If using GitHub Actions for CI/CD:

**Actions permissions**:
- ⚪ Allow all actions and reusable workflows (most permissive)
- ⚪ Allow actions created by GitHub (recommended)
- ⚪ Allow only local actions (most restrictive)

**Workflow permissions**:
- ⚪ Read repository contents and packages permissions (recommended for security)
- ⚪ Read and write permissions (if workflows need to commit)

### Discussions (Settings → General → Features)

**Current**: Disabled
**Recommendation**: Keep disabled
- Use Issues for bug reports and feature requests
- Discussions add overhead and duplicate functionality for a maintainer-only project

### Wiki (Settings → General → Features)

**Current**: Disabled
**Recommendation**: Keep disabled
- Documentation is in the `docs/` directory and versioned with code
- Wiki is not version-controlled and would be redundant

### Projects (Settings → General → Features)

**Current**: Enabled
**Recommendation**: Keep enabled
- Useful for tracking internal roadmap and task management
- Private to the maintainer

## 📝 Additional Recommendations

### Issue Labels

Consider creating these labels for better organization:
- `bug` - Something isn't working (created automatically by bug_report.yml)
- `enhancement` - New feature or request (created automatically by feature_request.yml)
- `documentation` - Improvements to documentation
- `question` - Further information is requested
- `wontfix` - This will not be worked on
- `duplicate` - This issue already exists
- `help wanted` - Community input desired (for discussion, not code)

### Topics (Settings → General → About)

Add relevant topics to help discoverability if/when the repo becomes public:
- `rust`
- `astronomy`
- `cli`
- `astronomical-calculations`
- `solar-position`
- `lunar-phases`
- `ephemeris`
- `terminal-ui`

### Repository Description

Current: "Astrotimes in Rust"
Suggested: "High-precision astronomical calculations CLI - Sun/Moon rise/set times, twilight, and lunar phases"

## 🔒 Security Settings

### Security Advisories (Settings → Security → Code security and analysis)

- ✅ **Dependency graph** - Enable (helps track vulnerabilities)
- ✅ **Dependabot alerts** - Enable (notifies of security issues)
- ✅ **Dependabot security updates** - Enable (auto-creates PRs for security fixes)

**Note**: These are available for private repos and don't conflict with the maintainer-only model since Dependabot PRs can be reviewed and merged by you.

### Code Scanning

Available on public repos or with GitHub Advanced Security. Optional for private repos.

## 📋 Summary

### Configured Automatically ✅
- Auto-delete branches on merge
- Merge method options
- Issues and Projects enabled

### Requires Manual Configuration ⚙️
1. Branch protection rules (if you have GitHub Pro)
2. Security features (Dependabot)
3. Repository description and topics
4. Issue labels

### Not Recommended 🚫
- Discussions (redundant with Issues)
- Wiki (docs are in-repo)
- Auto-merge (not needed for single maintainer)

---

**Last Updated**: 2025-10-23
**Maintainer**: @FunKite
