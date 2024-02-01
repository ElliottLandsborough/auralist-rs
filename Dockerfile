FROM rust:alpine as builder

RUN USER=root

RUN apk add --no-cache musl-dev sqlite sqlite-libs sqlite-dev

# Everything needed for compilation
COPY src /app/src/src
COPY Cargo.toml /app/src/Cargo.toml
COPY Cargo.lock /app/src/Cargo.lock

# Compile
WORKDIR /app/src
RUN cargo build --release

FROM alpine:latest
RUN apk update \
    && apk add openssl ca-certificates

EXPOSE 1337

COPY --from=builder /app/src/target/release/auralist-rs /app
COPY ./static/* /static/

CMD ["/app"]