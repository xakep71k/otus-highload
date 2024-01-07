FROM rust:latest as builder
WORKDIR /src
RUN git clone https://github.com/xakep71k/otus-highload.git
WORKDIR /src/otus-highload/homework1
RUN cargo build --release

FROM debian:latest

COPY --from=builder /src/otus-highload/homework1/target/release/social-network /usr/local/bin/social-network
EXPOSE 8080
ENTRYPOINT social-network --postgres-conn-string "$POSTGRES_CONN_STRING" --bind-string "$BIND_STRING"
