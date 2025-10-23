# Quick Reference - Maintainer Workflow

## Daily Workflow

### Making Changes
```bash
# Work directly on main or use feature branches
git checkout -b feature/my-feature  # optional
# ... make changes ...
git add .
git commit -m "description"
git push origin main  # or feature branch
```

### Merging Feature Branches
```bash
git checkout main
git merge feature/my-feature
git push origin main
# Branch will be auto-deleted on GitHub
```

### Responding to Issues

**Bug Reports**: Issues will have structured data from the template
- Check Version, OS, Reproduction steps
- Respond with acknowledgment or request for more info
- Close when fixed with a reference to the fixing commit

**Feature Requests**: Issues will have structured data from the template
- Consider: Use case, Benefits, Examples
- Add to internal roadmap if valuable
- Close with explanation if not planning to implement

### If Someone Opens a PR

The PR template will automatically inform them that PRs aren't accepted, but if needed:
1. Thank them for their interest
2. Direct them to open an issue instead
3. Close the PR without merging
4. Reference the CONTRIBUTING.md

## Repository Settings Applied

✅ **Automated**:
- Auto-delete merged branches
- Issues enabled
- Projects enabled (for internal tracking)

📝 **Manual Configuration Needed** (one-time):
See `.github/REPOSITORY_SETTINGS.md` for details on:
- Security settings (Dependabot)
- Repository description and topics
- Issue labels
- Branch protection (if you have GitHub Pro)

## Useful Commands

```bash
# View repository settings
gh repo view --json name,isPrivate,hasIssuesEnabled,hasProjectsEnabled,hasWikiEnabled,hasDiscussionsEnabled

# List open issues
gh issue list

# View specific issue
gh issue view 123

# Close an issue
gh issue close 123 -c "Fixed in commit abc123"

# List branches (should stay clean with auto-delete)
git branch -a

# Force push if needed (you have full control)
git push --force origin main
```

## File Locations

- **Contributing Guide**: `CONTRIBUTING.md`
- **Issue Templates**: `.github/ISSUE_TEMPLATE/`
- **PR Template**: `.github/pull_request_template.md`
- **Settings Guide**: `.github/REPOSITORY_SETTINGS.md`
- **This File**: `.github/QUICK_REFERENCE.md`

## Remember

- You can push directly to main
- You can force push if needed
- You can rewrite history
- Feature branches auto-delete after merge
- Issues are valuable feedback - respond thoughtfully
- PRs aren't accepted - template handles this gracefully
