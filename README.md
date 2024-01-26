# auralist-rs

## How?

### Client

Development
```
npm install
npm run start
```

Build
```
npm run build
```

### Server

Dependencies
```
$ sudo apt install libsqlite3-dev libsqlite3-0 libtagc0-dev
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
OSX?
```
$ brew install taglib
```
Do nothing
```
$ cargo run
```
Initialise conf.ini
```
$ cargo run init
```
Index files
```
$ cargo run index
```
Serve
```
$ cargo run serve
```

```
sudo mount -t cifs -o ro,guest,vers=1.0 //192.168.1.55/music /music
```

- async worker process to index all changes all the time
- page with stats
- need to see distribution of song length
- button to press when song fails
- better ui, sort out milkdrop, it sucks
- possible api/ai based categorisation
- stream the videos
- browsable ui
- better docs