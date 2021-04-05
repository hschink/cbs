FROM resin/raspberry-pi-debian:buster

RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y libpq-dev && \
    apt-get autoremove -y && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN groupadd -r csb && useradd -r -g csb csb

WORKDIR /home/csb/bin/

COPY target/arm-unknown-linux-gnueabihf/release/cargobike_share_backend .

RUN chown -R csb:csb .

USER csb

CMD ["./cargobike_share_backend"]