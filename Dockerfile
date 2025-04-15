FROM clux/muslrust:1.85.0-stable-2025-03-18 AS chef

WORKDIR /app
# 写入镜像源配置
RUN cat <<EOF > /root/.cargo/config.toml
[source.crates-io]
replace-with = "rsproxy-sparse"

[source.rsproxy]
registry = "https://rsproxy.cn/crates.io-index"

[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"

[registries.rsproxy]
index = "https://rsproxy.cn/crates.io-index"

[net]
git-fetch-with-cli = true

[target.x86_64-unknown-linux-musl]
rustflags = ["-C", "target-feature=+crt-static"]
EOF

RUN cargo install cargo-chef --locked

ENV PKG_CONFIG_ALL_STATIC=1
ENV OPENSSL_STATIC=1

FROM chef AS planner
COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

# 构建应用程序
COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl --bin bloglite
RUN cargo build --release --target x86_64-unknown-linux-musl --bin refresh_token

FROM alpine:latest AS runtime
WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/bloglite ./
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/refresh_token ./
COPY --from=builder /app/bins/bloglite/sql/prod_initial/create-schema.sql ./sql/prod_initial/create-schema.sql

# 设置默认运行的二进制
ENTRYPOINT ["./bloglite"]