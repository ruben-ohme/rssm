# rssm

A simple command-line tool to connect to EC2 instances.

### Prerequisites

1. Install Rust:

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build

```shell
make build
```

### Usage

```shell
aws sso login --profile app-dev
./target/debug/rssm --profile app-dev
```

Then just paste from clipboard to connect:
```shell
aws ssm start-session --target <id> --profile <profile>
```

![img.png](docs/img/img.png)