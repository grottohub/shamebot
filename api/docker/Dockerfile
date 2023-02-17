FROM rust:1.66 AS builder

WORKDIR /usr/src
COPY ./database ./database
COPY ./utils ./utils
COPY ./api ./api
RUN cargo install --path ./api

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/api /usr/local/bin/api

ENV POSTGRES_HOST=database
ENV POSTGRES_PORT=5432
ENV POSTGRES_USERNAME=shamebot
ENV POSTGRES_PASSWORD=secret
ENV POSTGRES_DB=shamebot
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

EXPOSE 8000
CMD ["api"]
