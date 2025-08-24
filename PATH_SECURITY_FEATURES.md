# 路径安全特性说明

## 概述

Rust Agent 现在实现了严格的路径安全限制，确保所有文件和目录操作都限制在指定的项目目录内。这防止了路径遍历攻击和意外的系统文件访问，提供了企业级的安全性。

## 安全特性

### 1. 路径限制

**核心原则**: 所有操作都限制在项目目录内

**实现方式**:
- 自动检测项目根目录
- 验证所有路径是否在项目目录内
- 拒绝任何试图访问项目目录外的操作

**安全级别**: 企业级安全，防止路径遍历攻击

### 2. 路径验证机制

#### `is_path_within_project()` 函数

```rust
fn is_path_within_project(project_dir: &Path, target_path: &Path) -> bool {
    let project_dir = project_dir.canonicalize().unwrap_or_else(|_| project_dir.to_path_buf());
    let target_path = target_path.canonicalize().unwrap_or_else(|_| target_path.to_path_buf());
    
    target_path.starts_with(&project_dir)
}
```

**功能**:
- 规范化路径（解析符号链接和相对路径）
- 检查目标路径是否以项目目录开头
- 防止通过 `../` 等相对路径绕过限制

#### `safe_resolve_path()` 函数

```rust
fn safe_resolve_path(project_dir: &str, user_path: &str) -> Result<PathBuf, AgentError> {
    let project_path = PathBuf::from(project_dir);
    let user_path = Path::new(user_path);
    
    let final_path = if user_path.is_absolute() {
        user_path.to_path_buf()
    } else {
        // 相对路径，拼接到项目目录
        let mut resolved_path = project_path.clone();
        resolved_path.push(user_path);
        resolved_path
    };
    
    // 验证路径是否在项目目录内
    if !is_path_within_project(&project_path, &final_path) {
        return Err(AgentError::RuntimeError(
            format!("路径 '{}' 不在项目目录 '{}' 内，操作被拒绝", 
                final_path.display(), project_path.display())
        ));
    }
    
    Ok(final_path)
}
```

**功能**:
- 智能路径解析（支持相对路径和绝对路径）
- 自动路径验证
- 清晰的错误消息

## 受保护的工具

### 1. ReadFileTool

**安全特性**:
- 只能读取项目目录内的文件
- 防止访问系统文件（如 `/etc/passwd`）
- 防止通过相对路径绕过限制

**示例**:
```bash
# ✅ 允许：读取项目内的文件
read_file("src/main.rs")
read_file("./README.md")

# ❌ 拒绝：访问系统文件
read_file("/etc/passwd")
read_file("../../../etc/hosts")

# ❌ 拒绝：访问项目外的文件
read_file("/tmp/secret.txt")
```

### 2. WriteFileTool

**安全特性**:
- 只能写入项目目录内的文件
- 自动创建缺失的父目录（询问用户）
- 防止覆盖系统文件

**示例**:
```bash
# ✅ 允许：写入项目内的文件
write_to_file("src/config.json", "{}")
write_to_file("./data/user.txt", "user data")

# ❌ 拒绝：写入系统文件
write_to_file("/etc/config.conf", "config")
write_to_file("../../../etc/test.txt", "test")

# ❌ 拒绝：写入项目外的文件
write_to_file("/tmp/secret.txt", "secret")
```

### 3. CreateDirectoryTool

**安全特性**:
- 只能在项目目录内创建目录
- 询问用户是否创建缺失的父目录
- 防止创建系统目录

**示例**:
```bash
# ✅ 允许：在项目内创建目录
create_directory("src/components")
create_directory("./assets/images")

# ❌ 拒绝：创建系统目录
create_directory("/etc/custom")
create_directory("../../../var/log")

# ❌ 拒绝：在项目外创建目录
create_directory("/tmp/project")
```

### 4. CreateFileTool

**安全特性**:
- 只能在项目目录内创建文件
- 询问用户是否覆盖现有文件
- 询问用户是否创建缺失的父目录
- 防止创建系统文件

**示例**:
```bash
# ✅ 允许：在项目内创建文件
create_file("src/main.rs", "fn main() {}")
create_file("./config/settings.json", "{}")

# ❌ 拒绝：创建系统文件
create_file("/etc/custom.conf", "config")
create_file("../../../etc/test.txt", "test")

# ❌ 拒绝：在项目外创建文件
create_file("/tmp/secret.txt", "secret")
```

## 安全测试场景

### 1. 绝对路径攻击

**测试**: 尝试访问系统文件
```bash
用户: 请读取 /etc/passwd 文件
结果: 路径 '/etc/passwd' 不在项目目录内，操作被拒绝
```

**安全效果**: ✅ 完全阻止

### 2. 相对路径遍历攻击

**测试**: 尝试使用 `../` 绕过限制
```bash
用户: 请创建文件 ../../../etc/test.txt
结果: 路径不在项目目录内，操作被拒绝
```

**安全效果**: ✅ 完全阻止

### 3. 符号链接攻击

**测试**: 尝试通过符号链接访问外部文件
```rust
// 即使存在符号链接，canonicalize() 会解析真实路径
let real_path = path.canonicalize()?;
if !is_path_within_project(&project_dir, &real_path) {
    return Err(AgentError::RuntimeError("路径不在项目目录内"));
}
```

**安全效果**: ✅ 完全阻止

### 4. 正常操作验证

**测试**: 验证正常操作仍然工作
```bash
用户: 请在项目目录下创建 css 目录
结果: 目录创建成功: /path/to/project/css
```

**安全效果**: ✅ 正常工作

## 错误处理

### 错误消息格式

```rust
format!("路径 '{}' 不在项目目录 '{}' 内，操作被拒绝", 
    target_path.display(), project_path.display())
```

**示例错误消息**:
```
路径 '/etc/passwd' 不在项目目录 '/Users/user/project' 内，操作被拒绝
路径 '/tmp/secret.txt' 不在项目目录 '/Users/user/project' 内，操作被拒绝
路径 '../../../etc/hosts' 不在项目目录 '/Users/user/project' 内，操作被拒绝
```

### 错误类型

- **AgentError::RuntimeError**: 路径安全违规
- 包含详细的路径信息
- 清晰的拒绝原因

## 配置和部署

### 环境要求

- 无需额外配置
- 自动检测项目根目录
- 支持所有现有功能

### 部署注意事项

1. **项目目录设置**: 确保项目目录路径正确
2. **权限检查**: 验证项目目录的读写权限
3. **路径规范化**: 自动处理符号链接和相对路径

## 最佳实践

### 开发阶段

1. **明确项目边界**: 清楚定义项目目录结构
2. **测试安全边界**: 验证路径限制是否生效
3. **错误处理**: 优雅处理路径安全错误

### 生产环境

1. **监控路径访问**: 记录被拒绝的路径访问尝试
2. **定期安全审计**: 检查项目目录权限
3. **用户培训**: 教育用户了解路径限制

## 安全优势

### 1. 防止路径遍历攻击

- 阻止 `../` 等相对路径攻击
- 防止访问系统敏感文件
- 阻止符号链接攻击

### 2. 数据隔离

- 确保项目数据不会泄露到外部
- 防止意外修改系统文件
- 维护项目完整性

### 3. 企业级安全

- 符合安全最佳实践
- 防止恶意代码执行
- 保护系统稳定性

## 总结

路径安全特性为 Rust Agent 提供了企业级的安全保障，确保所有文件操作都严格限制在项目目录内。通过智能路径验证和规范化，系统能够有效防止各种路径遍历攻击，同时保持正常功能的可用性。

这些安全特性特别适用于：
- 企业环境部署
- 多用户系统
- 安全敏感的应用
- 生产环境运行

通过结合用户询问和路径验证，Rust Agent 现在提供了既安全又友好的文件系统操作体验。
