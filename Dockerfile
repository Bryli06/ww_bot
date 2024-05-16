FROM lukemathwalker/cargo-chef:latest-rust-alpine as chef
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
COPY ./.cargo/config.toml ./.cargo/config.toml
COPY ./.sqlx ./.sqlx
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY . .
RUN cargo build --release
RUN mv ./target/release/ww_bot ./app

FROM scratch AS runtime
WORKDIR /app
COPY --from=builder /app/app /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/app"]
