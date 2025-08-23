FROM registry.suse.com/bci/rust:1.85 as build

WORKDIR /app

COPY . ./

RUN zypper -n in openssl-devel

RUN cargo build --release

FROM registry.suse.com/bci/bci-micro:latest

COPY --from=build /app/target/release/auth-api /usr/local/bin/auth-api

CMD ["auth-api"]
