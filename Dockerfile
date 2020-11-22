FROM rust:slim
ENV DEBIAN_FRONTEND noninteractive
RUN apt update
RUN apt -y upgrade
RUN apt install -y sudo

RUN sudo apt install -y pkg-config openssl
RUN sudo apt install -y libssl-dev
RUN sudo apt install -y xpdf ffmpeg 
RUN mkdir /app
COPY src /app/src/
COPY ./Cargo.* /app/
WORKDIR /app/
RUN ls
RUN cargo build
ENV DISCORD_TOKEN ""
ENTRYPOINT cargo run
