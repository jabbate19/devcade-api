FROM rust:1.66-slim-buster

WORKDIR /app

RUN apt-get update && apt-get -y install zip libssl-dev pkg-config

RUN cargo init .

COPY Cargo.* ./

RUN cargo build --release

COPY src/ ./

RUN cargo build --release

CMD ["./target/release/devcade-api-rs"]