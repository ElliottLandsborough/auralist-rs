FROM rust:alpine as builder

WORKDIR /app/src
RUN USER=root

RUN apk add --no-cache musl-dev sqlite sqlite-libs sqlite-dev

COPY ./ ./
RUN cargo build --release

FROM alpine:latest
RUN apk update \
    && apk add openssl ca-certificates

EXPOSE 1337

COPY ./static/* /static/
COPY ./conf.ini /conf.ini
COPY ./exclusions.txt /exclusions.txt
COPY ./auralist.sqlite3.gz /auralist.sqlite3.gz
COPY --from=builder /app/src/target/release/auralist-rs /app

CMD ["/app", "serve"]