# Create a release tag

Create each release tag from a clean, up-to-date `main` checkout:

```sh
.github/scripts/create-release-tag.sh vX.Y.Z
git push origin vX.Y.Z
```

The script creates a release-only commit whose README links to `/tombi/vX.Y.Z/`. It tags that commit without changing `main`, whose README continues to link to `/tombi/main/`. The tag push builds the versioned documentation and release packages. Existing tags are immutable and are not changed by this procedure.
