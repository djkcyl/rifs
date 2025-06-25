# 多阶段构建 Dockerfile for RIFS
# 第一阶段：构建环境
FROM rust:1.87-slim AS builder

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /app

# 复制项目文件
COPY Cargo.toml ./
COPY src/ ./src/

# 构建应用程序（Release模式）
RUN cargo build --release

# 第二阶段：运行环境
FROM debian:bookworm-slim AS runtime

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# 创建非root用户
RUN useradd -m -u 1000 rifs

# 设置工作目录
WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/rifs /app/rifs

# 创建必要的目录
RUN mkdir -p /app/uploads /app/cache /app/data \
    && chown -R rifs:rifs /app

# 设置环境变量标识容器环境
ENV CONTAINER=true

# 切换到非root用户
USER rifs

# 暴露端口
EXPOSE 3000

# 启动应用
CMD ["./rifs"] 