#!/usr/bin/env bash

argv0="$0"
set -u

die() {
    printf "%s: %s\n" "$argv0" "$1"
    exit "${2:-1}"
}

git add runcount
git update-index --no-skip-worktree vat.log && git add vat.log
git commit -m "auto(aux): update internal data"

git add p/ALL.* p/*/versions.* p/*/channels/*

# TODO: Use porcelain here
packages_updated=$(git status -s p | grep -vF ALL | cut -d/ -f2 | uniq | wc -l)
versions_updates=$(git status -s p | cut -d/ -f2- | uniq | grep -F channels/)

release_versions_updates=$(echo "$versions_updates" | grep -c 'channels/release$')
unstable_versions_updates=$(echo "$versions_updates" | grep -c 'channels/unstable$')
commit_versions_updates=$(echo "$versions_updates" | grep -c 'channels/commit$')
other_versions_updates=$(echo "$versions_updates" | grep -Evc 'channels/(release|unstable|commit)$')

versions_updated=$(echo "$versions_updates" | wc -l)
vat_version=$(git describe --tags || echo "???")

vat_header="[ Vat v$vat_version | $(date +"%Y-%m-%d %H:%M:%S %z") | #$(<runcount) ]"
git restore --staged p

readarray -t changed < <(git status --porcelain=v1 p/**/channels | awk '{print $2}')
declare -A changed_map
for c in "${changed[@]}"; do
    changed_map["$c"]=1
done

shopt -s globstar
shopt -s nullglob

is_changed() {
    local f="$1"
    for c in "${changed[@]}"; do
        [[ $c == "$f" || $c == $f/* ]] && return 0
    done
    return 1
}

# Paranoia
rm -f .vat-cache/commit-*-*

for p in p/**/config; do
    p="${p%/config}"

    is_changed "$p" || continue
    pname="${p#p/}"

    tmp="$(mktemp -u .vat-cache/commit-$pname-XXXX)"
    :>"$tmp"

    # Per-channel commit message
    for channel in "$p"/channels/*; do
        [[ -v changed_map[$channel] ]] || continue
        cname="${channel#"$p"/channels/}"

        wdiff="$(git diff --word-diff=plain --no-color -- "$channel" |
            tail -n1 |
            sed -e "s,-]{+, -> ," \
                -e 's,^\[-,,' \
                -e 's,+},,' \
                -re 's,([0-9a-f]{8})([0-9a-f]{32}),\1,g'
        )"

        # Channels without prior version history
        if [ -z "$wdiff" ]; then
            wdiff="$(<"$channel")"
        fi

        msg_long="$(printf "%-40s%s\n" "$pname:$cname" "$wdiff")"
        echo "$msg_long" >> "$tmp"

        git add "$channel"
        msg_short="$pname:$cname | $wdiff"
        git commit -m "auto(p): $msg_short" -m "$vat_header"
    done

# Per-package commit message
versions_desc="
$vat_header

$(sed 's,^, - ,' "$tmp")
"

    git add "$p"/versions.*
    git commit -m "auto(p): update versions for $pname" -m "$versions_desc"

    echo "auto(p): update versions for $pname"
    echo "$versions_desc"
done

# Overall commit message
pushd .vat-cache >/dev/null || die "Couldn't access .vat-cache"
desc="
$vat_header

- Completed in $(<elapsed)

- Processed $(<total) packages:
    - Checked   $(<checked)
    - Skipped   $(<skipped)
    - Failed    $(<failed)

- Updated $((versions_updated)) versions for $packages_updated packages:
    - Release   $release_versions_updates
    - Unstable  $unstable_versions_updates
    - Commit    $commit_versions_updates
    - Other     $other_versions_updates

- Updated versions:
$(sed 's,^,    - ,' commit-*-*)
"
popd >/dev/null

echo "$desc"

git add p/ALL.*

git commit -m "auto(p): update versions" -m "$desc"
git push
git update-index --skip-worktree vat.log
