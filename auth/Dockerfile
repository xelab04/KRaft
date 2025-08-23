FROM registry.suse.com/bci/rust:1.85 as build

WORKDIR /app

COPY . ./

RUN cargo build --release

FROM registry.suse.com/bci/bci-micro:latest

COPY --from=build /app/target/release/auth-api /usr/local/bin/auth-api

CMD ["auth-api"]
