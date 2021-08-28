FROM rust:1.54-alpine as builder

RUN apk add musl-dev openssl-dev

WORKDIR /build
ADD . .

RUN cargo build --release
RUN cargo install --path $PWD

FROM alpine:3.14
COPY --from=builder /usr/local/cargo/bin/email-verification-service /usr/bin/email-verification-service

CMD email-verification-service