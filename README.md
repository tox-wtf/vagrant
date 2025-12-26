<div align="center">
    <img src="https://www.dandreapartners.com/wp-content/uploads/2024/01/vat.jpg" width="25%">
    <h1 align="center">Vat</h1>
</div>

<h2 align="center">Version Aggregator and Tracker</h2>

Vat aggregates and tracks arbitrarily many versions for a collection of
packages. Though designed with Linux From Scratch maintenance in mind, Vat
is adaptable to most tasks requiring version fetching.

Feature set:
- Arbitrary version channels
- Caching
- Multiple APIs
- Single and bulk requests


## Using the APIs
Two APIs are provided for convenience. Choose whichever works better for your
use case.

> [!WARNING]
> API stability is not currently guaranteed.

### Plaintext API
The plaintext API is accessible through a file hierarchy. Individual version
channels are stored in files under `./p/$package/channels/$channel`. This API
was designed to be used easily from a shell with standard utilities.

#### Examples
To check the release version channel of ffmpeg:
```sh
curl -fsSL https://raw.githubusercontent.com/tox-wtf/vat/refs/heads/master/p/ffmpeg/channels/release
```

To check the sdk version channel of glslang:
```sh
curl -fsSL https://raw.githubusercontent.com/tox-wtf/vat/refs/heads/master/p/glslang/channels/sdk
```

To retrieve the release, unstable, and commit version channels of bc, saving
them to variables, in a single request:
```sh
curl -fsSL https://raw.githubusercontent.com/tox-wtf/vat/refs/heads/master/p/bc/versions.txt > _
release=$(grep release _ | cut -f2)
unstable=$(grep unstable _ | cut -f2)
commit=$(grep commit _ | cut -f2)
rm _
```

To retrieve all version channels for all packages, display them, and then parse
out acl's release and inih's commit:
```sh
curl -fsSL https://raw.githubusercontent.com/tox-wtf/vat/refs/heads/master/p/ALL.txt > _

# display versions prettily
expand -t 32,44 _ | cat

acl_release=$(grep acl _ | grep release | cut -f3)
inih_commit=$(grep 'inih\scommit' _ | cut -f3)
rm _
```

To count the number of tracked release versions:
```sh
curl -fsSL https://raw.githubusercontent.com/tox-wtf/vat/refs/heads/master/p/ALL.txt |
    grep '\srelease\s' |
    wc -l
```


### JSON API

#### Examples
To retrieve a JSON object of all version channels of btop:
```sh
curl -fsSL https://raw.githubusercontent.com/tox-wtf/vat/refs/heads/master/p/btop/versions.json
```

To retrieve all versions and parse out lz4's release:
```sh
curl -fsSL https://raw.githubusercontent.com/tox-wtf/vat/refs/heads/master/p/ALL.json |
    jq -r '
    .[] |
    select(.package == "lz4") |
    .versions[] |
    select(.channel == "release") |
    .version
    '
```


## Running
Vat must be run from its source directory. This is by design as Vat is intended
to be run in a controlled/contained environment, and doing so reduces
complexity.

To test that all packages work, execute the following command:
```bash
make test
```

To update the database, execute the following command:
```bash
make run
```

> [!TIP]
> You may want to reset the runcount:
> ```bash
> printf 0 > runcount
> ```


### Dependencies
#### Required
- GCC libraries
- Glibc

#### Buildtime
- Cargo
- Make

#### Runtime
- Bash
- Coreutils
- Curl
- Git
- [Versort](https://github.com/tox-wtf/versort)

#### Development
In addition to the required, buildtime, and runtime dependencies, you'll want
the following:
- Rustup
- Typos


## Roadmap

### Automation
I intend for Vat to be a mostly automatic system that runs on its own at
specified intervals. It should only need intervention for the addition of new
packages or the amelioration of upstream messing with their versioning scheme.

For the moment, I intend to use GitHub workflows for this automation, but plan
to self-host Vat in the future. (This should also make for a less ugly URL.)

### Collaboration
I'd love to work alongside anyone building a package repository, and I want this
to be a community-driven project. I'm open to tracking new packages. For more
information, read [the contributing guidelines](./CONTRIBUTING.md).
