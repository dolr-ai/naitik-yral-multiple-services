FROM scratch
WORKDIR /app

COPY ./target/x86_64-unknown-linux-musl/release/naitik-yral-multiple-services .
COPY ./config.toml .

ENV RUST_LOG="debug"
ENV BIND_ADDRESS="0.0.0.0:6000"
ENV APP_ENV="development"
EXPOSE 6000

CMD ["./naitik-yral-multiple-services"]