# req-enc-dec-rust

基于 Rust 的请求/响应加解密 Flask 中间件，提供与原始 Python 版本相同的功能，但具有更好的性能和安全性。

## 特性

- 🚀 使用 Rust 编写，性能更高
- 🔒 支持 AES 和 DES 加密算法
- 🔧 可配置的字段级加解密
- 🐍 完全兼容 Python Flask 框架
- 📦 支持跨平台打包 (Windows, Linux, macOS)

## 安装

### 从 PyPI 安装

```bash
pip install req-enc-dec-rust
```

### 从源码安装

```bash
# 安装 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 maturin
pip install maturin

# 构建并安装
cd req_enc_dec_rust
maturin develop
```

## 使用方法

```python
from flask import Flask
from req_enc_dec_rust import EncryptionPlugin

app = Flask(__name__)

# 配置中间件
app.config["ENCRYPTION_ALGO"] = "AES"
app.config["ENCRYPTION_SALT"] = b"your_salt_value_1234567890123456"
app.config["ENCRYPTION_KEY"] = b'secret_key_32_bytes_long_1234567890'
app.config["ENCRYPTION_URL_CONFIGS"] = {
    "/api/user": {
        "decrypt_fields": ["email"],
        "encrypt_fields": ["user.token", "user.list.name", "user.list.email.email_name", "user.list.qq"]
    }
}

# 初始化中间件
EncryptionPlugin(app=app)

@app.route("/api/user", methods=["POST"])
def handle_user():
    # 请求中的 email 字段会自动解密
    # 响应中的指定字段会自动加密
    return {
        "user": {
            "token": "test_token_12345",
            "list": [
                {
                    "name": "test_name01",
                    "email": [
                        {"email_name": "test_email01@example.com"},
                        {"email_name": "test_email02@example.com"}
                    ],
                    "qq": ["test_qq01", "test_qq02"],
                }
            ]
        }
    }
```

## 配置说明

### 必需配置
- `ENCRYPTION_ALGO`: 加密算法，支持 "AES" 或 "DES"
- `ENCRYPTION_SALT`: 盐值，用于生成 IV
- `ENCRYPTION_KEY`: 加密密钥

### URL 配置
- `ENCRYPTION_URL_CONFIGS`: 路径到配置的映射
  - `decrypt_fields`: 需要解密的字段列表（支持嵌套字段，使用点号分隔）
  - `encrypt_fields`: 需要加密的字段列表

## 支持的平台

- Linux (x86_64, x86, aarch64)
- Windows (x64, x86)
- macOS (x86_64, aarch64)

## 开发

### 本地开发

```bash
# 安装开发依赖
pip install maturin

# 开发模式安装
maturin develop

# 运行测试
python example_app.py
```

### 构建发布包

```bash
# 构建所有平台的 wheel 包
maturin build --release

# 构建特定平台的 wheel 包
maturin build --release --target x86_64

# 构建源码包
maturin sdist
```

## 性能对比

与原始 Python 版本相比，Rust 版本提供：
- ⚡ 更快的加解密速度
- 💾 更低的内存使用
- 🔒 更好的安全性

## 许可证

MIT License