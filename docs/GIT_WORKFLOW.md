# Git Workflow & Collaboration Guide

## Branch Strategy

We use a **protected main branch** with feature branches for all work.

### Branch Naming Convention

```
main                              # Always compiles, protected
├── aksh/milestone-N-feature      # Aksh's work
├── saksham/milestone-N-feature   # Saksham's work
└── proto-update/description      # CRITICAL: Any proto/ change
```

### Rules

| Branch | Who | Protection | Merges | Rules |
|--------|-----|-----------|--------|-------|
| `main` | Both | ✅ Protected | GitHub Actions + both reviews | Never push directly. CI must pass. |
| `aksh/*` | Aksh | ❌ None | Need Saksham review | 1 approval needed |
| `saksham/*` | Saksham | ❌ None | Need Aksh review | 1 approval needed |
| `proto-update/*` | Either | ✅ Protected | BOTH must review | Breaks IPC contract — careful! |

## Starting New Work

### Step 1: Sync with main

```bash
git checkout main
git pull origin main
```

### Step 2: Create Feature Branch

```bash
# If you're Aksh working on Milestone 2 token bucket:
git checkout -b aksh/milestone-2-token-bucket

# If you're Saksham working on WFP engine:
git checkout -b saksham/milestone-1-wfp-engine

# If anyone is changing proto/:
git checkout -b proto-update/add-device-priority
```

### Step 3: Make Changes

```bash
vim daemon/src/bucket.rs
cargo test -p daemon
git add daemon/src/bucket.rs
git commit -m "feat: implement token bucket drain_ready method"
```

### Commit Message Format

Use conventional commits:

```
type(scope): description

[optional body]
[optional footer]
```

Examples:
```
feat(daemon): implement token bucket refill logic
fix(proto): correct PacketMetadata field alignment
docs(setup): add macOS cross-compile instructions
test(daemon): add integration test for bucket drain
chore: update cargo dependencies
```

Types: `feat`, `fix`, `docs`, `test`, `chore`, `refactor`, `perf`

### Step 4: Push & Create PR

```bash
git push origin aksh/milestone-2-token-bucket
```

Then open a PR on GitHub:
- **Title:** Concise description
- **Body:** Why? What changed? Any gotchas?
- **Reviewers:** Assign Saksham (or Aksh if they're reviewing you)
- **Labels:** `daemon`, `crypto`, `wfp-callout`, `ui`, `proto`, or combinations

## Code Review Process

### Submitter's Perspective

1. **Create PR** with clear description
2. **Link issues** (e.g., `Closes #5`)
3. **Wait for review** (not `Approved` until checks pass)
4. **Address feedback** — reply to each comment, push new commits
5. **Resolve conversations** once you've fixed things
6. **Merge** after approval (you can do this, or ask reviewer)

### Reviewer's Perspective

#### For Regular PRs (aksh/* or saksham/*)

Look for:
- ✅ Code compiles without warnings
- ✅ Tests pass (GitHub Actions CI)
- ✅ No unsafe code without SAFETY comments
- ✅ Follows Rust idioms and project conventions
- ✅ Comments explain "why", not "what"

Comment template:
```
## Summary
[What does this PR do?]

## Questions
- [Any unclear patterns?]

## Suggestions
- Consider using `...` instead of `...` for performance
```

#### For Proto Updates (proto-update/*)

⚠️ **CRITICAL:** This changes the IPC contract. Extra scrutiny needed.

Check:
- ✅ Serialization tests added/updated
- ✅ All affected crates updated (wfp-callout, daemon, ui)
- ✅ No breaking changes to existing message types (add-only is OK)
- ✅ Bincode encoding is deterministic
- ✅ Backwards compatibility considered (if live systems exist)

Comment template:
```
## Proto Change Review

- [ ] Serialization test added
- [ ] All consuming crates updated
- [ ] No silent data corruption risk
- [ ] Bincode encoding verified
- [ ] IPC version documented

Approved for merge.
```

## Merging

### When Should You Merge?

- ✅ GitHub Actions passed all checks
- ✅ At least 1 reviewer approved
- ✅ All conversations resolved
- ✅ Branch is up-to-date with main (no merge commits after approval)

### How to Merge

**Option 1: GitHub UI** (Recommended)

1. Click "Squash and merge" (unless 1 commit, then "Create merge commit")
2. Edit final commit message if needed
3. Delete branch

**Option 2: Command Line**

```bash
git checkout main
git pull origin main
git merge --no-ff aksh/milestone-2-token-bucket
git push origin main
```

### If Merge Conflicts

1. **Pull the conflicting branch**
   ```bash
   git checkout aksh/milestone-2-token-bucket
   git pull origin main
   ```

2. **Resolve conflicts**
   ```bash
   git status  # See which files conflict
   vim daemon/src/bucket.rs  # Edit conflict markers
   cargo test  # Verify it still works
   ```

3. **Commit resolution**
   ```bash
   git add daemon/src/bucket.rs
   git commit -m "merge: resolve main branch updates"
   git push origin aksh/milestone-2-token-bucket
   ```

4. **Re-request review** if needed

## CI/CD Pipeline

### GitHub Actions Status

Every PR triggers two workflows:

| Workflow | Runs On | What It Does |
|----------|---------|-------------|
| `build-wfp.yml` | `windows-latest` | Compiles wfp-callout + daemon for Windows |
| `test-daemon.yml` | `ubuntu-latest`, `macos-latest`, `windows-latest` | Runs unit tests on 3 platforms |

**PR cannot merge unless:**
- ✅ All Actions workflows pass (green checkmarks)
- ✅ At least 1 human review approved
- ✅ Branch is up-to-date with main

### Checking CI Status

Click the **Details** link next to each check:

```
✅ build-wfp / build-wfp (pull_request) — All checks passed
✅ test-daemon / test (pull_request) — All checks passed
```

### If CI Fails

1. **Download artifact** (if available)
2. **Read the logs** — click "Logs" tab
3. **Reproduce locally**
   ```bash
   cargo build -p wfp-callout --target x86_64-pc-windows-msvc
   cargo test -p daemon
   ```
4. **Fix and push** — CI re-runs automatically

## Emergency: Direct Main Push (For Admins Only)

If something breaks main and blocks everyone:

```bash
git checkout main
git pull origin main
git commit -m "fix: urgent fix to unblock team"
git push origin main
```

**Post-mortem required:** Document what happened and how to prevent it.

## Pull Request Templates

### Example: Aksh's Token Bucket PR

```markdown
## Description
Implements the token bucket refill logic in the daemon.

Fixes: #12

## Changes
- Added `refill()` method to `DeviceBucket` struct
- Refill adds tokens based on `elapsed().as_secs_f64() * rate`
- Tokens capped at `max_burst_bytes`
- Unit tests verify token accumulation over time

## Testing
- [x] Local tests pass: `cargo test -p daemon`
- [x] Cross-compile verified: Windows target builds
- [x] No unsafe code added

## Type of Change
- [ ] Bug fix
- [x] New feature
- [ ] Breaking change
- [ ] Documentation update

## Checklist
- [x] I have tested this locally
- [x] New tests added/updated
- [ ] Documentation updated
- [ ] No warnings from `cargo clippy`
```

### Example: Saksham's WFP Engine PR

```markdown
## Description
Implements RAII wrapper for WFP engine with proper error handling.

Fixes: #7

## Changes
- Added `WfpEngine` struct with `open()` and Drop implementation
- Defined `WfpError` enum with all Windows error codes
- All unsafe blocks commented with SAFETY rationale
- Verified against Windows Filtering Platform documentation

## Testing
- [x] Builds without warnings
- [x] Can be compiled on windows-latest runner
- [ ] Manual test on Windows PC (will do after merge)

## Checklist
- [x] Unsafe code has SAFETY comments
- [x] No memory leaks (Drop implemented)
- [ ] Performance profiled (not needed yet)
```

### Example: Proto Update PR

```markdown
## Description
Add `priority` field to `DeviceState` for traffic prioritization.

Fixes: #20

## Changes
- Updated `DeviceState` struct: `pub priority: u8` (0=lowest, 255=highest)
- Updated serialization tests for bincode round-trip
- Updated daemon `bucket.rs` to use priority field
- Updated UI to display priority slider

## Testing
- [x] Proto tests pass: `cargo test -p proto`
- [x] Daemon tests pass: `cargo test -p daemon`
- [x] Serialization verified: no data loss

## Backwards Compatibility
⚠️ **Breaking change:** Old devices.json will be missing priority field.
Migration: Default to 128 (medium) if missing.

## Checklist
- [x] Serialization test added
- [x] All affected crates updated
- [ ] Backwards compatibility documented
- [ ] Both reviewers approved (Aksh & Saksham)
```

## Common Workflows

### "I want to see what changed since main"

```bash
git diff main...HEAD
```

### "I want to revert a commit"

```bash
# If it's not pushed yet
git reset --soft HEAD~1  # Undo last commit, keep changes

# If it's already pushed
git revert HEAD  # Create a new commit that undoes HEAD
git push origin branch
```

### "I accidentally committed to main instead of a branch"

```bash
# Find the commit hash
git log --oneline | head -5

# Create a branch from it
git branch aksh/feature 84cf8e9

# Reset main to before your commits
git reset --hard origin/main
```

### "I want to squash multiple commits before PR"

```bash
# If you have 3 commits to squash:
git rebase -i HEAD~3

# Interactive editor opens:
# pick 84cf8e9 first commit
# squash 2d4c8b1 second commit
# squash a9f2e3c third commit

# Then push (may need force)
git push origin aksh/feature --force
```

### "Proto changed on main, I need to merge"

```bash
git checkout main
git pull origin main

git checkout aksh/feature
git merge main

# If conflicts in proto/src/lib.rs:
cargo test -p proto  # Verify merged proto works

git add proto/
git commit -m "merge: sync proto updates from main"
git push origin aksh/feature
```

## Tips & Tricks

### 1. Pre-commit Hook (Optional)

Automatically run tests before committing:

```bash
# Create .git/hooks/pre-commit
#!/bin/bash
cargo test -p proto -p daemon -p crypto || exit 1
```

```bash
chmod +x .git/hooks/pre-commit
```

### 2. Useful Aliases

Add to `~/.gitconfig`:

```ini
[alias]
    co = checkout
    br = branch
    ci = commit
    st = status
    unstage = reset HEAD --
    last = log -1 HEAD
    visual = log --graph --oneline --all
```

Usage:
```bash
git co main
git visual  # See all branches graphically
```

### 3. Keep Fork Updated (If You Fork)

```bash
git remote add upstream https://github.com/aksh-saksham/netshaper.git
git fetch upstream
git rebase upstream/main
git push origin main
```

---

## Communication During Development

### Blocking Issues?

1. **Check GitHub Issues** — someone may have documented it
2. **Post in #netshaper-dev** — mention `@aksh` or `@saksham`
3. **Open an issue** — include error log, what you tried
4. **Pair program** — if stuck >1 hour, ask for live help

### Code Review Questions?

In the PR comment:
```
@saksham Can you explain why `try_consume` returns false instead of panicking here?
```

### When to Discuss Offline

- Architecture changes (call, not comment)
- Blocking disagreements (can't resolve on PR)
- Proto breaking changes (must align first)

---

**Next:** Both teams complete Milestone 0 setup, then start Milestone 1 on parallel branches.
