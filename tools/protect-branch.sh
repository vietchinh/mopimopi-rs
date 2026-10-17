#!/usr/bin/env bash
# Makes the "Tests" check required before anything can be merged into `dioxus`.
# Needs the GitHub CLI (`gh auth login`) and admin rights on the repository. Safe to run again.
set -euo pipefail
REPO="${1:-vietchinh/mopimopi}"
BRANCH="${2:-dioxus}"

gh api -X PUT "repos/${REPO}/branches/${BRANCH}/protection" --input - <<JSON
{
  "required_status_checks": { "strict": true, "contexts": ["Tests"] },
  "enforce_admins": true,
  "required_pull_request_reviews": { "required_approving_review_count": 0 },
  "restrictions": null,
  "allow_force_pushes": false,
  "allow_deletions": false
}
JSON
echo "Protected ${REPO}@${BRANCH}: pull request + passing 'Tests' required (branch must be up to date)."
