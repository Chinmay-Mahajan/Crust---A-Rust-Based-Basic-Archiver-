FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev

WORKDIR /usr/src/crust
COPY . .
RUN cargo build --release

FROM python:3.11-alpine


RUN apk add --no-cache bash


COPY --from=builder /usr/src/crust/target/release/Crust /usr/local/bin/crust


WORKDIR /workspace
