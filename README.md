# auralist-rs

## How?

### Client

Development
```
npm install
npm run start
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