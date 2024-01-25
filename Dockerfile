FROM rust:1.75.0 as builder
RUN apt-get update && apt-get install -y build-essential libsqlite3-0 libtagc0 && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/auralist-rs
COPY . .
RUN cargo install --path .
FROM debian:buster-slim
RUN apt-get update & apt-get install -y libsqlite3-0 libtagc0 & rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/auralist-rs /usr/local/bin/auralist-rs
CMD ["auralist-rs"]