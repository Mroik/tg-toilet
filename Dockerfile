FROM rust:1.81.0-alpine
RUN apk add pkgconfig openssl musl-dev libressl-dev

COPY . /app
WORKDIR /app
RUN cargo b -r

ENTRYPOINT ["./target/release/tg-toilet"]
