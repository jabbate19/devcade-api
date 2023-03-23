FROM rust:1.66-slim-buster

RUN apt-get update && apt-get -y install zip libssl-dev pkg-config

COPY ./ ./

RUN cargo build --release

CMD ["./target/release/devcade-api-rs"]