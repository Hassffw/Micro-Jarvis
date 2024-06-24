FROM rust:1.78.0 as builder
WORKDIR /usr/src/perplexitytelegramassistant
COPY . .
RUN apt-get update && apt-get install -y libpq-dev
RUN cargo build --release
RUN cargo install diesel_cli --no-default-features --features postgres --locked

FROM ubuntu:22.04
RUN apt-get update && apt-get install -y libpq-dev ca-certificates postgresql-client && rm -rf /var/lib/apt/lists/*
WORKDIR /app
RUN mkdir -p /app/data
COPY --from=builder /usr/src/perplexitytelegramassistant/target/release/perplexitytelegramassistant /usr/local/bin/
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel
COPY --from=builder /usr/src/perplexitytelegramassistant/migrations ./migrations
COPY --from=builder /usr/src/perplexitytelegramassistant/diesel.toml .
COPY start.sh .
RUN chmod +x start.sh
CMD ["./start.sh"]
