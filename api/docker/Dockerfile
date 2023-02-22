FROM rust:1.66 AS builder

WORKDIR /usr/src
COPY ./database ./database
COPY ./discord ./discord
COPY ./utils ./utils
COPY ./api ./api
RUN cargo install --path ./api

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/api /usr/local/bin/api

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

ENV POSTGRES_HOST=database
ENV POSTGRES_PORT=5432
ENV POSTGRES_USERNAME=shamebot
ENV POSTGRES_PASSWORD=secret
ENV POSTGRES_DB=shamebot
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000
ENV SHAMEBOT_DISCORD_TOKEN=secret
ENV SHAMEBOT_URL=http://to.do
ENV SHAMEBOT_CLIENT_ID=secret
ENV SHAMEBOT_CLIENT_SECRET=secret
ENV SHAMEBOT_REDIRECT_URI=http://localhost:8000/discord/authorize

EXPOSE 8000
CMD ["api"]
