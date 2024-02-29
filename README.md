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
- total/elapsed time
- show a waveform
- skip through
- make milkdrop respond properly
- page with stats
- better docs

### Example nginx
```
upstream prod_http {
    server 127.0.0.1:1337;
    #server 192.168.1.2:1337;
    keepalive 64;
    keepalive_time 200m;
    keepalive_timeout 180s;
}

server {
        listen 80;
        listen [::]:80;

        server_name example.com;

        location / {
                proxy_pass  http://prod_http;
                proxy_http_version 1.1;
                #proxy_set_header Connection "";
        }
}
```

## Notes

```
SELECT path FROM files where duration > 1000 and duration <  10000 and file_ext = 'mp3' ORDER BY path;
```