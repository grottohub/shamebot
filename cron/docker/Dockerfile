FROM rust:1.66 AS builder

WORKDIR /usr/src
COPY ./database ./database
COPY ./discord ./discord
COPY ./utils ./utils
COPY ./cron ./cron
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*
RUN cargo install --path ./cron

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/cron /usr/local/bin/cron

ENV POSTGRES_HOST=database
ENV POSTGRES_PORT=5432
ENV POSTGRES_USERNAME=shamebot
ENV POSTGRES_PASSWORD=secret
ENV POSTGRES_DB=shamebot
ENV POSTGRES_INIT_NOTIFICATIONS=true
ENV SHAMEBOT_DISCORD_GUILD=secret
ENV SHAMEBOT_DISCORD_TOKEN=secret
ENV SHAMEBOT_URL=http://to.do

EXPOSE 3030
CMD ["cron"]
