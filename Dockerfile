FROM ekidd/rust-musl-builder:1.59.0 as builder

# We need to add the source code to the image because `rust-musl-builder`
# assumes a UID of 1000, but TravisCI has switched to 2000.
ADD . ./
RUN sudo chown -R rust:rust .

RUN cargo build --release

FROM ubuntu:20.04
WORKDIR /usr/local/
RUN apt-get update -qq && apt-get install -y ca-certificates wget python libpython2.7
COPY --from=builder \
   /home/rust/src/target/x86_64-unknown-linux-musl/release/webar_assets \
   /usr/local/bin/


ENTRYPOINT /usr/local/bin/webar_assets
EXPOSE 3000