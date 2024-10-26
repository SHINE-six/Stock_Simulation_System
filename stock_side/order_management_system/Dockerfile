FROM rust:1.80.1 as builder
WORKDIR /app

ARG app_name

# clang is a C compiler
# lld is a linker
# musl-dev is a standard C library
RUN apt-get update && apt-get install -y clang lld musl-dev

COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src

RUN cargo install --path .

# release mode meaning that the compiler will optimize the code
RUN cargo build --release && \
    cp ./target/release/${app_name} /bin/server

# ----------- Final Image -----------
FROM ubuntu:latest AS final

COPY --from=builder /bin/server /bin/

CMD ["/bin/server"]
