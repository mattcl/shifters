FROM rust:1.73-alpine as builder

RUN apk add musl-dev

WORKDIR /usr/src/shifters
COPY . .

RUN cargo install --locked --target-dir /target --path .

FROM alpine:3.18
COPY --from=builder /usr/local/cargo/bin/shifters /usr/local/bin/shifters
CMD ["shifters"]
