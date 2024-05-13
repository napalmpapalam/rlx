<h1>
  <p align="center">
    <img alt="rlx" src="https://raw.githubusercontent.com/napalmpapalam/rlx/feature/cl/assets/logo.png" width="450"/>
  </p>
</h1>

<p align="center">
  Work with releases of the npm packages with ease
</p>

<div align="center">
  <a href="https://github.com/napalmpapalam/rlx/blob/master/LICENSE">
    <img alt="License: MIT" src="https://img.shields.io/github/license/napalmpapalam/rlx.svg" />
  </a>
  <a href="https://github.com/napalmpapalam/rlx/actions">
    <img alt="Lint" src="https://github.com/napalmpapalam/rlx/actions/workflows/lint.yaml/badge.svg" />
  </a>
  <a href="https://www.npmjs.com/package/@napalmpapalam/rlx">
    <img alt="NPM version" src="https://img.shields.io/npm/v/@napalmpapalam/rlx.svg" />
  </a>
</div>

## Changelog

For the change log, see [CHANGELOG.md](CHANGELOG.md).

## Installation

To install the `rlx` with the npm, run:

```sh
npm i -D @napalmpapalam/rlx
```

or with the yarn:

```sh
yarn add -D @napalmpapalam/rlx
```

## Configuration

There are a few ways to configure the `rlx`:

- `.rlx.yml` file in the root of the project, names of the options equal to the CLI flags but in the snake case (e.g. `tag-prefix` -> `tag_prefix`)
- Global flags which will be passed to the `rlx` command
- Environment variables with the `RLX_` prefix

Options:

| Option name      | Option alias | Environment variable | Description                                                                                                                                                                                                                                          |
| ---------------- | ------------ | -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `workspace-path` | `-w`         | `RLX_WORKSPACE_PATH` | Path to the workspace directory with the packages directories if it's mono-repo (eg. `rlx --workspace-path ./packages`). Used to infer the packages path for validating `package.json` version. If not provided, the current directory will be used. |
| `changelog-path` | `-cp`        | `RLX_CHANGELOG_PATH` | Path to the changelog file (eg. `rlx --changelog-path ./CHANGELOG.md`). If not provided, the `CHANGELOG.md` file in the root of the project will be used.                                                                                            |
| `debug`          | ❌            | `RLX_DEBUG`          | Enable debug mode, which will print debug logs.                                                                                                                                                                                                      |
| `remote-url`     | `-url`       | `RLX_REMOTE_URL`     | The Git Remote URL of the repository, used to generate compare links in the changelog. If not provided it will be inferred from the git configuration.                                                                                               |
| `tag-prefix`     | `-t`         | `RLX_TAG_PREFIX`     | The tag prefix to use (e.g. `rlx --tag-prefix v`), used to generate compare links in the changelog. If not provided it will empty.                                                                                                                   |
| `head`           | ❌            | `RLX_HEAD`           | The head to use (by default `HEAD`, e.g. `rlx --head master`), used to generate compare links in the changelog                                                                                                                                       |

## Usage

`rlx` CLI provides a set of commands to work with the releases, such as:

- [`rlx rsc`](#rlx-rsc)
- [`rlx change-log`](#rlx-change-log)
- [`rlx help`](#rlx-help)

### `rlx rsc`

Release Sanity Check. Check that a release is sane (`package.json`, `CHANGELOG.md` and semantic versioning are valid for the release).

**Usage:**

```sh
rlx rsc [OPTIONS] [VERSION]
```

**Arguments:**

- `[VERSION]` - The release version to check, if not provided, the not pushed git tag will be used. If no git tag is found, the check will be skipped

The command will check the following:

- The provided version is a valid semver version
- The `package.json` version is equal to the provided version
- The `CHANGELOG.md` contains the provided version release notes and the release date is equal to the current date, and the release compares link is valid\exists

**Example:**

```sh
rlx rsc 1.0.0
```

### `rlx change-log`

Changelog commands, used to parse and manipulate changelog

TODO: Add usage examples

### `rlx help`

Display help for the `rlx` CLI

**Usage:**

```sh
rlx help
```

It will display the help message with the available commands and options such as:

```
Usage: rlx [OPTIONS] <COMMAND>

Commands:
  rsc         Release Sanity Check. Check that a release is sane (`package.json`, `CHANGELOG.md` and semantic versioning are valid for the release)
  change-log  Change Log commands, used to parse and manipulate changelog
  help        Print this message or the help of the given subcommand(s)

Options:
  -w, --workspace-path <WORKSPACE_PATH>
          Path to the workspace directory with the packages directories if it's mono-repo (eg. "./packages"). Used to infer the package(s) path for validating package.json version.

          If not provided, the current directory will be used.

          Can be set via `RLX_WORKSPACE_PATH` environment variable or `workspace_path` config option in the `.rlx.yml` file.

      --debug
          Enable debug mode, which will print debug logs.

          Can be set via `RLX_DEBUG` environment variable or `debug` config option in the `.rlx.yml` file.

      --changelog-path <CHANGELOG_PATH>
          The path to the changelog file, defaults to `CHANGELOG.md`

          Can be set via `RLX_CHANGELOG_PATH` environment variable or `changelog_path` config option in the `.rlx.yml` file.

      --remote-url <REMOTE_URL>
          The Git Remote URL of the repository, used to generate compare links in the changelog.

          If not provided it will be inferred from the git configuration.

          Can be set via `RLX_REMOTE_URL` environment variable or `remote_url` config option in the `.rlx.yml` file.

  -t, --tag-prefix <TAG_PREFIX>
          The tag prefix to use (e.g. `v`), used to generate compare links in the changelog.

          If not provided it will empty.

          Can be set via `RLX_TAG_PREFIX` environment variable or `tag_prefix` config option in the `.rlx.yml` file.

      --head <HEAD>
          The head to use (by default `HEAD`), used to generate compare links in the changelog

          Can be set via `RLX_HEAD` environment variable or `head` config option in the `.rlx.yml` file.

  -h, --help
          Print help (see a summary with '-h')
```

## Contribute

First off, thanks for taking the time to contribute!
Now, take a moment to be sure your contributions make sense to everyone else.

### Reporting Issues

Found a problem? Want a new feature? First of all, see if your issue or idea has [already been reported](../../issues).
If don't, just open a [new clear and descriptive issue](../../issues/new).

### Submitting pull requests

Pull requests are the greatest contributions, so be sure they are focused in scope and avoid unrelated commits.

-   Fork it!
-   Clone your fork: `git clone https://github.com/<your-username>/rlx`
-   Navigate to the newly cloned directory: `cd rlx`
-   Create a new branch for the new feature: `git checkout -b feature/my-new-feature`
-   Make your changes.
-   Commit your changes: `git commit -am 'Add some feature'`
-   Push to the branch: `git push origin feature/my-new-feature`
-   Submit a pull request with full remarks documenting your changes.

## License

[MIT License](https://opensource.org/licenses/MIT) © Semen Loktionov

