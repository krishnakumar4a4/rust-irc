FROM rust
#RUN curl https://sh.rustup.rs -sSf | sh
COPY * /rust-irc-server/
COPY src/* /rust-irc-server/src/
WORKDIR /rust-irc-server/
RUN rustup override set nightly
RUN rustup update && cargo update
RUN ls src
ENTRYPOINT cargo run
