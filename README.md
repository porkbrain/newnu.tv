# newnu.tv

There are some practices that add value.
There are others which rather extract value.
This project attempts to walk a fine line between the two.

Early focus is on automated value extraction from Twitch clips though.
We download popular Twitch content, classify it and group it.
Then we export video files ready to be edited together into a final video
which can be uploaded to YouTube.

## Dependencies

```bash
apt install protobuf-compiler libprotobuf-dev
```

## Database

Sqlite connection behind a mutex.

The convention is to use reference to the connection for scopes which don't
yield (or not for very long anyway).
Scopes which yield take the mutex and lock it as they need.
This allows reasonable concurrency in exchange for simple management.

## Http routes

We typically run first RPC and then insert to the database.
It's more likely that RPC fails than the database op.

## Views

Simple no-css handlebar templates.

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
