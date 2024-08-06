FROM rust:1.79-slim AS builder

WORKDIR /src/builder

ENV SQLX_OFFLINE=true
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12

WORKDIR /src/app

COPY --from=builder /src/builder/target/release/email-verifier .

CMD ["/src/app/email-verifier"]
