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
- work out why it stops sometimes
- total/elapsed time
- show a waveform
- skip through
- make milkdrop respond properly
- page with stats
- need to see distribution of song lengths
- button to press when song fails
- possible api/ai based categorisation
- browsable ui
- better docs