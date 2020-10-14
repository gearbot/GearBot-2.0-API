FROM rust:latest as builder
USER root
WORKDIR /compile
RUN mkdir ./src
RUN echo "fn main() {}" > ./src/main.rs
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN cargo build --release
COPY ./src ./src
RUN rm -f ./target/release/deps/gearbot_api*
RUN cargo build --release
FROM debian:buster-slim
WORKDIR /GearBot_api
COPY --from=builder ./compile/target/release/gearbot_api /GearBot_api/gearbot_api
ENTRYPOINT /GearBot_api/gearbot_api