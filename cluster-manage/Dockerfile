FROM registry.suse.com/bci/rust:1.85 AS build

WORKDIR /app

COPY . ./

RUN zypper -n in openssl-devel

RUN cargo build --release

FROM registry.suse.com/bci/bci-base:latest

RUN zypper -n in openssl-devel

COPY --from=build /app/target/release/cluster-manage /usr/local/bin/cluster-manage

CMD ["cluster-manage"]
