FROM rust:1.83 AS builder

WORKDIR /src/builder

RUN apt-get update \
    && apt-get install -y ldd gcc

ENV SQLX_OFFLINE=true
COPY . .
RUN cargo build --release

FROM debian

WORKDIR /src/app

RUN apt-get update \
    && apt-get install -y ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/builder/target/release/*.so .
COPY --from=builder /src/builder/target/release/email-verifier .

CMD ["/src/app/email-verifier"]
