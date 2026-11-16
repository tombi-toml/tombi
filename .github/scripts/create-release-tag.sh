#!/usr/bin/env bash
set -euo pipefail

tag="${1:-}"
if [[ ! "${tag}" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
  echo "Usage: $0 vX.Y.Z[-prerelease]" >&2
  exit 1
fi

if [[ "$(git branch --show-current)" != main ]]; then
  echo "Create release tags from the main branch." >&2
  exit 1
fi
if [[ -n "$(git status --porcelain)" ]]; then
  echo "Create release tags from a clean working tree." >&2
  exit 1
fi
if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
  echo "Tag ${tag} already exists." >&2
  exit 1
fi

release_dir="$(mktemp -d)"
rmdir "${release_dir}"
trap 'git worktree remove --force "${release_dir}" 2>/dev/null || true' EXIT
git worktree add --detach "${release_dir}" HEAD

readme="${release_dir}/README.md"
latest_url="https://tombi-toml.github.io/tombi/latest"
if ! grep -Fq "href=\"${latest_url}\"" "${readme}"; then
  echo "README.md does not contain the latest documentation URL." >&2
  exit 1
fi
sed -i.bak "s|href=\"${latest_url}\"|href=\"https://tombi-toml.github.io/tombi/${tag}/\"|g" "${readme}"
rm "${readme}.bak"

git -C "${release_dir}" add README.md
git -C "${release_dir}" commit -m "docs: link ${tag} README to versioned documentation"
git tag -a -m "${tag}" "${tag}" "$(git -C "${release_dir}" rev-parse HEAD)"
echo "Created ${tag}. Push it with: git push origin ${tag}"
