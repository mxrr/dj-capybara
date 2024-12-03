FROM rust:bookworm AS rust_builder
ARG GIT_COMMIT="unknown"

WORKDIR /usr/src

RUN USER=root cargo new --bin capybara

WORKDIR /usr/src/capybara

RUN apt-get update
RUN apt-get install -y --no-install-recommends cmake

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./build.rs ./build.rs

ENV GIT_COMMIT=$GIT_COMMIT

RUN cargo build --release
RUN rm ./target/release/deps/capybara*
RUN rm src/*.rs

COPY ./src ./src
RUN cargo build --release

FROM debian:bookworm-slim AS runner

WORKDIR /usr/src/capybara

RUN apt-get update
RUN apt-get install -y ca-certificates
RUN apt-get install libopus0
RUN apt-get install -y --no-install-recommends curl
RUN apt-get install -y --no-install-recommends python3

RUN curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp
RUN chmod a+rx /usr/local/bin/yt-dlp

COPY --from=rust_builder /usr/src/capybara/target/release/capybara ./capybara

CMD ["./capybara"]
