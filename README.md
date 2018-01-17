# mm_api_notify
[![CircleCI](https://circleci.com/gh/twincitiespublictelevision/mm_api_notify.svg?style=svg)](https://circleci.com/gh/twincitiespublictelevision/mm_api_notify)

`mm_api_notify` is a service for emitting notifications about changes in the [PBS Media Manager](https://docs.pbs.org/display/MM) system.

---

## Requirements

1. [rustup](https://rustup.rs) - mm_api_notify is built in [Rust](https://www.rust-lang.org/en-US/). It is needed to compile the service, and can be installed easily with [rustup](https://rustup.rs)
2. [MongoDB](https://www.mongodb.com/) - mm_api_notify uses [MongoDB](https://www.mongodb.com/) to store a record of the records it has seen

## Installation

1. `git clone https://github.com/twincitiespublictelevision/mm_api_notify.git`
2. Create a release build `cargo build --release`
3. Use the `mm_api_import.example` sample or other scripts to create a service
4. Move application build to the location specified in service script
5. Create a config file based on `config.toml.example` file (see below)
6. Start the service with `rebuild` to build the cache for the first time.

## Docker Image

Alternatively a [CentOS](https://www.centos.org/) based [Docker](https://www.docker.com/) image is [available for use](https://hub.docker.com/r/tptwebmaster/mm_api_notify/).

## Configuration

A sample config file is supplied in `config.toml.example`

### General

| Option             | Value                                         |
| ------------------ | --------------------------------------------- |
| thread_pool_size   | Max number of threads to use                  |
| min_runtime_delta  | Min seconds to wait between updates           |
| lookback_timeframe | Number of seconds to look back during updates |
| enable_hooks       | Global control over hooks                     |

### Database [db]

These are the values required to connect to the MongoDB instance. Authentication currently is run against the `admin` database.

| Option   |
| -------- |
| host     |
| port     |
| name     |
| username |
| password |

### Media Manager [mm]

| Option                 | Value                                |
| ---------------------- | ------------------------------------ |
| key                    | Media Manager API key                |
| key                    | Media Manager API secret             |
| changelog_max_timespan | Max time allowed between update runs |

### Logging [log]

| Option   | Value            |
| -------- | ---------------- |
| location | Path to log file |
| level    | Level to report  |

### Hooks [hooks]

Hooks allow for defining urls that the service should send notifications to when
objects in Media Manager change. Hooks can be define for each of the emitted
types. The service currently supports basic auth for authentication with urls.

| Option | Value         |
| ------ | ------------- |
| *type* | List of hooks |

A hook consists of 1 required part and 2 optional parts.

`{ url: required, username: optional, password: optional }`

Each *type* has its own list of hooks that it should call to.

## Usage

mm_api_notify watches for changes to resources via the `changelog` endpoint of [Media Manager API](https://docs.pbs.org/display/CDA/Media+Manager+API) and when it sees a change, emits it out as a **POST** or **DELETE** against the defined hooks.

**POST** - Each change is emitted as a nested JSON structure containing the changed resource along with its parent chain up to a franchise.

**DELETE** - Deletes are emitted when an element is listed as a deletion in the `changelog` or when the keys defined for the service are no longer able to access a resource (404 or 403).

## Query Mode

The binary also offers a query mode to generate emit payloads that are useful for debugging what is being sent during and update POST request.

```
mm_api_notify --query asset 0146e77a-b7c2-4492-b791-47586bb2a154
```

---

### Licensing

mm_api_notify is licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the full license text.