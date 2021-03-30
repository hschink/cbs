# 1: Build the exe
FROM rustlang/rust:nightly as builder

WORKDIR /usr/src

RUN USER=root cargo new cargobike_share_backend

WORKDIR /usr/src/cargobike_share_backend

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

COPY src/ src/
COPY migrations/ migrations/

RUN cargo build --release

# 2: Copy the exe and extra files to an empty Docker image
FROM debian:buster-slim

RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y libpq-dev && \
    apt-get autoremove -y && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN groupadd -r csb && useradd -r -g csb csb

WORKDIR /home/csb/bin/

COPY --from=builder /usr/src/cargobike_share_backend/target/release/cargobike_share_backend .
COPY --from=builder /usr/src/cargobike_share_backend/migrations/ migrations/

RUN chown -R csb:csb .

USER csb

CMD ["./cargobike_share_backend"]