FROM scratch
WORKDIR /app
ENV HOST 0.0.0.0
COPY --chmod=755 ./target/x86_64-unknown-linux-musl/release/waline-mini .
EXPOSE 8360
ENTRYPOINT ["./waline-mini"]
