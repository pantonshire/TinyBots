FROM rust:1.54-alpine as build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
RUN apk update
RUN apk add --no-cache musl-dev protoc
RUN cargo build --release --no-default-features

FROM alpine:latest as runtime
COPY --from=build /app/target/release/trans_rights_bot /usr/local/bin/trans_rights_bot
WORKDIR /bot
COPY docker_runtime/ ./
RUN addgroup -S bot
RUN adduser -SDH -G bot bot
RUN crontab -u bot bot.crontab
ENTRYPOINT ["/usr/sbin/crond", "-f"]
