# GitHub Runner Token 设置指南

## 问题诊断

您遇到的 404 错误原因：
- PAT 缺少必要的权限
- 需要仓库的 admin 权限才能创建 runner registration token

## 解决方案

### 方案 1：使用 Fine-grained Personal Access Token（推荐）

1. 访问 GitHub Token 设置页面：
   https://github.com/settings/personal-access-tokens/new

2. 配置 Token：
   - **Token name**: aurelia-runner
   - **Expiration**: 选择合适的过期时间（建议 90 天）
   - **Repository access**: 选择 "Selected repositories"
   - 选择仓库：`tricorefile/aurelia`

3. 设置权限（重要！）：
   ```
   Repository permissions:
   ✅ Actions: Read
   ✅ Administration: Read and Write  ← 关键权限
   ✅ Contents: Read
   ✅ Metadata: Read (自动选中)
   ✅ Pull requests: Read
   ```

4. 点击 "Generate token"

5. 复制生成的 token（以 `github_pat_` 开头）

### 方案 2：使用 Classic Personal Access Token

1. 访问：https://github.com/settings/tokens/new

2. 配置：
   - **Note**: aurelia-runner
   - **Expiration**: 选择过期时间
   - **Scopes** 选择：
     ```
     ✅ repo (全部)
     ✅ workflow
     ✅ admin:repo_hook
     ✅ admin:org (如果是组织仓库)
     ```

3. 生成并复制 token（以 `ghp_` 开头）

### 方案 3：直接使用 Registration Token（最简单）

1. 访问：https://github.com/tricorefile/aurelia/settings/actions/runners

2. 点击 "New self-hosted runner"

3. 选择 "Linux"

4. 在 "Configure" 部分，复制命令中的 token：
   ```bash
   ./config.sh --url https://github.com/tricorefile/aurelia --token XXXXXXXXXX
   ```
   复制这个 `XXXXXXXXXX` 部分

5. **注意**：这个 token 只有 1 小时有效期，适合快速测试

## 验证 Token

### 测试 PAT 权限：
```bash
# 测试基本认证
curl -H "Authorization: token YOUR_TOKEN" \
     https://api.github.com/user

# 测试仓库访问
curl -H "Authorization: token YOUR_TOKEN" \
     https://api.github.com/repos/tricorefile/aurelia

# 测试 runner 权限
curl -X POST \
     -H "Authorization: token YOUR_TOKEN" \
     -H "Accept: application/vnd.github.v3+json" \
     https://api.github.com/repos/tricorefile/aurelia/actions/runners/registration-token
```

成功响应示例：
```json
{
  "token": "AARAAXXXXXXXXXXXXXXXXXX",
  "expires_at": "2024-01-01T12:00:00.000Z"
}
```

## 使用 Token

创建 `.env` 文件：
```bash
echo "GITHUB_TOKEN=your_token_here" > .env
```

然后启动 runner：
```bash
docker compose up -d
```

## 常见问题

### Q: 为什么返回 404 而不是 403？
A: GitHub 出于安全考虑，对没有权限的请求返回 404，避免暴露仓库是否存在。

### Q: 需要是仓库 owner 吗？
A: 不一定是 owner，但需要仓库的 admin 权限。

### Q: Registration token 多久过期？
A: 1 小时。PAT 不会过期（除非设置了过期时间）。

### Q: 组织仓库需要什么额外权限？
A: 需要组织的 owner 权限或 `manage_runners` 权限。

## 权限矩阵

| Token 类型 | 仓库级 Runner | 组织级 Runner | 有效期 |
|-----------|--------------|--------------|--------|
| Registration Token | ✅ 直接使用 | ✅ 直接使用 | 1小时 |
| Classic PAT + repo scope | ✅ 需要 admin | ⚠️ 需要 org owner | 自定义 |
| Fine-grained PAT | ✅ 需要 Administration 权限 | ✅ 需要特定权限 | 自定义 |

## 推荐方案

1. **开发测试**：使用 Registration Token（简单快速）
2. **生产环境**：使用 Fine-grained PAT（安全、权限精确）
3. **CI/CD**：使用 GitHub App（最安全，但配置复杂）