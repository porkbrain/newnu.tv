# newnu.tv

There are some practices that add value.
There are others which rather extract value.
This project attempts to walk a fine line between the two.

Early focus is on automated value extraction from Twitch clips though.
We download popular Twitch content, classify it and group it.
Then we export video files ready to be edited together into a final video
which can be uploaded to YouTube.

## How to run this?

If you want to use docker for local development, then install [`cross`][cross-install] and [`just`][just-install].

If you don't want to use it, then you need to [install][grpc-protoc-install] from source or if your package manager has a recent version, you can use that.
Just be careful as some package managers have quite dated versions.

There are two relevant commands you want to run if you use docker for local development:

1. **`$ just up`** to compile everything and start all containers.
   This process prints logs to stdout and into `./tmp/**/logs` directories for each service.

2. **`$ just watch`** to watch for changes in the source code, recompile and restart relevant containers

## Database

Sqlite connection behind a mutex.

The convention is to use reference to the connection for scopes which don't
yield (or not for very long anyway).
Scopes which yield take the mutex and lock it as they need.
This allows reasonable concurrency in exchange for simple management.

## Http routes

In the admin, we typically run first the RPC call and then insert to the database.
It's more likely that RPC fails than the database op.
This is a naive approach but works for now.

## Views

Simple no-css handlebar templates.

## Crates

### [`twitch`](crates/twitch)

Wrapper around the Twitch API crate.
This crate only exists because we cannot specify two versions of `reqwest` in the same project, and the Twitch API crate uses an older version in its public API while not exporting it, therefore requiring us to define it as a dependency.

### [`observation`](crates/observation)

We use `tracing` for logging.
There are two layers right now:

- pretty print to stderr
- jsonlines to rolling log files

This crate puts it all together so that the binaries can use it as a one-liner.

# To do

- worker can download clip mp4s from twitch
- worker can run a clip through whisper
  - whisper can be run as a container dependency of worker
- groups
  - clips are manually selected
  - a group can be set to auto downloaded and to be processed
  - when viewing a game clip show groups it belongs to
  - allow adding game clip to a new group
- label data with gpt
  - given context and clip, cluster clip with respect to context
  - the transcribed text is then vectorized
  - search on vectors and select the top N clips
- backup db
- select multiple broadcasters at once in clips view
- no need for a separate twitch crate

<!-- List of Links -->

[grpc-protoc-install]: https://grpc.io/docs/protoc-installation/#install-pre-compiled-binaries-any-os
[cross-install]: https://github.com/cross-rs/cross
[just-install]: https://github.com/casey/just?tab=readme-ov-file#installation
