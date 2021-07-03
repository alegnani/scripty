#syntax=docker  /dockerfile:1

FROM rust:latest
WORKDIR /app
RUN cargo new prg
WORKDIR /app/prg
COPY run.sh run.sh
CMD /bin/bash run.sh
