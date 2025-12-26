# TODO

- [ ] Make ERROR and WARN logs less ugly
    - [x] Move timeout errors to rust
    - [ ] Clean up redundant error messages
- [x] Rename project to "vat"
    - This avoids confusion with hashicorp's vagrant
    - I also just prefer that name
- [x] Add upstream as a field for release, unstable, and commit. If defined, the
  regular upstream field should fill out release, unstable, and commit's
  upstreams. This is just a clearer way to override upstreams.
- [x] Add [-p|--pretend] flag that doesn't write any versions, but merely
  displays them.
- [x] Pivot from a deterministic delay system to a chance-based one.
- [x] Add an expected field for release, unstable, and commit. If defined,
  vagrant will compile the regex denoted therein and match against the fetched
  version. A mismatch will indicates a failed fetch. This should help enforce
  correct version detection.
- [ ] Write a shell script to find package versions that have not been updated
  in a while. These might then be manually confirmed.
- [x] Implement parallelization
- [x] Add GitHub issue templates
- [ ] Mirror to vat.tox.wtf
- [x] Add changelog
- [x] Add caching for `gr`
- [x] Add version channels
    - [x] Overhaul the codebase and API
- [x] Test release.sh
- [x] Fix chance skipping
- [x] Fix changelog formatting for breaking changes
- [x] Define `defgitrelease`, `defgitunstable`, `defgitcommit`
- [x] Use mtime for cache
- [x] Move transient data files to .vagrant-cache
- [x] Better display for fetched versions
- [x] Support calling scripts in the fetch command
- [ ] Consider supporting nvchecker
- [x] Figure out package categorization
    - Ideally categorize only as needed by conflict
    - Ideally categorize using directories, so py/build, for instance
    - Would require reworking code to detect packages by looking for their config
    - ~~Probably add an `org` field to `Package` in the form "org/name"~~
- [ ] Support chances at the channel level
- [x] Ensure curl doesn't write incomplete files
- [x] Fix commit script behavior for newly added channels
