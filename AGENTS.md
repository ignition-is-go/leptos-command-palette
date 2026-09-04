# Agent Instructions

## Non-Interactive Shell Commands

Always use non-interactive flags with file operations to avoid hanging on
confirmation prompts.

```bash
cp -f source dest
mv -f source dest
rm -f file
rm -rf directory
cp -rf source dest
```

Use `scp -o BatchMode=yes`, `ssh -o BatchMode=yes`, `apt-get -y`, and
`HOMEBREW_NO_AUTO_UPDATE=1 brew` where applicable.
