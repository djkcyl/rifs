use axum::{
    http::StatusCode,
    response::IntoResponse,
};

/// 内嵌的HTML文档内容
pub const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RIFS - 图床服务</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            line-height: 1.6;
            color: #e2e8f0;
            background: linear-gradient(135deg, #0f172a 0%, #1e293b 50%, #334155 100%);
            min-height: 100vh;
            padding: 20px;
        }
        
        .container {
            max-width: 1000px;
            margin: 0 auto;
        }
        
        .header {
            text-align: center;
            margin-bottom: 40px;
        }
        
        .header h1 {
            font-size: 2.8rem;
            font-weight: 800;
            margin-bottom: 15px;
            background: linear-gradient(135deg, #06b6d4, #3b82f6, #8b5cf6);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
            text-shadow: none;
        }
        
        .header p {
            font-size: 1.2rem;
            color: #94a3b8;
            font-weight: 300;
        }
        
        .card {
            background: rgba(30, 41, 59, 0.8);
            backdrop-filter: blur(20px);
            border-radius: 16px;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
            margin-bottom: 30px;
            overflow: hidden;
            border: 1px solid rgba(148, 163, 184, 0.2);
        }
        
        .card-header {
            background: linear-gradient(135deg, #06b6d4, #3b82f6);
            color: white;
            padding: 24px 28px;
            font-size: 1.3rem;
            font-weight: 700;
            letter-spacing: 0.5px;
        }
        
        .card-content {
            padding: 28px;
        }
        
                 .features-grid {
             display: grid;
             grid-template-columns: repeat(2, 1fr);
             gap: 20px;
             margin-bottom: 25px;
         }
         
         @media (max-width: 768px) {
             .features-grid {
                 grid-template-columns: 1fr;
             }
         }
        
        .feature {
            display: flex;
            align-items: center;
            padding: 20px;
            background: rgba(15, 23, 42, 0.6);
            border-radius: 12px;
            border: 1px solid rgba(6, 182, 212, 0.3);
            transition: all 0.3s ease;
            backdrop-filter: blur(10px);
        }
        
        .feature:hover {
            transform: translateY(-2px);
            border-color: rgba(6, 182, 212, 0.6);
            box-shadow: 0 8px 25px rgba(6, 182, 212, 0.15);
        }
        
        .feature-icon {
            width: 45px;
            height: 45px;
            background: linear-gradient(135deg, #06b6d4, #0891b2);
            border-radius: 12px;
            display: flex;
            align-items: center;
            justify-content: center;
            color: white;
            margin-right: 18px;
            font-weight: bold;
            font-size: 1.2rem;
        }
        
        .endpoints {
            display: grid;
            gap: 16px;
        }
        
        .endpoint {
            border: 1px solid rgba(148, 163, 184, 0.2);
            border-radius: 12px;
            overflow: hidden;
            transition: all 0.3s ease;
            background: rgba(15, 23, 42, 0.4);
            backdrop-filter: blur(10px);
        }
        
        .endpoint:hover {
            transform: translateY(-2px);
            box-shadow: 0 12px 25px rgba(0,0,0,0.2);
            border-color: rgba(6, 182, 212, 0.5);
        }
        
        .endpoint-header {
            background: rgba(30, 41, 59, 0.8);
            padding: 18px 24px;
            display: flex;
            align-items: center;
            gap: 18px;
            border-bottom: 1px solid rgba(148, 163, 184, 0.15);
        }
        
        .method {
            padding: 8px 14px;
            border-radius: 8px;
            font-weight: 700;
            font-size: 0.75rem;
            text-transform: uppercase;
            letter-spacing: 1px;
        }
        
        .method.get { background: linear-gradient(135deg, #10b981, #059669); color: white; }
        .method.post { background: linear-gradient(135deg, #3b82f6, #2563eb); color: white; }
        .method.delete { background: linear-gradient(135deg, #ef4444, #dc2626); color: white; }
        
        .path {
            font-family: 'Monaco', 'Courier New', monospace;
            font-size: 1rem;
            color: #06b6d4;
            background: rgba(6, 182, 212, 0.1);
            padding: 8px 12px;
            border-radius: 8px;
            flex: 1;
            font-weight: 500;
        }
        
        .endpoint-content {
            padding: 24px;
        }
        
        .description {
            color: #cbd5e1;
            margin-bottom: 10px;
            font-size: 0.95rem;
        }
        
        .upload-section {
            background: rgba(15, 23, 42, 0.6);
            border-radius: 16px;
            padding: 35px;
            text-align: center;
            border: 1px solid rgba(6, 182, 212, 0.2);
            backdrop-filter: blur(10px);
        }
        
        .upload-form {
            max-width: 400px;
            margin: 0 auto;
        }
        
        .file-input-wrapper {
            position: relative;
            margin: 25px 0;
        }
        
        .file-input {
            display: none;
        }
        
        .file-label {
            display: block;
            padding: 35px;
            border: 2px dashed rgba(6, 182, 212, 0.5);
            border-radius: 16px;
            cursor: pointer;
            transition: all 0.3s ease;
            background: rgba(30, 41, 59, 0.4);
            backdrop-filter: blur(10px);
        }
        
        .file-label:hover,
        .file-label.drag-over {
            border-color: #06b6d4;
            background: rgba(6, 182, 212, 0.1);
            transform: translateY(-2px);
            box-shadow: 0 12px 25px rgba(6, 182, 212, 0.15);
        }
        
        .upload-icon {
            font-size: 3rem;
            margin-bottom: 15px;
            color: #06b6d4;
        }
        
        .btn {
            background: linear-gradient(135deg, #06b6d4, #3b82f6);
            color: white;
            border: none;
            padding: 16px 32px;
            font-size: 1rem;
            font-weight: 600;
            border-radius: 12px;
            cursor: pointer;
            transition: all 0.3s ease;
            text-transform: uppercase;
            letter-spacing: 1px;
            box-shadow: 0 4px 12px rgba(6, 182, 212, 0.3);
        }
        
        .btn:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 25px rgba(6, 182, 212, 0.4);
            background: linear-gradient(135deg, #0891b2, #2563eb);
        }
        
        .btn:active {
            transform: translateY(0);
        }
        
        .storage-info {
            background: rgba(15, 23, 42, 0.6);
            border-radius: 12px;
            padding: 20px;
            border: 1px solid rgba(6, 182, 212, 0.2);
            backdrop-filter: blur(10px);
        }
        
        .storage-info h4 {
            color: #06b6d4;
            margin-bottom: 12px;
            font-size: 1.1rem;
        }
        
        .storage-info p {
            color: #cbd5e1;
            margin-bottom: 8px;
            font-size: 0.9rem;
        }
        
        .storage-info code {
            background: rgba(6, 182, 212, 0.1);
            color: #06b6d4;
            padding: 2px 6px;
            border-radius: 4px;
            font-family: 'Monaco', 'Courier New', monospace;
            font-size: 0.85rem;
        }
        
        @media (max-width: 768px) {
            body {
                padding: 10px;
            }
            
            .header h1 {
                font-size: 2.2rem;
            }
            
            .endpoint-header {
                flex-direction: column;
                align-items: flex-start;
                gap: 10px;
            }
            
            .path {
                width: 100%;
            }
        }
        
        /* 格式支持样式 */
        .format-table {
            display: grid;
            gap: 20px;
        }
        
        .format-category {
            background: rgba(15, 23, 42, 0.6);
            border-radius: 12px;
            padding: 20px;
            border: 1px solid rgba(148, 163, 184, 0.2);
        }
        
        .format-category h4 {
            color: #06b6d4;
            margin-bottom: 15px;
            font-size: 1.1rem;
            font-weight: 600;
        }
        
        .format-list {
            display: flex;
            flex-wrap: wrap;
            gap: 8px;
        }
        
        .format-item {
            padding: 8px 12px;
            border-radius: 8px;
            font-size: 0.85rem;
            font-weight: 500;
            border: 1px solid rgba(148, 163, 184, 0.3);
            transition: all 0.3s ease;
        }
        
        .format-item.supported {
            background: rgba(16, 185, 129, 0.1);
            color: #10b981;
            border-color: rgba(16, 185, 129, 0.4);
        }
        
        .format-item.decode-only {
            background: rgba(251, 146, 60, 0.1);
            color: #f59e0b;
            border-color: rgba(251, 146, 60, 0.4);
        }
        
        .format-item:hover {
            transform: translateY(-1px);
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        }
        
        .format-notes {
            margin-top: 25px;
            padding: 20px;
            background: rgba(15, 23, 42, 0.8);
            border-radius: 12px;
            border-left: 4px solid #06b6d4;
        }
        
        .format-notes p {
            color: #06b6d4;
            font-weight: 600;
            margin-bottom: 12px;
        }
        
        .format-notes ul {
            color: #cbd5e1;
            line-height: 1.6;
        }
        
        .format-notes li {
            margin-bottom: 8px;
        }
        
        @media (max-width: 768px) {
            .format-category {
                padding: 15px;
            }
            
            .format-item {
                font-size: 0.8rem;
                padding: 6px 10px;
            }
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>RIFS</h1>
            <p>Rust 图床服务 - 高性能、安全、易用</p>
        </div>

        <div class="card">
            <div class="card-header">核心特性</div>
            <div class="card-content">
                <div class="features-grid">
                    <div class="feature">
                        <div class="feature-icon">🖼️</div>
                        <div>
                                                <strong style="color: #f1f5f9; font-size: 1.1rem;">多格式支持</strong><br>
                    <small style="color: #94a3b8;">JPEG, PNG, GIF, WebP, AVIF, ICO</small>
                        </div>
                    </div>
                    <div class="feature">
                        <div class="feature-icon">🔒</div>
                        <div>
                            <strong style="color: #f1f5f9; font-size: 1.1rem;">智能去重</strong><br>
                            <small style="color: #94a3b8;">SHA256 哈希自动去重</small>
                        </div>
                    </div>
                                         <div class="feature">
                         <div class="feature-icon">🦀</div>
                         <div>
                             <strong style="color: #f1f5f9; font-size: 1.1rem;">Rust 驱动</strong><br>
                             <small style="color: #94a3b8;">内存安全、零成本抽象、极致性能</small>
                         </div>
                     </div>
                     <div class="feature">
                         <div class="feature-icon">⚡</div>
                         <div>
                             <strong style="color: #f1f5f9; font-size: 1.1rem;">高性能</strong><br>
                             <small style="color: #94a3b8;">异步并发、超低延迟</small>
                         </div>
                     </div>
                     <div class="feature">
                         <div class="feature-icon">🗄️</div>
                         <div>
                             <strong style="color: #f1f5f9; font-size: 1.1rem;">智能缓存</strong><br>
                             <small style="color: #94a3b8;">转换结果缓存、热度评分、LRU清理</small>
                         </div>
                     </div>
                     <div class="feature">
                         <div class="feature-icon">🧹</div>
                         <div>
                             <strong style="color: #f1f5f9; font-size: 1.1rem;">自动清理</strong><br>
                             <small style="color: #94a3b8;">基于年龄、大小、访问频率的智能清理</small>
                         </div>
                     </div>
                </div>
                
                <div class="storage-info">
                    <h4>存储架构</h4>
                    <p><strong>分层存储:</strong> 文件按 SHA256 哈希前4位分层存储</p>
                    <p><strong>示例路径:</strong> <code>uploads/a1/b2/a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456.jpg</code></p>
                    <p><strong>智能去重:</strong> 相同文件只存储一份，节省存储空间</p>
                </div>
            </div>
        </div>

        <div class="card">
            <div class="card-header">API 接口</div>
            <div class="card-content">
                <div class="endpoints">
                    <div class="endpoint">
                        <div class="endpoint-header">
                            <span class="method get">GET</span>
                            <span class="path">/</span>
                        </div>
                        <div class="endpoint-content">
                            <div class="description">API文档页面</div>
                        </div>
                    </div>

                    <div class="endpoint">
                        <div class="endpoint-header">
                            <span class="method get">GET</span>
                            <span class="path">/health</span>
                        </div>
                        <div class="endpoint-content">
                            <div class="description">健康检查接口</div>
                        </div>
                    </div>

                    <div class="endpoint">
                        <div class="endpoint-header">
                            <span class="method post">POST</span>
                            <span class="path">/upload</span>
                        </div>
                        <div class="endpoint-content">
                            <div class="description">上传图片文件 (multipart/form-data, field: file)</div>
                        </div>
                    </div>

                    <div class="endpoint">
                        <div class="endpoint-header">
                            <span class="method get">GET</span>
                            <span class="path">/images/{identifier}[@params]</span>
                        </div>
                        <div class="endpoint-content">
                            <div class="description">获取图片文件 (通过哈希值)</div>
                            <div style="margin-top: 12px;">
                                <strong style="color: #06b6d4;">🎯 实时转换功能</strong><br>
                                <small style="color: #94a3b8;">在文件名后添加 @ 参数即可实现实时转换</small>
                            </div>
                            <div style="margin-top: 8px; font-family: 'Monaco', 'Courier New', monospace; font-size: 0.85rem; background: rgba(6, 182, 212, 0.1); padding: 8px; border-radius: 6px;">
                                <strong>示例:</strong> /images/abc123@w800_h600_jpeg_q90<br>
                                <strong>参数:</strong> w宽度_h高度_格式_na去透明_q质量
                            </div>
                        </div>
                    </div>

                    <div class="endpoint">
                        <div class="endpoint-header">
                            <span class="method get">GET</span>
                            <span class="path">/images/{identifier}/info</span>
                        </div>
                        <div class="endpoint-content">
                            <div class="description">获取图片元数据信息 (通过哈希值，JSON格式)</div>
                        </div>
                    </div>

                    <div class="endpoint">
                        <div class="endpoint-header">
                            <span class="method delete">DELETE</span>
                            <span class="path">/images/{identifier}</span>
                        </div>
                        <div class="endpoint-content">
                            <div class="description">删除图片文件 (通过哈希值，同时清理相关缓存)</div>
                        </div>
                    </div>

                    <div class="endpoint">
                        <div class="endpoint-header">
                            <span class="method get">GET</span>
                            <span class="path">/cache/management</span>
                        </div>
                        <div class="endpoint-content">
                            <div class="description">缓存管理面板 (可视化缓存管理界面)</div>
                        </div>
                    </div>

                    <div class="endpoint">
                        <div class="endpoint-header">
                            <span class="method get">GET</span>
                            <span class="path">/api/cache/stats</span>
                        </div>
                        <div class="endpoint-content">
                            <div class="description">获取缓存统计信息 (JSON格式)</div>
                        </div>
                    </div>

                    <div class="endpoint">
                        <div class="endpoint-header">
                            <span class="method post">POST</span>
                            <span class="path">/api/cache/cleanup/auto</span>
                        </div>
                        <div class="endpoint-content">
                            <div class="description">触发智能自动缓存清理</div>
                        </div>
                    </div>

                    <div class="endpoint">
                        <div class="endpoint-header">
                            <span class="method post">POST</span>
                            <span class="path">/api/cache/cleanup/policy</span>
                        </div>
                        <div class="endpoint-content">
                            <div class="description">根据自定义策略清理缓存 (JSON参数)</div>
                        </div>
                    </div>

                    <div class="endpoint">
                        <div class="endpoint-header">
                            <span class="method delete">DELETE</span>
                            <span class="path">/api/cache/clear</span>
                        </div>
                        <div class="endpoint-content">
                            <div class="description">清空所有缓存 (⚠️ 危险操作)</div>
                        </div>
                    </div>



                    <div class="endpoint">
                        <div class="endpoint-header">
                            <span class="method get">GET</span>
                            <span class="method post">POST</span>
                            <span class="path">/api/images/query</span>
                        </div>
                        <div class="endpoint-content">
                            <div class="description">高级查询图片列表 (支持分页、过滤、排序) - GET使用URL参数，POST使用JSON请求体</div>
                        </div>
                    </div>

                    <div class="endpoint">
                        <div class="endpoint-header">
                            <span class="method get">GET</span>
                            <span class="path">/api/stats</span>
                        </div>
                        <div class="endpoint-content">
                            <div class="description">获取存储统计信息</div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <div class="card">
            <div class="card-header">图片转换参数详解</div>
            <div class="card-content">
                <div style="color: #cbd5e1; line-height: 1.8;">
                    <h4 style="color: #06b6d4; margin-bottom: 15px;">🎯 转换参数语法</h4>
                    <p style="margin-bottom: 15px;">在图片URL后添加 <code style="background: rgba(6, 182, 212, 0.2); padding: 2px 6px; border-radius: 4px;">@</code> 符号，然后用下划线分隔各种转换参数：</p>
                    
                    <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 20px; margin-bottom: 20px;">
                        <div>
                            <strong style="color: #f1f5f9;">📏 尺寸控制</strong>
                            <ul style="margin-top: 8px; padding-left: 20px;">
                                <li><code>w{数字}</code> - 设置最大宽度像素</li>
                                <li><code>h{数字}</code> - 设置最大高度像素</li>
                                <li>等比缩放，保持原图比例</li>
                                <li>小于设定值的图片不会放大</li>
                            </ul>
                        </div>
                        <div>
                            <strong style="color: #f1f5f9;">🎨 格式转换</strong>
                            <ul style="margin-top: 8px; padding-left: 20px;">
                                <li><code>jpeg</code> - 转为JPEG格式（有损）</li>
                                <li><code>png</code> - 转为PNG格式（无损）</li>
                                <li><code>webp</code> - 转为WebP格式（无损）</li>
                                <li><code>avif</code> - 转为AVIF格式（有损）</li>
    
    
                                <li><code>ico</code> - 转为ICO格式</li>
                            </ul>
                        </div>
                        <div>
                            <strong style="color: #f1f5f9;">🎛️ 质量控制</strong>
                            <ul style="margin-top: 8px; padding-left: 20px;">
                                <li><code>q{1-100}</code> - 设置图片质量</li>
                                <li>仅对JPEG等有损格式有效</li>
                                <li>数值越高质量越好</li>
                            </ul>
                        </div>
                        <div>
                            <strong style="color: #f1f5f9;">🌈 透明度处理</strong>
                            <ul style="margin-top: 8px; padding-left: 20px;">
                                <li><code>na</code> - 去除透明通道（默认白色背景）</li>
                                <li><code>naw</code> - 去透明+白色背景</li>
                                <li><code>nab</code> - 去透明+黑色背景</li>
                                <li><code>na#ff0000</code> - 去透明+自定义颜色</li>
                            </ul>
                        </div>
                    </div>

                    <div style="background: rgba(6, 182, 212, 0.1); padding: 15px; border-radius: 8px; border-left: 4px solid #06b6d4;">
                        <strong style="color: #06b6d4;">💡 使用示例</strong>
                        <div style="margin-top: 8px; font-family: 'Monaco', 'Courier New', monospace; font-size: 0.9rem;">
                            <div>/images/abc123@w800_h600 - 限制在800x600范围内，保持比例</div>
                            <div>/images/abc123@w1200_jpeg_q90 - 最大宽度1200px，转JPEG，质量90</div>
                            <div>/images/abc123@h800_webp_naw - 最大高度800px，转WebP，白背景</div>
                            <div>/images/abc123@png - GIF转PNG（提取第一帧）</div>
                            <div>/images/abc123@w600_jpeg - GIF第一帧转JPEG，最大宽度600px</div>
                            <div>/images/abc123@w600_na#00ff00 - 最大宽度600px，绿色背景</div>
                        </div>
                    </div>
                    
                    <div style="background: rgba(139, 92, 246, 0.1); padding: 15px; border-radius: 8px; border-left: 4px solid #8b5cf6; margin-top: 15px;">
                        <strong style="color: #8b5cf6;">🎬 动画处理示例</strong>
                        <div style="margin-top: 8px; font-family: 'Monaco', 'Courier New', monospace; font-size: 0.9rem;">
                            <div>/images/animated_gif - 保持GIF动画（原尺寸）</div>
                            <div>/images/animated_gif@w800 - 仍返回原动画（不支持动画尺寸调整）</div>
                            <div>/images/animated_gif@jpeg - 提取第一帧转JPEG</div>
                            <div>/images/animated_gif@w600_jpeg - 第一帧转JPEG，600px宽</div>
                            <div>/images/animated_gif@w600_png_naw - 第一帧转PNG，白背景，600px宽</div>
                            <div>/images/animated_webp@avif_q90 - WebP第一帧转AVIF，质量90</div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <div class="card">
            <div class="card-header">上传测试</div>
            <div class="card-content">
                <div class="upload-section">
                    <h3 style="margin-bottom: 20px; color: #475569;">测试图片上传</h3>
                    <form class="upload-form" action="/upload" method="post" enctype="multipart/form-data">
                        <div class="file-input-wrapper">
                            <input type="file" name="file" accept="image/*" required class="file-input" id="fileInput">
                            <label for="fileInput" class="file-label" id="fileLabel">
                                <div class="upload-icon">📁</div>
                                <div>
                                    <strong>点击选择图片</strong><br>
                                    <small>或拖拽文件到此处</small>
                                </div>
                            </label>
                        </div>
                        <button type="submit" class="btn">上传图片</button>
                    </form>
                </div>
            </div>
        </div>

        <div class="card">
            <div class="card-header">🎯 支持的图片格式</div>
            <div class="card-content">
                <div class="format-table">
                    <div class="format-category">
                        <h4>传统格式 (支持完整编解码)</h4>
                        <div class="format-list">
                            <span class="format-item supported">JPEG (.jpg, .jpeg)</span>
                            <span class="format-item supported">PNG (.png)</span>
                            <span class="format-item supported">GIF (.gif) - 智能动画处理</span>


                            <span class="format-item supported">ICO (.ico)</span>
                        </div>
                    </div>
                    <div class="format-category">
                        <h4>现代格式 (支持完整编解码)</h4>
                        <div class="format-list">
                            <span class="format-item supported webp">WebP (.webp) - 质量可控</span>
                            <span class="format-item supported avif">AVIF (.avif) - 高效压缩</span>
                        </div>
                    </div>
                </div>

            </div>
        </div>
    </div>

         <script>
         // 预设配色方案
         const colorSchemes = [
             {
                 name: '青蓝主题',
                 primary: '#06b6d4',
                 secondary: '#3b82f6',
                 accent: '#8b5cf6',
                 primaryDark: '#0891b2',
                 secondaryDark: '#2563eb'
             },
             {
                 name: '紫罗兰主题',
                 primary: '#8b5cf6',
                 secondary: '#a855f7',
                 accent: '#06b6d4',
                 primaryDark: '#7c3aed',
                 secondaryDark: '#9333ea'
             },
             {
                 name: '翠绿主题',
                 primary: '#10b981',
                 secondary: '#059669',
                 accent: '#06b6d4',
                 primaryDark: '#047857',
                 secondaryDark: '#065f46'
             },
             {
                 name: '橙红主题',
                 primary: '#f59e0b',
                 secondary: '#ef4444',
                 accent: '#8b5cf6',
                 primaryDark: '#d97706',
                 secondaryDark: '#dc2626'
             },
             {
                 name: '玫瑰主题',
                 primary: '#ec4899',
                 secondary: '#f43f5e',
                 accent: '#8b5cf6',
                 primaryDark: '#db2777',
                 secondaryDark: '#e11d48'
             },
             {
                 name: '靛青主题',
                 primary: '#6366f1',
                 secondary: '#8b5cf6',
                 accent: '#06b6d4',
                 primaryDark: '#4f46e5',
                 secondaryDark: '#7c3aed'
             },
             {
                 name: '深海主题',
                 primary: '#0284c7',
                 secondary: '#0f766e',
                 accent: '#7c3aed',
                 primaryDark: '#0369a1',
                 secondaryDark: '#134e4a'
             },
             {
                 name: '夕阳主题',
                 primary: '#ea580c',
                 secondary: '#dc2626',
                 accent: '#f59e0b',
                 primaryDark: '#c2410c',
                 secondaryDark: '#b91c1c'
             },
             {
                 name: '森林主题',
                 primary: '#16a34a',
                 secondary: '#059669',
                 accent: '#65a30d',
                 primaryDark: '#15803d',
                 secondaryDark: '#047857'
             },
             {
                 name: '樱花主题',
                 primary: '#f472b6',
                 secondary: '#e879f9',
                 accent: '#fb7185',
                 primaryDark: '#ec4899',
                 secondaryDark: '#d946ef'
             },
             {
                 name: '暗夜主题',
                 primary: '#64748b',
                 secondary: '#475569',
                 accent: '#6366f1',
                 primaryDark: '#475569',
                 secondaryDark: '#334155'
             },
             {
                 name: '极光主题',
                 primary: '#22d3ee',
                 secondary: '#a78bfa',
                 accent: '#34d399',
                 primaryDark: '#06b6d4',
                 secondaryDark: '#8b5cf6'
             },
             {
                 name: '火焰主题',
                 primary: '#f97316',
                 secondary: '#ef4444',
                 accent: '#eab308',
                 primaryDark: '#ea580c',
                 secondaryDark: '#dc2626'
             },
             {
                 name: '天空主题',
                 primary: '#3b82f6',
                 secondary: '#06b6d4',
                 accent: '#8b5cf6',
                 primaryDark: '#2563eb',
                 secondaryDark: '#0891b2'
             },
             {
                 name: '薄荷主题',
                 primary: '#10b981',
                 secondary: '#06b6d4',
                 accent: '#34d399',
                 primaryDark: '#059669',
                 secondaryDark: '#0891b2'
             }
         ];

         // 生成随机颜色 (HSL色彩空间，确保颜色鲜艳且和谐)
         function generateRandomColor() {
             const hue = Math.floor(Math.random() * 360);
             const saturation = 60 + Math.floor(Math.random() * 40); // 60-100%
             const lightness = 45 + Math.floor(Math.random() * 20);  // 45-65%
             return `hsl(${hue}, ${saturation}%, ${lightness}%)`;
         }

         // 生成更深的颜色变体
         function generateDarkerColor(baseHsl) {
             const hslMatch = baseHsl.match(/hsl\((\d+), (\d+)%, (\d+)%\)/);
             if (hslMatch) {
                 const [, h, s, l] = hslMatch;
                 const newLightness = Math.max(20, parseInt(l) - 15);
                 return `hsl(${h}, ${s}%, ${newLightness}%)`;
             }
             return baseHsl;
         }

         // 生成随机配色方案
         function generateRandomScheme() {
             const primary = generateRandomColor();
             const secondary = generateRandomColor();
             const accent = generateRandomColor();
             
             return {
                 name: '🎨 随机主题',
                 primary: primary,
                 secondary: secondary,
                 accent: accent,
                 primaryDark: generateDarkerColor(primary),
                 secondaryDark: generateDarkerColor(secondary)
             };
         }

         // 随机选择配色方案 (30%概率生成随机颜色，70%使用预设主题)
         function getRandomColorScheme() {
             const useRandomGeneration = Math.random() < 0.3;
             
             if (useRandomGeneration) {
                 return generateRandomScheme();
             } else {
                 return colorSchemes[Math.floor(Math.random() * colorSchemes.length)];
             }
         }

         // 应用配色方案
         function applyColorScheme(scheme) {
             const root = document.documentElement;
             
             // 创建动态样式
             const style = document.createElement('style');
             style.textContent = `
                 :root {
                     --primary-color: ${scheme.primary};
                     --secondary-color: ${scheme.secondary};
                     --accent-color: ${scheme.accent};
                     --primary-dark: ${scheme.primaryDark};
                     --secondary-dark: ${scheme.secondaryDark};
                 }
                 
                 .header h1 {
                     background: linear-gradient(135deg, ${scheme.primary}, ${scheme.secondary}, ${scheme.accent}) !important;
                     -webkit-background-clip: text !important;
                     -webkit-text-fill-color: transparent !important;
                     background-clip: text !important;
                 }
                 
                 .card-header {
                     background: linear-gradient(135deg, ${scheme.primary}, ${scheme.secondary}) !important;
                 }
                 
                 .feature {
                     border: 1px solid ${scheme.primary}50 !important;
                 }
                 
                 .feature:hover {
                     border-color: ${scheme.primary}99 !important;
                     box-shadow: 0 8px 25px ${scheme.primary}25 !important;
                 }
                 
                 .feature-icon {
                     background: linear-gradient(135deg, ${scheme.primary}, ${scheme.primaryDark}) !important;
                 }
                 
                 .endpoint:hover {
                     border-color: ${scheme.primary}80 !important;
                 }
                 
                 .method.get {
                     background: linear-gradient(135deg, ${scheme.primary}, ${scheme.primaryDark}) !important;
                 }
                 
                 .method.post {
                     background: linear-gradient(135deg, ${scheme.secondary}, ${scheme.secondaryDark}) !important;
                 }
                 
                 .path {
                     color: ${scheme.primary} !important;
                     background: ${scheme.primary}1a !important;
                 }
                 
                 .upload-section {
                     border: 1px solid ${scheme.primary}33 !important;
                 }
                 
                 .file-label {
                     border: 2px dashed ${scheme.primary}80 !important;
                 }
                 
                 .file-label:hover,
                 .file-label.drag-over {
                     border-color: ${scheme.primary} !important;
                     background: ${scheme.primary}1a !important;
                     box-shadow: 0 12px 25px ${scheme.primary}25 !important;
                 }
                 
                 .upload-icon {
                     color: ${scheme.primary} !important;
                 }
                 
                 .btn {
                     background: linear-gradient(135deg, ${scheme.primary}, ${scheme.secondary}) !important;
                     box-shadow: 0 4px 12px ${scheme.primary}4d !important;
                 }
                 
                 .btn:hover {
                     background: linear-gradient(135deg, ${scheme.primaryDark}, ${scheme.secondaryDark}) !important;
                     box-shadow: 0 8px 25px ${scheme.primary}66 !important;
                 }
                 
                 .storage-info {
                     border: 1px solid ${scheme.primary}33 !important;
                 }
                 
                 .storage-info h4 {
                     color: ${scheme.primary} !important;
                 }
                 
                 .storage-info code {
                     background: ${scheme.primary}1a !important;
                     color: ${scheme.primary} !important;
                 }
             `;
             
             document.head.appendChild(style);
             
             // 在控制台显示当前主题
             console.log(`🎨 当前主题: ${scheme.name}`);
         }

         // 页面加载时应用随机配色
         document.addEventListener('DOMContentLoaded', function() {
             const randomScheme = getRandomColorScheme();
             applyColorScheme(randomScheme);
         });

         const fileInput = document.getElementById('fileInput');
         const fileLabel = document.getElementById('fileLabel');

        // 文件选择处理
        fileInput.addEventListener('change', function(e) {
            if (e.target.files.length > 0) {
                const fileName = e.target.files[0].name;
                fileLabel.innerHTML = `
                    <div class="upload-icon">✓</div>
                    <div>
                        <strong>已选择: ${fileName}</strong><br>
                        <small>点击重新选择</small>
                    </div>
                `;
            }
        });

        // 拖拽支持
        fileLabel.addEventListener('dragover', function(e) {
            e.preventDefault();
            fileLabel.classList.add('drag-over');
        });

        fileLabel.addEventListener('dragleave', function() {
            fileLabel.classList.remove('drag-over');
        });

        fileLabel.addEventListener('drop', function(e) {
            e.preventDefault();
            fileLabel.classList.remove('drag-over');
            
            const files = e.dataTransfer.files;
            if (files.length > 0 && files[0].type.startsWith('image/')) {
                fileInput.files = files;
                const fileName = files[0].name;
                fileLabel.innerHTML = `
                    <div class="upload-icon">✓</div>
                    <div>
                        <strong>已选择: ${fileName}</strong><br>
                        <small>点击重新选择</small>
                    </div>
                `;
            }
        });
         </script>
 </body>
 </html>"#;

/// 缓存管理面板HTML内容
pub const CACHE_MANAGEMENT_HTML: &str = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RIFS 缓存管理面板</title>
    <style>
        body { 
            font-family: 'Segoe UI', Arial, sans-serif; 
            margin: 0; 
            padding: 20px; 
            background: linear-gradient(135deg, #0f172a 0%, #1e293b 50%, #334155 100%);
            min-height: 100vh;
            color: #e2e8f0;
        }
        .container { 
            max-width: 1200px; 
            margin: 0 auto; 
        }
        .card { 
            background: rgba(30, 41, 59, 0.8);
            backdrop-filter: blur(20px);
            border-radius: 16px; 
            padding: 20px; 
            margin: 20px 0; 
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
            border: 1px solid rgba(148, 163, 184, 0.2);
        }
        .header { 
            text-align: center; 
            color: #f1f5f9; 
            margin-bottom: 30px; 
        }
        .header h1 {
            font-size: 2.5rem;
            font-weight: 800;
            margin-bottom: 10px;
            background: linear-gradient(135deg, #06b6d4, #3b82f6, #8b5cf6);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }
        .header p {
            color: #94a3b8;
            font-size: 1.1rem;
        }
        .stats-grid { 
            display: grid; 
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); 
            gap: 20px;
            margin-bottom: 20px;
        }
        .stat-card { 
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); 
            color: white; 
            padding: 24px; 
            border-radius: 12px; 
            text-align: center;
            transition: transform 0.3s ease;
        }
        .stat-card:hover {
            transform: translateY(-2px);
        }
        .stat-value { 
            font-size: 2.2em; 
            font-weight: bold; 
            margin-bottom: 8px; 
        }
        .stat-label { 
            font-size: 0.9em; 
            opacity: 0.9; 
        }
        .action-grid { 
            display: grid; 
            grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); 
            gap: 20px; 
        }
        .action-card { 
            border: 2px solid rgba(148, 163, 184, 0.2); 
            border-radius: 12px; 
            padding: 24px; 
            transition: all 0.3s;
            background: rgba(15, 23, 42, 0.6);
            backdrop-filter: blur(10px);
        }
        .action-card:hover { 
            border-color: #667eea; 
            transform: translateY(-2px);
            box-shadow: 0 8px 25px rgba(102, 126, 234, 0.15);
        }
        .action-card h3 {
            color: #f1f5f9;
            margin-bottom: 12px;
            font-size: 1.2rem;
        }
        .action-card p {
            color: #cbd5e1;
            margin-bottom: 20px;
            line-height: 1.6;
        }
        .btn { 
            background: linear-gradient(135deg, #667eea, #764ba2); 
            color: white; 
            border: none; 
            padding: 12px 24px; 
            border-radius: 8px; 
            cursor: pointer; 
            font-size: 14px; 
            font-weight: 600;
            transition: all 0.3s;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }
        .btn:hover { 
            background: linear-gradient(135deg, #5a67d8, #6b46c1);
            transform: translateY(-1px);
            box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
        }
        .btn-danger { 
            background: linear-gradient(135deg, #e53e3e, #c53030);
        }
        .btn-danger:hover { 
            background: linear-gradient(135deg, #c53030, #9c2626);
        }
        .btn-warning { 
            background: linear-gradient(135deg, #ed8936, #dd6b20);
        }
        .btn-warning:hover { 
            background: linear-gradient(135deg, #dd6b20, #c05621);
        }
        .result { 
            margin-top: 20px; 
            padding: 16px; 
            border-radius: 8px; 
            display: none;
            font-weight: 500;
        }
        .result.success { 
            background: rgba(16, 185, 129, 0.1); 
            color: #10b981; 
            border: 1px solid rgba(16, 185, 129, 0.3);
        }
        .result.error { 
            background: rgba(239, 68, 68, 0.1); 
            color: #ef4444; 
            border: 1px solid rgba(239, 68, 68, 0.3);
        }
        .loading { 
            display: none; 
            text-align: center; 
            color: #94a3b8;
            font-style: italic;
        }
        .policy-form { 
            display: grid; 
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); 
            gap: 15px; 
            margin: 15px 0; 
        }
        .form-group label { 
            display: block; 
            margin-bottom: 5px; 
            font-weight: 500;
            color: #f1f5f9;
        }
        .form-group input { 
            width: 100%; 
            padding: 10px; 
            border: 1px solid rgba(148, 163, 184, 0.3); 
            border-radius: 6px;
            background: rgba(30, 41, 59, 0.8);
            color: #f1f5f9;
            transition: border-color 0.3s;
        }
        .form-group input:focus {
            outline: none;
            border-color: #667eea;
            box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
        }
        .form-group input::placeholder {
            color: #94a3b8;
        }
        
        @media (max-width: 768px) {
            body { padding: 10px; }
            .header h1 { font-size: 2rem; }
            .action-grid { grid-template-columns: 1fr; }
            .stats-grid { grid-template-columns: repeat(2, 1fr); }
        }
        
        @media (max-width: 480px) {
            .stats-grid { grid-template-columns: 1fr; }
            .policy-form { grid-template-columns: 1fr; }
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🗄️ RIFS 缓存管理面板</h1>
            <p>智能图片转换缓存管理系统</p>
        </div>

        <div class="card">
            <h2 style="color: #f1f5f9; margin-bottom: 20px; display: flex; align-items: center; gap: 10px;">
                📊 缓存统计
                <button class="btn" onclick="loadStats()" style="margin-left: auto; padding: 8px 16px; font-size: 12px;">
                    🔄 刷新
                </button>
            </h2>
            <div id="stats-container">
                <div class="loading">正在加载统计信息...</div>
                <div class="stats-grid" id="stats-grid" style="display: none;"></div>
            </div>
        </div>

        <div class="card">
            <h2 style="color: #f1f5f9; margin-bottom: 20px;">🧹 缓存清理操作</h2>
            <div class="action-grid">
                <div class="action-card">
                    <h3>🤖 智能自动清理</h3>
                    <p>根据系统配置的策略进行智能清理，包括年龄、热度、大小等因素。安全且高效。</p>
                    <button class="btn" onclick="autoCleanup()">执行自动清理</button>
                </div>

                <div class="action-card">
                    <h3>🧠 热度衰减</h3>
                    <p>手动触发热度评分衰减计算，根据配置的衰减因子更新所有缓存的热度评分。</p>
                    <button class="btn btn-warning" onclick="decayHeatScores()">执行热度衰减</button>
                </div>

                <div class="action-card">
                    <h3>🚀 智能清理</h3>
                    <p>执行完整的智能清理流程：先进行热度衰减，再清理低热度缓存和过期项。最全面的清理策略。</p>
                    <button class="btn" onclick="smartCleanup()">执行智能清理</button>
                </div>

                <div class="action-card">
                    <h3>💾 智能空间管理</h3>
                    <p>分层清理策略：总是清理完全无热度的缓存，仅在空间不足时清理低热度缓存。智能保护有价值的内容。</p>
                    <button class="btn btn-warning" onclick="smartSpaceCleanup()">执行空间管理</button>
                </div>

                <div class="action-card">
                    <h3>⚙️ 自定义策略清理</h3>
                    <p>根据您的自定义参数进行精确清理，可单独或组合使用各种策略。</p>
                    <div class="policy-form">
                        <div class="form-group">
                            <label>最大缓存数量</label>
                            <input type="number" id="maxEntries" placeholder="如：1000">
                        </div>
                        <div class="form-group">
                            <label>最大总大小 (MB)</label>
                            <input type="number" id="maxSize" placeholder="如：100">
                        </div>
                        <div class="form-group">
                            <label>最大生存时间 (天)</label>
                            <input type="number" id="maxAge" placeholder="如：30">
                        </div>
                        <div class="form-group">
                            <label>最小热度评分</label>
                            <input type="number" step="0.1" id="minHeat" placeholder="如：0.1">
                        </div>
                    </div>
                    <button class="btn btn-warning" onclick="customCleanup()">执行自定义清理</button>
                </div>

                <div class="action-card">
                    <h3>🗑️ 清空所有缓存</h3>
                    <p style="color: #ef4444;">⚠️ 危险操作：此操作将删除所有缓存文件，不可恢复！请谨慎使用。</p>
                    <button class="btn btn-danger" onclick="clearAll()">清空所有缓存</button>
                </div>
            </div>
        </div>

        <div class="card">
            <h2 style="color: #f1f5f9; margin-bottom: 20px;">📚 操作说明</h2>
            <div style="color: #cbd5e1; line-height: 1.6;">
                <h4 style="color: #06b6d4; margin-bottom: 10px;">🎯 清理策略说明</h4>
                <ul style="margin-left: 20px;">
                    <li><strong>年龄清理:</strong> 删除超过指定天数的缓存</li>
                    <li><strong>大小限制:</strong> 当总大小超出限制时删除最冷缓存</li>
                    <li><strong>数量限制:</strong> 当缓存数量超出限制时使用LRU策略</li>
                    <li><strong>热度清理:</strong> 删除热度评分低于阈值的缓存</li>
                </ul>
                
                <h4 style="color: #06b6d4; margin: 20px 0 10px 0;">🔄 缓存热度评分</h4>
                <p style="margin-left: 20px;">
                    系统根据访问频率和时间衰减自动计算热度评分，热门缓存会被优先保留。
                    评分越高表示缓存越重要，越不容易被清理。
                </p>

                <h4 style="color: #06b6d4; margin: 20px 0 10px 0;">📉 热度衰减机制</h4>
                <ul style="margin-left: 20px;">
                    <li><strong>基础评分:</strong> 访问次数 ÷ 缓存年龄（小时）</li>
                    <li><strong>时间衰减:</strong> 基础评分 × 衰减因子^(距上次访问小时数)</li>
                    <li><strong>衰减因子:</strong> 配置值（0.0-1.0），越小衰减越快</li>
                    <li><strong>自动触发:</strong> 定时任务自动执行衰减和清理</li>
                </ul>

                <h4 style="color: #06b6d4; margin: 20px 0 10px 0;">💾 智能空间管理</h4>
                <ul style="margin-left: 20px;">
                    <li><strong>分层清理:</strong> 零热度缓存总是清理，低热度缓存按需清理</li>
                    <li><strong>零热度清理:</strong> 完全无热度（≤0.001）的缓存随时清理，不占用宝贵空间</li>
                    <li><strong>阈值触发:</strong> 低热度缓存仅在使用率超过设定阈值（默认80%）时清理</li>
                    <li><strong>保护机制:</strong> 智能保护有价值的缓存内容不被误删</li>
                    <li><strong>目标控制:</strong> 清理到阈值的90%，避免频繁触发</li>
                </ul>

                <h4 style="color: #06b6d4; margin: 20px 0 10px 0;">🚀 智能清理优势</h4>
                <ul style="margin-left: 20px;">
                    <li><strong>动态评分:</strong> 实时更新热度，确保评分准确性</li>
                    <li><strong>渐进清理:</strong> 优先清理最冷的缓存，保护热门内容</li>
                    <li><strong>多重策略:</strong> 结合年龄、大小、热度等多种清理策略</li>
                    <li><strong>自适应:</strong> 根据系统负载自动调整清理频率</li>
                </ul>
            </div>
        </div>

        <div id="result" class="result"></div>
    </div>

    <script>
        // 加载统计信息
        async function loadStats() {
            const loading = document.querySelector('#stats-container .loading');
            const grid = document.getElementById('stats-grid');
            
            loading.style.display = 'block';
            grid.style.display = 'none';
            
            try {
                const response = await fetch('/api/cache/stats');
                const result = await response.json();
                
                if (result.success && result.data) {
                    displayStats(result.data);
                } else {
                    showResult('获取统计信息失败: ' + (result.message || '未知错误'), 'error');
                }
            } catch (error) {
                showResult('网络错误: ' + error.message, 'error');
            } finally {
                loading.style.display = 'none';
            }
        }

        function displayStats(stats) {
            const grid = document.getElementById('stats-grid');
            grid.innerHTML = `
                <div class="stat-card">
                    <div class="stat-value">${stats.total_count || 0}</div>
                    <div class="stat-label">缓存总数</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">${formatSize(stats.total_size || 0)}</div>
                    <div class="stat-label">总大小</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">${formatSize(stats.average_size || 0)}</div>
                    <div class="stat-label">平均大小</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">${((stats.hit_rate || 0) * 100).toFixed(1)}%</div>
                    <div class="stat-label">命中率</div>
                </div>
            `;
            grid.style.display = 'grid';
        }

        function formatSize(bytes) {
            if (bytes === 0) return '0 B';
            const k = 1024;
            const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
        }

        // 自动清理
        async function autoCleanup() {
            if (!confirm('确定要执行自动清理吗？\n\n系统将根据配置的策略智能清理过期和低热度的缓存。')) return;
            
            try {
                showResult('正在执行自动清理...', 'success');
                const response = await fetch('/api/cache/cleanup/auto', { method: 'POST' });
                const result = await response.json();
                
                if (result.success && result.data) {
                    const message = `自动清理完成！\n删除 ${result.data.cleaned_count} 个缓存\n释放 ${formatSize(result.data.freed_space)}\n耗时 ${result.data.duration_ms}ms`;
                    showResult(message, 'success');
                    setTimeout(loadStats, 1000); // 延迟刷新统计
                } else {
                    showResult('自动清理失败: ' + (result.message || '未知错误'), 'error');
                }
            } catch (error) {
                showResult('网络错误: ' + error.message, 'error');
            }
        }

        // 热度衰减
        async function decayHeatScores() {
            if (!confirm('确定要执行热度衰减吗？\n\n系统将根据配置的衰减因子重新计算所有缓存的热度评分。')) return;
            
            try {
                showResult('正在执行热度衰减...', 'success');
                const response = await fetch('/api/cache/decay', { method: 'POST' });
                const result = await response.json();
                
                if (result.success) {
                    const message = `热度衰减完成！\n更新了 ${result.data || 0} 个缓存项的热度评分`;
                    showResult(message, 'success');
                    setTimeout(loadStats, 1000); // 延迟刷新统计
                } else {
                    showResult('热度衰减失败: ' + (result.message || '未知错误'), 'error');
                }
            } catch (error) {
                showResult('网络错误: ' + error.message, 'error');
            }
        }

        // 智能清理
        async function smartCleanup() {
            if (!confirm('确定要执行智能清理吗？\n\n系统将先进行热度衰减，然后清理低热度缓存和过期项。这是最全面的清理策略。')) return;
            
            try {
                showResult('正在执行智能清理...', 'success');
                const response = await fetch('/api/cache/cleanup/smart', { method: 'POST' });
                const result = await response.json();
                
                if (result.success && result.data) {
                    const policies = result.data.applied_policies.join('\\n• ');
                    const message = `智能清理完成！\n删除 ${result.data.cleaned_count} 个缓存\n释放 ${formatSize(result.data.freed_space)}\n耗时 ${result.data.duration_ms}ms\n\n应用的策略:\n• ${policies}`;
                    showResult(message, 'success');
                    setTimeout(loadStats, 1000); // 延迟刷新统计
                } else {
                    showResult('智能清理失败: ' + (result.message || '未知错误'), 'error');
                }
            } catch (error) {
                showResult('网络错误: ' + error.message, 'error');
            }
        }

        // 智能空间管理清理
        async function smartSpaceCleanup() {
            if (!confirm('确定要执行智能空间管理吗？\n\n系统将：\n1. 总是清理完全无热度的缓存（≤0.001）\n2. 仅在空间不足时清理低热度缓存')) return;
            
            try {
                showResult('正在检查空间使用情况...', 'info');
                const response = await fetch('/api/cache/cleanup/space', { method: 'POST' });
                const result = await response.json();
                
                                 if (result.success && result.data) {
                     if (result.data.cleaned_count > 0) {
                         const policies = result.data.applied_policies.join('\n• ');
                         const message = `智能空间管理清理完成！\n清理了 ${result.data.cleaned_count} 个缓存项\n释放 ${formatSize(result.data.freed_space)} 空间\n\n应用的策略:\n• ${policies}`;
                         showResult(message, 'success');
                     } else {
                         showResult('无需清理\n\n• 没有完全无热度的缓存\n• 空间使用率未超过阈值，无需清理低热度缓存', 'info');
                     }
                    setTimeout(loadStats, 1000); // 延迟刷新统计
                } else {
                    showResult('空间管理失败: ' + (result.message || '未知错误'), 'error');
                }
            } catch (error) {
                showResult('网络错误: ' + error.message, 'error');
            }
        }

        // 自定义清理
        async function customCleanup() {
            const policy = {
                max_entries: document.getElementById('maxEntries').value ? parseInt(document.getElementById('maxEntries').value) : null,
                max_total_size: document.getElementById('maxSize').value ? parseInt(document.getElementById('maxSize').value) * 1024 * 1024 : null,
                max_age: document.getElementById('maxAge').value ? parseInt(document.getElementById('maxAge').value) * 24 * 3600 : null,
                min_heat_score: document.getElementById('minHeat').value ? parseFloat(document.getElementById('minHeat').value) : null,
                enable_lru: true
            };

            // 检查是否至少设置了一个策略
            const hasPolicy = Object.values(policy).some(value => value !== null && value !== true);
            if (!hasPolicy) {
                showResult('请至少设置一个清理策略参数', 'error');
                return;
            }

            let policyDesc = '清理策略:\n';
            if (policy.max_entries) policyDesc += `• 最大缓存数量: ${policy.max_entries}\n`;
            if (policy.max_total_size) policyDesc += `• 最大总大小: ${formatSize(policy.max_total_size)}\n`;
            if (policy.max_age) policyDesc += `• 最大生存时间: ${policy.max_age / (24 * 3600)} 天\n`;
            if (policy.min_heat_score) policyDesc += `• 最小热度评分: ${policy.min_heat_score}\n`;

            if (!confirm(`确定要执行自定义策略清理吗？\n\n${policyDesc}`)) return;
            
            try {
                showResult('正在执行自定义清理...', 'success');
                const response = await fetch('/api/cache/cleanup/policy', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(policy)
                });
                const result = await response.json();
                
                if (result.success && result.data) {
                    const message = `策略清理完成！\n删除 ${result.data.cleaned_count} 个缓存\n释放 ${formatSize(result.data.freed_space)}\n耗时 ${result.data.duration_ms}ms`;
                    showResult(message, 'success');
                    setTimeout(loadStats, 1000); // 延迟刷新统计
                } else {
                    showResult('策略清理失败: ' + (result.message || '未知错误'), 'error');
                }
            } catch (error) {
                showResult('网络错误: ' + error.message, 'error');
            }
        }

        // 清空所有缓存
        async function clearAll() {
            if (!confirm('⚠️ 确定要清空所有缓存吗？\n\n此操作将删除所有缓存文件，不可恢复！')) return;
            if (!confirm('⚠️ 最后确认\n\n真的要删除所有缓存吗？这个操作无法撤销！')) return;
            
            try {
                showResult('正在清空所有缓存...', 'success');
                const response = await fetch('/api/cache/clear', { method: 'DELETE' });
                const result = await response.json();
                
                if (result.success && result.data) {
                    const message = `清空完成！\n删除 ${result.data.cleaned_count} 个缓存\n释放 ${formatSize(result.data.freed_space)}\n耗时 ${result.data.duration_ms}ms`;
                    showResult(message, 'success');
                    setTimeout(loadStats, 1000); // 延迟刷新统计
                } else {
                    showResult('清空失败: ' + (result.message || '未知错误'), 'error');
                }
            } catch (error) {
                showResult('网络错误: ' + error.message, 'error');
            }
        }

        function showResult(message, type) {
            const result = document.getElementById('result');
            result.textContent = message;
            result.className = `result ${type}`;
            result.style.display = 'block';
            
            // 成功消息5秒后消失，错误消息10秒后消失
            const timeout = type === 'success' ? 5000 : 10000;
            setTimeout(() => {
                result.style.display = 'none';
            }, timeout);
        }

        // 页面加载时获取统计信息
        window.addEventListener('load', loadStats);
        
        // 每30秒自动刷新统计信息
        setInterval(loadStats, 30000);
    </script>
</body>
</html>"#;

/// API文档根路径
pub async fn api_docs() -> impl IntoResponse {
    (StatusCode::OK, [("content-type", "text/html; charset=utf-8")], INDEX_HTML)
} 