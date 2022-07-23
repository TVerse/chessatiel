FROM rust:1.62.1

COPY . .

RUN cargo build --release

RUN strip target/release/chessatiel
