# GitHub Actions 工作流说明

本项目使用GitHub Actions进行自动化构建、测试和部署。

## 工作流概览

### 1. CI (持续集成) - `.github/workflows/ci.yml`

**触发条件：**
- Push到`main`或`develop`分支
- 向`main`分支提交Pull Request

**功能：**
- 在多个操作系统上运行测试 (Ubuntu, macOS, Windows)
- 测试多个Rust版本 (stable, beta, nightly)
- 代码格式检查 (cargo fmt)
- 代码质量检查 (cargo clippy)
- 构建多平台二进制文件

### 2. Release (发布) - `.github/workflows/release.yml`

**触发条件：**
- 推送带`v`前缀的标签 (如 `v1.0.0`)
- 手动触发 (workflow_dispatch)

**功能：**
- 为以下平台构建二进制文件：
  - Linux x86_64 (标准glibc)
  - Linux x86_64 (musl静态链接)
  - Linux ARM64
  - macOS x86_64 (Intel)
  - macOS ARM64 (Apple Silicon)
  - Windows x86_64
- 生成SHA256校验和
- 创建GitHub Release
- 上传所有构建产物到Release

### 3. Deploy (部署) - `.github/workflows/deploy.yml`

**触发条件：**
- 手动触发，可选择环境（staging/production）

**功能：**
- 构建Linux二进制文件
- 创建部署包
- (可选) 通过SSH部署到服务器

## 使用说明

### 创建新版本发布

1. **更新版本号** (在`Cargo.toml`中)：
```toml
[package]
version = "1.0.0"
```

2. **提交更改**：
```bash
git add .
git commit -m "Release v1.0.0"
git push
```

3. **创建并推送标签**：
```bash
git tag v1.0.0
git push origin v1.0.0
```

4. **等待自动构建**：
- GitHub Actions会自动开始构建所有平台的二进制文件
- 构建完成后会自动创建Release并上传文件
- 访问 `https://github.com/YOUR_USERNAME/aurelia/releases` 查看

### 手动触发发布

1. 访问 Actions 页面
2. 选择 "Release" workflow
3. 点击 "Run workflow"
4. 输入标签名称（如 `manual-v1.0.0`）
5. 点击 "Run workflow" 按钮

### 手动部署

1. 访问 Actions 页面
2. 选择 "Deploy" workflow
3. 点击 "Run workflow"
4. 选择环境 (staging/production)
5. (可选) 输入服务器IP
6. 点击 "Run workflow" 按钮

## 配置服务器部署

要启用自动部署到服务器，需要在GitHub仓库设置中添加以下Secrets：

1. **SSH_PRIVATE_KEY**：服务器的SSH私钥
2. **DEFAULT_SERVER_IP**：默认服务器IP地址
3. **SERVER_USER**：SSH用户名（如`ubuntu`或`root`）

### 添加Secrets步骤：

1. 进入仓库设置 (Settings)
2. 选择 "Secrets and variables" → "Actions"
3. 点击 "New repository secret"
4. 添加上述secrets

### 生成部署用SSH密钥：

```bash
# 生成新的SSH密钥对（专门用于部署）
ssh-keygen -t ed25519 -C "github-actions-deploy" -f deploy_key

# 将公钥添加到服务器
ssh-copy-id -i deploy_key.pub user@your-server-ip

# 将私钥内容复制到GitHub Secrets
cat deploy_key
```

## 下载构建的二进制文件

### 从Release页面下载

1. 访问 `https://github.com/YOUR_USERNAME/aurelia/releases`
2. 选择需要的版本
3. 下载对应平台的文件

### 使用命令行下载

```bash
# Linux x86_64
wget https://github.com/YOUR_USERNAME/aurelia/releases/latest/download/aurelia-linux-x86_64.tar.gz
tar xzf aurelia-linux-x86_64.tar.gz

# macOS Apple Silicon
wget https://github.com/YOUR_USERNAME/aurelia/releases/latest/download/aurelia-macos-aarch64.tar.gz
tar xzf aurelia-macos-aarch64.tar.gz

# Windows
curl -L -o aurelia-windows.zip https://github.com/YOUR_USERNAME/aurelia/releases/latest/download/aurelia-windows-x86_64.zip
```

## 验证文件完整性

每个发布的文件都包含SHA256校验和：

```bash
# Linux/macOS
sha256sum -c aurelia-linux-x86_64.tar.gz.sha256

# Windows (PowerShell)
Get-FileHash aurelia-windows-x86_64.zip -Algorithm SHA256
```

## 故障排除

### 构建失败

1. 检查Rust版本兼容性
2. 确保所有依赖都在`Cargo.toml`中正确声明
3. 查看Actions日志获取详细错误信息

### 部署失败

1. 确认SSH密钥正确配置
2. 检查服务器防火墙设置
3. 确保目标目录有写入权限

### 跨平台编译问题

- Linux musl构建需要`musl-tools`
- ARM64构建需要相应的交叉编译工具链
- Windows构建可能需要Visual Studio Build Tools

## 本地测试Workflows

使用[act](https://github.com/nektos/act)可以在本地测试GitHub Actions：

```bash
# 安装act
brew install act  # macOS
# 或
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash  # Linux

# 测试CI workflow
act push

# 测试Release workflow
act release

# 使用特定的事件文件
act -e .github/events/release.json
```

## 监控构建状态

在README中添加构建状态徽章：

```markdown
![CI](https://github.com/YOUR_USERNAME/aurelia/workflows/CI/badge.svg)
![Release](https://github.com/YOUR_USERNAME/aurelia/workflows/Release/badge.svg)
```

## 最佳实践

1. **版本管理**：使用语义化版本号 (SemVer)
2. **分支保护**：为main分支启用保护规则，要求CI通过
3. **缓存优化**：合理使用缓存加速构建
4. **并行构建**：利用matrix策略并行构建多个目标
5. **安全性**：永远不要在代码中硬编码密钥或密码