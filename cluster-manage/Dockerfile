FROM registry.suse.com/bci/rust:1.93 AS build

WORKDIR /app

COPY . ./

RUN zypper -n in openssl-devel

RUN cargo build --release

FROM registry.suse.com/bci/bci-base:latest

RUN zypper -n in openssl-devel wget

# RUN wget -qO k3kcli https://github.com/rancher/k3k/releases/download/v0.3.3/k3kcli-linux-amd64 && \
#   chmod +x k3kcli && \
#   mv k3kcli /usr/local/bin

COPY --from=build /app/target/release/cluster-manage /usr/local/bin/cluster-manage

CMD ["cluster-manage"]
