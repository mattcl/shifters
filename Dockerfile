FROM rust:1.70-slim as builder
WORKDIR /usr/src/shifters
COPY . .

RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/shifters /usr/local/bin/shifters
CMD ["shifters"]
