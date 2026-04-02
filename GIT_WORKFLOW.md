# Git Workflow Guide - Netshaper Project

## Branch Structure (Git Flow)

```
main (Production-Ready)
 ├── Stable releases only
 ├── Merged via pull requests from release/ branches
 └── Protected - requires PR review

develop (Development)
 ├── Main development branch
 ├── Base for all feature branches
 ├── Always compilable and passing tests
 └── Merged after feature completion

feature/* (Feature Development)
 ├── Created from: develop
 ├── Naming: feature/description or aksh/feature-name
 ├── Merged back to: develop via PR
 └── Deleted after merge

release/* (Pre-Release)
 ├── Created from: develop
 ├── Naming: release/v1.0.0
 ├── For final testing and bug fixes
 └── Merged to: main (with tag) and develop

hotfix/* (Emergency Fixes)
 ├── Created from: main
 ├── Naming: hotfix/issue-description
 ├── Critical production fixes only
 └── Merged to: main (with tag) and develop
```

## Workflow Rules

### 1. Main Branch (Production)
- **NEVER commit directly** to main
- Only merge from PR or `git merge --no-ff release/...`
- Every commit on main should be a stable release
- Tag every merge: `git tag -a v1.0.0 -m "Release version 1.0.0"`

### 2. Develop Branch (Integration)
- All feature branches merge here via PR
- Must always be compilable
- Daily development happens here
- Create backup before major changes

### 3. Feature Branches (Your Work)
- **Always branch from develop**: `git checkout develop && git pull && git checkout -b feature/xyz`
- **Naming convention**: 
  - Personal work: `aksh/feature-name`
  - Team features: `feature/feature-name`
- **Create PR when ready** (not commits directly to develop)
- **Rebase before merging**: Keep history clean

### 4. Pull Request Process
1. Create feature branch from develop
2. Make commits with clear messages
3. Push to remote: `git push -u origin aksh/feature-name`
4. Open PR on GitHub
5. Code review & discussion
6. Merge via GitHub (not terminal)
7. Delete feature branch
8. Delete local branch: `git branch -d aksh/feature-name`

## Current Status

| Branch | Status | Purpose |
|--------|--------|---------|
| `main` | ✅ Protected | Production releases |
| `develop` | ✅ Active | Integration & daily dev |
| `aksh/milestone-2-token-bucket` | 📝 Feature | M5 Phase 5 implementation |

## Quick Commands

### Setup
```bash
# Clone project
git clone https://github.com/aksh08022006/Wifly.git
cd netshaper

# Switch to develop
git checkout develop
git pull origin develop
```

### Start New Feature
```bash
# Update develop first
git checkout develop
git pull origin develop

# Create feature branch
git checkout -b aksh/my-feature-name

# Make changes and commit
git add .
git commit -m "feat: description of changes"

# Push to remote
git push -u origin aksh/my-feature-name
```

### Before Creating PR
```bash
# Ensure develop is latest
git fetch origin
git rebase origin/develop

# Push updated branch
git push -f origin aksh/my-feature-name
```

### After Merging PR
```bash
# Clean up locally
git checkout develop
git pull origin develop
git branch -d aksh/my-feature-name
git fetch -p  # Remove deleted remote branches
```

## Branch Protection Rules (GitHub)

The following are recommended settings for both `main` and `develop`:

- ✅ Require pull request reviews before merging (1 reviewer minimum)
- ✅ Require status checks to pass (CI/CD pipelines)
- ✅ Require branches to be up to date before merging
- ✅ Require code reviews from code owners
- ✅ Dismiss stale PR approvals when new commits are pushed
- ✅ Include administrators in restrictions

## Merging Strategy

### Feature → Develop (Squash + Merge)
```bash
# On PR page: "Squash and merge"
# Keeps develop history clean
```

### Develop → Release → Main (Create Merge Commit)
```bash
# On PR page: "Create a merge commit"
# Preserves branch history for releases
```

## Commit Message Format

```
type(scope): short description

Longer explanation if needed.
- Bullet point 1
- Bullet point 2

Closes #issue-number
```

### Types
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Tests
- `chore`: Maintenance
- `perf`: Performance

## GitHub Actions / CI/CD

When PR is created:
1. ✅ Build verification
2. ✅ Tests
3. ✅ Linting checks
4. ✅ Code coverage

**Never merge if checks fail!**

## Troubleshooting

### Accidentally Committed to Main?
```bash
git reset --soft HEAD~1  # Undo last commit, keep changes
git checkout develop
git commit -m "feat: description"
```

### Need to Update Feature with Latest Develop?
```bash
git fetch origin
git rebase origin/develop
git push -f origin aksh/my-feature-name
```

### Reset to Remote State
```bash
git fetch origin
git reset --hard origin/develop
```

## Best Practices

1. **Small PRs are better** - Easier to review, less merge conflicts
2. **Descriptive commit messages** - Future you will thank present you
3. **Test before pushing** - Run locally first
4. **One feature per branch** - Don't mix unrelated changes
5. **Delete branches after merge** - Keeps repository clean
6. **Keep develop stable** - Don't push broken code
7. **Use `.gitignore`** - Never commit build artifacts

---

**Last Updated**: 3 April 2026
**Git Flow Version**: 1.0
**Status**: ✅ Ready for Team Development
