# auralist-rs

## How?

### Client

Development
```
yarn install
yarn run start
```

Build
```
yarn run build
```

### Server

Dependencies
```
$ sudo apt install libsqlite3-dev libsqlite3-0 libtagc0-dev
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Do nothing
```bash
cargo run
```
#### Initialise conf.ini
```bash
cargo run init
```
#### Index files
WARNING: Will delete the old index
```bash
cargo run index
```
#### Serve
```bash
$ cargo run serve
```

### Docker rebuild container
```bash
make reset
```

### How to mount a samba share
```
sudo mount -t cifs -o ro,guest,vers=1.0 //192.168.769.857/music /files
```
### Todo
- async worker process to index all changes all the time
- make milkdrop respond properly
- page with stats
- need to see distribution of song lengths
- button to press when song fails
- possible api/ai based categorisation
- stream the videos
- browsable ui
- better docs

What the indexer will look like:

```
on startup:
- run the full directory walk
  - if any file in the walk exists in the current index
    - if its mtime and size does not match what is in the current db
      - add it to the to-be-indexed queue
  - if file does not exist in the current queue nor the to-be-indexed queue
    - add it to the to-be-indexed queue
- start an event listener for changes
   - if file has been created
      - add to to-be-indexed
  - if file deleted
      - if it exists in the db
        - remove from file db
  - if file edited
    - if exists in the db
      - add to to-be-indexed

- to-be-indexed queue:
  - if the file exists
    - if it exists in the current db
      - if its mtime or size differ
        - update it
  - if the file does not exist
    - update it
```