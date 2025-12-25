# Contributing

## Quick Start
You're probably interested in adding a package. The first step is forking and
cloning this repo.

### Maintainer Utilities
Maintainer utilities are provided in `./sh/m`. You'll want to source this file
from a POSIX-compliant shell.

You can add a package with `va mypackage`. There are various utility functions
defined in `./sh/lib.env`. Peruse existing packages for an idea of how to use
them.

### Fields
The available fields are as follows:

```
package
 ├── upstream     [string]
 ├── chance       (float between 0 and 1)
 └── channels     [array]
     ├── name     [string]
     ├── enabled  (bool)
     ├── upstream (string)
     ├── fetch    (string)
     └── expected (string)
```

None of the fields are required, but the recommended fields are typed with
brackets. Omitted fields are populated with sane defaults.

### Editor Configuration
The following config snippet should make working with Vat in Neovim a little
more pleasant by automatically setting the filetype to TOML, enabling syntax
highlighting:

```lua
-- Vat config filetype
vim.filetype.add({
    pattern = {
        [".*/p/.*/config"] = "toml",
    }
})
```

## Commits
Vat follows a variant of conventional commits, and uses pre-commit hooks to
enforce these.

Some general rules:
- Keep commit subject length to 72 characters or fewer.
- Commit subjects should be lowercase and limited to ASCII. Descriptions should
  also keep to ASCII, but may be capitalized as desired.
- Breaking changes (i.e. changes that might impact the version fetching of other
  packages) should start with '!' in the subject line.

To add a package, the commit message would be:
> feat(p): add mypackage

To fix the release fetch for a package, the commit message would be:
> fix(p): fix release fetch for mypackage

To make a breaking tweak to the vtrim function in the shell library, addressing
an issue, and signing off:
> !feat(lib): adjust vtrim behavior
>
> Instead of only trimming a leading 'v', vtrim now trims any leading alphabetic
> character if it's immediately followed by a number.
>
> Resolves: #488
> References: #122, #556

### Commit Types
- auto:     automatic commits made by Vat
- chore:    changes to auxiliary files
- docs:     changes to any documentation
- feat:     a new feature or package
- fix:      a bugfix

### Scopes
Scopes include but are not limited to:
- (p):      packages
- (lib):    the shell library
- (sh):     other shell stuff
- (aux):    auxiliary files

<!-- TODO: Add some more information -->
