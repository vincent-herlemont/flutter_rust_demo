FROM rust:1.71-bookworm as builder

ARG CRATE_NAME
ENV CARGO_TARGET_DIR=/target

WORKDIR /src

RUN --mount=type=bind,source=.,target=/src \
    --mount=type=cache,target=/target \
	--mount=type=cache,target=/usr/local/cargo/registry \
	set -eux; \
	cargo build --release -p ${CRATE_NAME}; \
	objcopy --compress-debug-sections /target/release/${CRATE_NAME} /usr/local/bin/service

FROM debian:bookworm-slim

ENV RUST_LIB_BACKTRACE=1
ENV RUST_BACKTRACE=1
#ENV RUST_BACKTRACE=full

COPY --from=builder /usr/local/bin/service /usr/local/bin/service

CMD COLORBT_SHOW_HIDDEN=1 /usr/local/bin/service
