# 图床服务 (Image Bed Service)

一个基于 Rust 和 Axum 框架构建的高性能图床服务，支持图片上传、存储和获取功能。

## 功能特性

- 图片上传（支持多种图片格式：JPEG/JPG、PNG、GIF、WebP、BMP、TIFF）
- 图片获取和管理（获取原图、查询信息、删除）
- 分层文件存储（基于UUID自动分目录）
- 类型验证、大小限制、CORS支持
- 配置文件和环境变量支持

## 安装与运行

```bash
# 构建项目
cargo build --release

# 运行服务
cargo run
```

服务默认在 `http://0.0.0.0:3000` 启动。

## API 接口

| 接口 | 方法 | 描述 |
|------|------|------|
| `/health` | GET | 健康检查 |
| `/upload` | POST | 上传图片（multipart/form-data, 字段名"file"） |
| `/images/{filename}` | GET | 获取原图 |
| `/images/{filename}/info` | GET | 获取图片信息 |
| `/images/{filename}` | DELETE | 删除图片 |

## 配置说明

配置文件：`config.toml`，支持环境变量覆盖（`IMAGE_BED_` 前缀）。

```toml
[server]
host = "0.0.0.0"
port = 3000
enable_cors = true

[storage]
# 文件将按UUID前2位和第3-4位分层存储
# 例如：uploads/55/0e/550e8400-e29b-41d4-a716-446655440000.jpg
upload_dir = "uploads"
max_file_size = 10485760  # 10MB
supported_types = ["image/jpeg", "image/jpg", "image/png", "image/gif", "image/webp", "image/bmp", "image/tiff"]

[logging]
level = "info"

[cache]
max_age = 31536000  # 1年
```

## 项目结构

```
image-bed/
├── src/          # 源码目录
├── uploads/      # 图片存储目录（按UUID分层）
├── config.toml   # 配置文件
└── README.md     # 项目说明
```

## 许可证

MIT License 