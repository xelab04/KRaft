FROM registry.suse.com/bci/rust:1.85 as build

WORKDIR /app

COPY . ./

RUN cargo build --release

# Bundle the application into a BCI micro (or other BCI image)
FROM registry.suse.com/bci/bci-micro:latest

COPY --from=build /app/target/release/cluster-manage /usr/local/bin/cluster-manage

CMD ["cluster-manage"]
