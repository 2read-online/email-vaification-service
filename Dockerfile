FROM ubuntu:21.10 as builder
ENV DEBIAN_FRONTEND=noninteractive

RUN apt update && apt install -y cargo libssl-dev pkg-config

WORKDIR /build
ADD . .
RUN cargo install --path $PWD

FROM ubuntu:21.10

RUN apt update && apt install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /root/.cargo/bin/email-verification-service /usr/bin/email-verification-service
CMD email-verification-service
