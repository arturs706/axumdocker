FROM rust:1.65 as build


WORKDIR /src/dockerserver
COPY . .

RUN cargo build --release


FROM gcr.io/distroless/cc-debian10

COPY --from=build /src/dockerserver/target/release/dockerserver /usr/local/bin/dockerserver

WORKDIR /usr/local/bin

CMD ["dockerserver"]