# 构建和发布指南

## 本地开发

### 1. 安装依赖

```bash
# 安装 Rust (如果尚未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 maturin
pip install maturin

# 或者使用 conda
conda install -c conda-forge maturin
```

### 2. 开发模式安装

```bash
cd req_enc_dec_rust
maturin develop
```

### 3. 测试应用

```bash
# 运行示例应用
python example_app.py

# 在另一个终端测试
curl -X GET http://127.0.0.1:5001/api/health
```

## 构建发布包

### 构建所有平台

```bash
# 构建所有支持平台的 wheel 包
maturin build --release

# 只构建当前平台
maturin build --release --target $(rustc -vV | sed -n 's|host: ||p')
```

### 构建特定平台

```bash
# Linux x86_64
maturin build --release --target x86_64-unknown-linux-gnu

# Linux ARM64
maturin build --release --target aarch64-unknown-linux-gnu

# Windows x64
maturin build --release --target x86_64-pc-windows-msvc

# macOS x86_64
maturin build --release --target x86_64-apple-darwin

# macOS ARM64
maturin build --release --target aarch64-apple-darwin
```

### 构建源码包

```bash
maturin sdist
```

## GitHub Actions 自动发布

### 1. 设置 PyPI API Token

在 GitHub 仓库的 Settings → Secrets and variables → Actions 中添加：
- `PYPI_API_TOKEN`: 你的 PyPI API token

### 2. 触发发布

创建新的 release 标签来触发自动发布：

```bash
git tag v0.1.0
git push origin v0.1.0
```

## 手动发布到 PyPI

### 1. 安装 twine

```bash
pip install twine
```

### 2. 构建包

```bash
# 构建所有包
maturin build --release

# 或者只构建当前平台
maturin build --release --interpreter python3
```

### 3. 上传到 PyPI

```bash
# 上传到测试 PyPI (推荐先测试)
twine upload --repository-url https://test.pypi.org/legacy/ target/wheels/*

# 上传到正式 PyPI
twine upload target/wheels/*
```

## 支持的平台列表

| 平台 | 目标名称 | 说明 |
|------|----------|------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` | 64位 Linux |
| Linux x86 | `i686-unknown-linux-gnu` | 32位 Linux |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | ARM64 Linux |
| Windows x64 | `x86_64-pc-windows-msvc` | 64位 Windows |
| Windows x86 | `i686-pc-windows-msvc` | 32位 Windows |
| macOS x86_64 | `x86_64-apple-darwin` | Intel Mac |
| macOS ARM64 | `aarch64-apple-darwin` | Apple Silicon Mac |

## 故障排除

### 常见问题

1. **OpenSSL 错误**: 确保系统已安装 OpenSSL 开发包
   - Ubuntu/Debian: `sudo apt-get install libssl-dev pkg-config`
   - CentOS/RHEL: `sudo yum install openssl-devel`
   - macOS: `brew install openssl pkg-config`

2. **Python 版本不兼容**: 确保使用 Python 3.7 或更高版本

3. **Rust 工具链问题**: 更新 Rust 工具链
   ```bash
   rustup update
   ```

### 调试信息

```bash
# 显示详细的构建信息
maturin build --release -v

# 检查 Rust 代码
cargo check

# 运行测试
cargo test
```