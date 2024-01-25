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

sudo mount -t cifs -o ro,guest,vers=1.0 //192.168.1.55/music /music



docker build -t auralist:latest .

docker run -p 3000:5000 -e INSTANCE_NAME=docker1 -d rust-warp-docker