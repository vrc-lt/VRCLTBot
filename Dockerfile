FROM rust:slim
ENV DEBIAN_FRONTEND noninteractive
RUN apt update
RUN apt -y upgrade
RUN apt install -y sudo

RUN sudo apt install -y xpdf ffmpeg 
RUN cargo build
RUN cargo run