FROM rust:buster as rust_builder

WORKDIR /usr/src

RUN USER=root cargo new --bin capybara

WORKDIR /usr/src/capybara

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN cargo build --release
RUN rm ./target/release/deps/capybara*
RUN rm src/*.rs

COPY ./src ./src
RUN cargo build --release

FROM debian:buster-slim as runner

WORKDIR /usr/src/capybara

RUN apt-get update
RUN apt-get install libopus0
RUN apt-get install -y --no-install-recommends ffmpeg
RUN apt-get install -y python-pip
RUN pip install --upgrade youtube-dl

COPY --from=rust_builder /usr/src/capybara/target/release/capybara ./capybara
COPY .env ./.env

CMD ./capybara