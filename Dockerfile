FROM rust:1.57 AS build
#ENV DEBIAN_FRONTEND noninteractive
#RUN apt update
#RUN apt install -y pkg-config openssl libssl-dev
WORKDIR /app/
COPY Cargo.lock Cargo.toml rust-toolchain ./
COPY src src
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update\
 && apt-get install -y --no-install-recommends poppler-utils ffmpeg ca-certificates \
 && rm -rf /var/cache/apt/lists/*
RUN install -d -o daemon -g daemon /work
COPY --from=build /app/target/release/vrcltbot /usr/local/bin/vrcltbot
WORKDIR /work
ENV DISCORD_TOKEN ""
CMD ["/usr/local/bin/vrcltbot"]
USER daemon
