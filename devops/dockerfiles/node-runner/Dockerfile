FROM mangatasolutions/node-builder-local:latest as bytehound
RUN git clone --depth 1 --branch 0.8.0 https://github.com/koute/memory-profiler /tmp/profiler
RUN cargo build --manifest-path /tmp/profiler/Cargo.toml --release -p bytehound-preload -p bytehound-cli

FROM debian:buster
COPY --from=bytehound /tmp/profiler/target/release/bytehound /usr/bin/bytehound
COPY --from=bytehound /tmp/profiler/target/release/libbytehound.so /usr/lib/libbytehound.so
RUN mkdir /mangata
