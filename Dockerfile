FROM rust:1.81.0-alpine AS builder
RUN apk add pkgconfig openssl musl-dev libressl-dev
COPY . /app
WORKDIR /app
RUN cargo b -r

FROM alpine:3.20.3
RUN mkdir /app
COPY --from=builder /app/target/release/tg-toilet /app
WORKDIR /app
ENTRYPOINT ["./tg-toilet"]
