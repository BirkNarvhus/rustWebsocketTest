# 1. This tells docker to use the Rust official image
FROM rust:1.49 as build


# create a new empty shell project
RUN USER=root cargo new --bin holodeck
WORKDIR /holodeck

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

RUN cargo build --release

# our final base
FROM rust:1.49-slim-buster


# set the startup command to run your binary
CMD ["./rustTest"]