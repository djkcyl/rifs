# RIFS - Rust Image File Service

一个基于 Rust 和 Axum 框架构建的高性能图床服务。

## 功能特性

- 图片上传、获取、删除（支持 JPEG、PNG、GIF、WebP、BMP、TIFF）
- 基于SHA256哈希的分层文件存储和自动去重
- 安全的文件类型检测和大小限制
- 多数据库支持（SQLite、PostgreSQL、MySQL）
- CORS支持、缓存优化
- 配置文件和环境变量支持

## 快速开始

```bash
# 复制配置文件
cp config.example.toml config.toml

# 运行服务（开发模式）
cargo run

# 多平台一键构建
./build.sh

# 单独构建特定平台
./build-linux.sh       # Linux版本
.\build.ps1             # Windows版本（在Windows上）
./build-windows.sh      # Windows版本（Linux交叉编译）
```

服务默认在 `http://localhost:3000` 启动。

## API 接口

| 接口 | 方法 | 描述 |
|------|------|------|
| `/health` | GET | 健康检查 |
| `/upload` | POST | 上传图片 (multipart/form-data，字段名"file") |
| `/images/{filename}` | GET | 获取图片（支持哈希或文件名） |
| `/images/{filename}/info` | GET | 获取图片信息（支持哈希或文件名） |
| `/images/{filename}` | DELETE | 删除图片（支持哈希或文件名） |
| `/api/images/query` | POST | 查询图片列表（支持分页、过滤、排序） |
| `/api/stats` | GET | 获取存储统计信息 |

## 配置说明

配置文件：`config.toml`，支持环境变量覆盖（`IMAGE_BED_` 前缀）。

```toml
[server]
host = "0.0.0.0"
port = 3000
enable_cors = true

[storage]
upload_dir = "uploads"
max_file_size = 10485760  # 10MB

[database]
database_type = "sqlite"  # 支持 sqlite/postgres/mysql
connection_string = "sqlite:./data/images.db"
max_connections = 20

[logging]
level = "info"
enable_color = true

[cache]
max_age = 31536000  # 1年
```

## 项目结构

```
rifs/
├── src/
│   ├── config.rs          # 配置管理
│   ├── entities/          # 数据库实体模型
│   ├── handlers/          # API处理器
│   ├── migrations/        # 数据库迁移
│   ├── models/            # 业务数据模型
│   ├── services/          # 业务服务
│   ├── utils/             # 工具函数
│   └── main.rs            # 入口文件
├── build/                # 构建输出目录（自动生成）
├── config.toml           # 配置文件
├── build.sh              # 多平台一键构建脚本
├── build-linux.sh        # Linux 构建脚本
├── build.ps1             # Windows PowerShell 构建脚本
├── build-windows.sh      # Linux 交叉编译 Windows 版本
└── uploads/              # 存储目录（自动创建）
```

## 存储结构

文件按SHA256哈希前4位分层存储：`uploads/a1/b2/a1b2c3d4.jpg`
- 自动去重：相同文件只存储一份
- 基于内容哈希的文件名，避免冲突
- 支持通过哈希值或文件名访问

### 多平台支持

| 平台 | 架构 | 状态 |
|------|------|------|
| **Linux** | x64/ARM64 | ✅ 支持 |
| **Windows** | x64/x86 | ✅ 支持 |
| **macOS** | x64/ARM64 | 🍎 原生构建 |

### 构建

- **Linux/Windows**: `./build.sh`
- **macOS**: `./build-macos.sh`

## 许可证

MIT License 