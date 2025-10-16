FROM scratch
WORKDIR /app
ENV HOST 0.0.0.0
COPY ./target/x86_64-unknown-linux-musl/release/waline-mini .
EXPOSE 8360
ENTRYPOINT ["./waline-mini"]
