FROM rust:1.84.0-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:22.04
WORKDIR /app
COPY --from=builder /app/target/release/stop_on_call /app
CMD [ "./stop_on_call" ]
