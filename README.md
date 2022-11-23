# sshkm

This is a tool for managing authorized SSH keys for users on a server based on their Github public keys.

## Installation

Currently the tool can be built from source using:

```bash
$ git clone git@github.com:teodor-pripoae/sshkm.git
$ cd sshkm
$ cargo build --release
$ sudo cp target/release/sshkm /usr/local/bin
```

If you wish to create a `deb` file for deploying on Debian/Ubuntu servers you can run:

```bash
$ cargo install cargo-deb
$ make
```

The resulting `deb` file will be in the `deb` directory.

## Configuration

Example `config.yaml`

```yaml
users:
  - username: toni # linux username
    github_username: teodor-pripoae # github username
# Timeout for HTTP requests to Github (default 10s)
timeout: 10
# Interval in seconds for fetching SSH keys when ran a a daemon (default 60s)
interval: 60
# optional, set Github URL for Github enterprise
github_url: "https://github.com
```

## Usage

### Sync

This command syncs all the SSH keys for users in config to their home directories:

```bash
$ sshkm sync -c config.yaml
```

### Daemon

sshkm can also be ran as a daemon, syncing the keys every `interval` seconds.

```bash
$ sshkm daemon -c config.yaml
```