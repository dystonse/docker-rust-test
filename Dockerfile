FROM rust:1.41 as builder
WORKDIR /usr/src/myapp
RUN apt-get update && apt-get install -y protobuf-compiler
COPY ./Cargo.* ./
RUN mkdir src && echo "fn main() { println!(\"Hello, world!\"); }" > src/main.rs
RUN cargo fetch
RUN cargo build --release
RUN rm src/main.rs 
COPY . .
RUN touch src/main.rs
RUN cargo build --release
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y libssl1.1
COPY --from=builder /usr/local/cargo/bin/dystonse-gtfs-importer /usr/local/bin/dystonse-gtfs-importer
WORKDIR /
CMD ["dystonse-gtfs-importer","automatic","/files/$GTFS_ID/"]
