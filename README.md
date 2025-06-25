# RIFS - Rust图床服务

<div align="center">

![RIFS Logo](https://img.shields.io/badge/RIFS-Rust%20Image%20File%20Server-blue?style=for-the-badge&logo=rust)

<p>
  <img src="https://img.shields.io/badge/Rust-1.85+-orange.svg?style=flat-square" alt="Rust Version">
  <img src="https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square" alt="License">
  <img src="https://img.shields.io/badge/Status-Production%20Ready-green.svg?style=flat-square" alt="Status">
  <img src="https://img.shields.io/badge/Platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey.svg?style=flat-square" alt="Platform">
</p>

**🚀 高性能 Rust 图床服务 • 支持实时转换和智能缓存**

</div>

---

## ⚠️ 重要声明

<div align="center">

**🤖 本项目完全由 AI (Claude) 生成和编写 🤖**

**此项目包括所有代码、文档、配置文件等均为人工智能自动生成**  
**请在使用前仔细检查和测试，AI生成的代码可能存在潜在问题**  
**不建议直接用于生产环境，需要经过充分的测试和验证**

</div>

---

## ✨ 特性

- **高性能** - Rust编写，内存安全，高并发处理
- **多格式支持** - 支持JPEG、PNG、GIF、WebP、AVIF、ICO 6种主流图片格式
- **实时转换** - 通过URL参数进行图片尺寸、格式、质量转换
- **智能缓存** - 自动缓存转换结果，支持LRU清理策略
- **去重存储** - SHA256哈希去重，避免重复存储
- **管理面板** - 内置Web管理界面，支持缓存管理和系统监控

## 🏗️ 系统架构

```mermaid
flowchart TD
    %% 客户端
    Client["🌍 HTTP客户端<br/>Web/Mobile/API"]
    
    %% 接入层
    Nginx["🔄 Nginx反向代理<br/>负载均衡·SSL·缓存"]
    Server["🦀 RIFS服务器<br/>Rust + Axum框架"]
    
    %% Web框架层
    Middleware["🛡️ 中间件层<br/>CORS·日志·限流·认证"]
    Router["🚦 路由层<br/>RESTful API路由"]
    
    %% 处理器层 - 分开排列避免重叠
    ImageH["🖼️ ImageHandler<br/>图片上传·访问·转换"]
    CacheH["⚡ CacheHandler<br/>缓存管理·清理·统计"]
    HealthH["💚 HealthHandler<br/>健康检查·系统监控"]
    StaticH["📁 StaticHandler<br/>静态资源·管理面板"]
    
    %% 服务层 - 分层排列
    ImageS["📸 ImageService<br/>图片业务逻辑"]
    TransformS["🔄 TransformService<br/>格式转换·尺寸调整"]
    CacheS["🧠 CacheService<br/>智能缓存策略"]
    
    %% 工具层
    Utils["🛠️ FormatUtils<br/>格式检测·验证"]
    Transform["⚙️ StaticTransform<br/>图像处理引擎"]
    
    %% 仓储层
    ImageRepo["📊 ImageRepository<br/>图片元数据管理"]
    CacheRepo["🗃️ CacheRepository<br/>缓存索引管理"]
    BaseRepo["🏛️ BaseRepository<br/>通用数据访问"]
    
    %% 数据存储
    SQLite[("🗃️ SQLite<br/>默认轻量级数据库")]
    PostgreSQL[("🐘 PostgreSQL<br/>高性能生产数据库")]
    MySQL[("🐬 MySQL<br/>兼容性数据库")]
    
    %% 文件存储
    Uploads["📤 原图存储<br/>uploads/目录<br/>SHA256分层"]
    Cache["⚡ 缓存存储<br/>cache/目录<br/>转换结果"]
    
    %% 状态管理
    AppState["🌟 AppState<br/>全局状态管理器"]
    DBPool["🏊 DatabasePool<br/>数据库连接池"]
    Config["⚙️ AppConfig<br/>配置热加载管理"]
    
    %% 垂直主流程 - 避免交叉
    Client --> Nginx
    Nginx --> Server
    Server --> Middleware
    Middleware --> Router
    
    %% 路由到处理器 - 分散连接
    Router --> ImageH
    Router --> CacheH
    Router --> HealthH
    Router --> StaticH
    
    %% 处理器到服务层 - 明确分工
    ImageH --> ImageS
    ImageH --> TransformS
    CacheH --> CacheS
    
    %% 服务层到工具层 - 水平连接
    ImageS --> Utils
    TransformS --> Transform
    
    %% 服务层到仓储层 - 直接对应
    ImageS --> ImageRepo
    CacheS --> CacheRepo
    
    %% 仓储继承关系
    ImageRepo --> BaseRepo
    CacheRepo --> BaseRepo
    
    %% 数据存储连接 - 分开避免重叠
    BaseRepo --> SQLite
    BaseRepo --> PostgreSQL
    BaseRepo --> MySQL
    
    %% 文件存储连接 - 独立路径
    ImageS -.-> Uploads
    CacheS -.-> Cache
    
    %% 状态管理连接 - 侧边路径
    AppState --> DBPool
    AppState --> Config
    DBPool -.-> BaseRepo
    
    %% 样式定义 - 增强可读性
    style Client fill:#e3f2fd,stroke:#1976d2,stroke-width:3px
    style Nginx fill:#f1f8e9,stroke:#689f38,stroke-width:2px
    style Server fill:#fce4ec,stroke:#c2185b,stroke-width:3px
    
    style Middleware fill:#fff3e0,stroke:#f57c00,stroke-width:2px
    style Router fill:#fff3e0,stroke:#f57c00,stroke-width:2px
    
    style ImageH fill:#e8f5e8,stroke:#388e3c,stroke-width:2px
    style CacheH fill:#e8f5e8,stroke:#388e3c,stroke-width:2px
    style HealthH fill:#e8f5e8,stroke:#388e3c,stroke-width:2px
    style StaticH fill:#e8f5e8,stroke:#388e3c,stroke-width:2px
    
    style ImageS fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px
    style TransformS fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px
    style CacheS fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px
    
    style Utils fill:#e1f5fe,stroke:#0277bd,stroke-width:2px
    style Transform fill:#e1f5fe,stroke:#0277bd,stroke-width:2px
    
    style ImageRepo fill:#e0f2f1,stroke:#00695c,stroke-width:2px
    style CacheRepo fill:#e0f2f1,stroke:#00695c,stroke-width:2px
    style BaseRepo fill:#e0f2f1,stroke:#00695c,stroke-width:3px
    
    style SQLite fill:#fff8e1,stroke:#f9a825,stroke-width:2px
    style PostgreSQL fill:#fff8e1,stroke:#f9a825,stroke-width:2px
    style MySQL fill:#fff8e1,stroke:#f9a825,stroke-width:2px
    
    style Uploads fill:#fafafa,stroke:#424242,stroke-width:2px
    style Cache fill:#fafafa,stroke:#424242,stroke-width:2px
    
    style AppState fill:#e8eaf6,stroke:#3f51b5,stroke-width:3px
    style DBPool fill:#e8eaf6,stroke:#3f51b5,stroke-width:2px
    style Config fill:#e8eaf6,stroke:#3f51b5,stroke-width:2px
```

## 🚀 快速开始

### 本地运行

```bash
# 克隆项目
git clone https://github.com/djkcyl/rifs.git
cd rifs

# 运行
cargo run --release
```

### Docker 运行

```bash
docker run --rm --pull always -d \
  -p 3000:3000 \
  -v ./uploads:/app/uploads \
  -v ./cache:/app/cache \
  -v ./data:/app/data \
  -v ./config.toml:/app/config.toml \
  djkcyl/rifs:latest
```

## 📖 使用示例

### 上传图片

```bash
curl -F "file=@image.jpg" http://localhost:3000/upload
```

### 图片访问

```bash
# 原图
http://localhost:3000/images/a1b2c3d4...

# 转换 - 宽度800px
http://localhost:3000/images/a1b2c3d4...@w800

# 复杂转换 - 尺寸+格式+质量
http://localhost:3000/images/a1b2c3d4...@w800_h600_jpeg_q90
```

### 转换参数

| 参数 | 说明 | 示例 |
|------|------|------|
| `w{数字}` | 最大宽度 | `w800` |
| `h{数字}` | 最大高度 | `h600` |
| `{格式}` | 目标格式 | `jpeg`, `png`, `webp`, `avif`, `ico` |
| `q{数字}` | 质量1-100 | `q90` |
| `na[w/b/#hex]` | 去透明+背景色 | `naw`(白), `nab`(黑), `na#ff0000` |

## ⚙️ 配置

首次运行时会自动创建 `config.toml` 配置文件，包含所有配置项的详细说明。修改配置后重启服务即可生效。

也可以通过环境变量覆盖配置，格式为 `RIFS_` 前缀，如：
```bash
export RIFS_SERVER_PORT=8080
```

## 📊 管理面板

- **API文档**: http://localhost:3000/
- **缓存管理**: http://localhost:3000/cache/management

## 🖼️ 支持格式

| 格式 | 扩展名 | 读取 | 写入 | URL转换 | 质量控制 |
|------|--------|------|------|---------|----------|
| **JPEG** | .jpg, .jpeg | ✅ | ✅ | ✅ | ✅ |
| **PNG** | .png | ✅ | ✅ | ✅ | ✅ |
| **GIF** | .gif | ✅ | ✅ | ✅ | ❌ |
| **WebP** | .webp | ✅ | ✅ | ✅ | ✅ |
| **AVIF** | .avif | ✅ | ✅ | ✅ | ❌ |
| **ICO** | .ico | ✅ | ✅ | ✅ | ❌ |

### 转换能力说明

- ✅ **完全支持**: 可读取、写入、URL参数转换
- ❌ **仅存储**: 支持上传存储原图，不支持参数转换
- **动画处理**: GIF/WebP动画转换时自动提取第一帧
- **质量控制**: JPEG、PNG、WebP支持质量参数优化
- **智能压缩**: PNG根据质量参数智能选择压缩级别和滤波器

---

<div align="center">
Made with ❤️ and 🦀
</div> 