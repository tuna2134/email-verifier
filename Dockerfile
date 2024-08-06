FROM rust:1.79-slim AS builder

WORKDIR /src/builder

COPY . .
RUN --mount=type=cache,target=/src/builder/target/ cargo build --release && \
    cp /src/builder/target/release/email-verifier /tmp/email-verifier

FROM gcr.io/distroless/cc-debian12

WORKDIR /src/app

COPY --from=builder /tmp/email-verifier .

CMD ["/src/app/cloud-shell"]
