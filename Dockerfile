FROM rust:1.58-alpine as builder

WORKDIR /volume

RUN apk add --no-cache build-base=~0.5 musl-dev=~1.2 perl=~5.32

WORKDIR /volume

COPY migrations/ migrations/
COPY src/ src/
COPY templates/ templates/
COPY Cargo.lock Cargo.toml ./

RUN cargo build --release && \
    strip --strip-all target/release/asgard

FROM alpine:3.15 as newuser

RUN echo "asgard:x:1000:" > /tmp/group && \
    echo "asgard:x:1000:1000::/dev/null:/sbin/nologin" > /tmp/passwd

FROM scratch

COPY --from=builder /volume/target/release/asgard /bin/
COPY --from=newuser /tmp/group /tmp/passwd /etc/

EXPOSE 8080
STOPSIGNAL SIGINT
USER asgard

ENTRYPOINT ["/bin/asgard"]
