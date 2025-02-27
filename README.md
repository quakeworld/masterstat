# masterstat [![Test](https://github.com/quakeworld/masterstat/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/quakeworld/masterstat/actions/workflows/test.yml) [![crates](https://img.shields.io/crates/v/masterstat)](https://crates.io/crates/masterstat) [![docs.rs](https://img.shields.io/docsrs/masterstat)](https://docs.rs/masterstat/)

> Get server addresses from QuakeWorld master servers

## Installation

```shell
cargo add masterstat
```

## Usage

**Get server addresses from a single master server**

```rust
use std::time::Duration;

let master = "master.quakeworld.nu:27000";
let timeout = Some(Duration::from_secs(2));

match masterstat::server_addresses(&master, timeout) {
    Ok(addresses) => { println!("found {} server addresses", addresses.len()) },
    Err(e) => { eprintln!("error: {}", e); }
}
```

**Get server addresses from multiple master servers** (async, in parallel)

```rust
use std::time::Duration;

async fn test() {
  let masters = ["master.quakeworld.nu:27000", "master.quakeservers.net:27000"];
  let timeout = Some(Duration::from_secs(2));
  let server_addresses = masterstat::server_addresses_from_many(&masters, timeout).await?;
}
```

## See also

* [masterstat](https://github.com/vikpe/masterstat) - golang version
* [masterstat-cli](https://github.com/vikpe/masterstat-cli) - CLI version
