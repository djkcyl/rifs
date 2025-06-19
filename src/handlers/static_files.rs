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
                            <small style="color: #94a3b8;">JPEG, PNG, GIF, WebP, BMP, TIFF</small>
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
                            <span class="path">/images/{identifier}</span>
                        </div>
                        <div class="endpoint-content">
                            <div class="description">获取图片文件 (通过哈希值)</div>
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
                            <div class="description">删除图片文件 (通过哈希值)</div>
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

/// API文档根路径
pub async fn api_docs() -> impl IntoResponse {
    (StatusCode::OK, [("content-type", "text/html; charset=utf-8")], INDEX_HTML)
} 