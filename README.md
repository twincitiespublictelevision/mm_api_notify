# mm_api_notify [![Latest Version]][crates.io]

`mm_api_notify` is a service for emitting notifications about changes in the
PBS Media Manager system.

---

## Requirements

1. [https://rustup.rs](Rust)
2. [https://www.mongodb.com/](MongoDB)

## Installation

1. `git clone https://github.com/twincitiespublictelevision/mm_api_notify.git`
2. Create a debug or release build `cargo build`
3. Use the `mm_api_import.example` sample or other scripts to create a service
4. Move application build to the location specified in service script
5. Create `config.toml` file (see below)
6. Initially run with the `--build` flag to build the cache for the first
time. This can take 1-2 hours.

## Configuration

A sample `config.toml` file is supplied in `config.toml.example`

### General

| Option            | Value                            |
| ----------------- | -------------------------------- |
| thread_pool_size  | Max number of threads to use     |
| min_runtime_delta | Min time to wait between updates |
| enable_hooks      | Global control over hooks        |

### Database [db]

These are the values required to connect to the MongoDB instance

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
