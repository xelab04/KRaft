FROM registry.suse.com/bci/rust:1.85 AS build

WORKDIR /app

COPY . ./

RUN zypper -n in openssl-devel

RUN cargo build --release

FROM registry.suse.com/bci/bci-base:latest

RUN zypper -n in openssl-devel

COPY --from=build /app/target/release/auth-api /usr/local/bin/auth-api

EXPOSE 5000

CMD ["auth-api"]
