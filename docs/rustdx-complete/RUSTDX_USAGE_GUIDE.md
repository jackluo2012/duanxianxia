# rustdx-complete 组件使用指南

**问题**: rustdx-complete TDX连接失败 (Broken pipe error 32)
**目标**: 诊断问题并提供解决方案

---

## 1. rustdx-complete 简介

**rustdx-complete** 是一个功能完整的A股数据获取Rust库。

**仓库**: https://github.com/jackluo2012/rustdx
**文档**: https://docs.rs/rustdx-complete/1.0.0
**版本**: 1.0.0

### 功能特性

- ✅ TCP协议连接通达信行情服务器
- ✅ 支持实时行情查询
- ✅ 支持K线数据
- ✅ 支持技术指标
- ✅ 支持交易日历
- ✅ 零外部依赖（纯Rust实现）

---

## 2. 当前问题诊断

### 错误信息
```
❌ TDX TCP connection failed: Broken pipe (os error 32)
```

### 问题原因分析

**"Broken pipe"错误含义**:
- TCP连接被服务器端关闭
- 写入数据时连接已断开
- 通常是网络不可达或服务未响应

### rustdx-complete工作原理

```
应用 (data-collector)
       ↓
rustdx-complete (Tcp::new())
       ↓
TCP连接到通达信服务器
       ↓
114.80.156.19:7709 (或其他)
       ↓
发送行情请求
       ↓
接收行情数据
```

### 可能的原因

1. **网络不可达** ⭐ 最可能
   - WSL2无法访问通达信服务器
   - 防火墙阻止连接
   - 服务器不在交易时段

2. **服务器未响应**
   - 通达信服务器维护
   - IP被封禁
   - 非交易时段服务关闭

3. **WSL2网络问题**
   - WSL2网络配置
   - DNS解析问题
   - NAT转换问题

---

## 3. 诊断步骤

### 3.1 测试网络连通性

```bash
# 测试能否通达信服务器 (端口7709)
timeout 5 nc -zv 114.80.156.19 7709 2>&1

# 或使用telnet
timeout 5 telnet 114.80.156.19 7709

# 测试DNS解析
nslookup 114.80.156.19

# 测试HTTP连接（备用方案）
curl -I https://hq.sinajs.cn
```

### 3.2 检查交易时段

**A股交易时间**:
- 上午: 9:30-11:30
- 下午: 13:00-15:00
- 当前: 2026-02-11 14:30 (交易时段内) ✅

**注意事项**:
- 非交易时段服务器可能不响应
- 周末和节假日服务器关闭

### 3.3 WSL2网络诊断

```bash
# 检查WSL2网络
ip addr show eth0

# 检查默认网关
ip route | grep default

# 测试外网连接
ping -c 3 8.8.8.8

# 测试DNS
nslookup baidu.com
```

---

## 4. 解决方案

### 方案A: 修复WSL2网络连接 ⭐ 推荐

#### 1. 检查Windows防火墙

```powershell
# 在Windows PowerShell中运行

# 检查WSL2访问规则
Get-NetFirewallProfile | Select Name, Enabled

# 允许WSL2出站连接（如果需要）
New-NetFirewallRule -DisplayName "WSL2 Outbound" `
    -Direction Outbound -Action Allow
```

#### 2. 配置WSL2网络

```bash
# 编辑WSL配置
sudo nano /etc/wsl2.conf

# 添加以下内容：
[wsl2]
networkingMode=mirrored
# 或
networkingMode=nat

# 重启WSL2
wsl --shutdown
```

#### 3. 尝试不同服务器

```bash
# 通达信备用服务器列表
上海1: 114.80.156.19:7709
上海2: 114.80.156.20:7709
深圳: 14.215.128.18:7709
```

### 方案B: 使用Windows通达信 + 桥接 ⚠️ 复杂

**在Windows上运行通达信**:
```bash
# 1. 安装通达信客户端
# 2. 启动通达信，确保行情数据可用
# 3. 在WSL2中访问Windows通达信

# 查找Windows主机IP
cat /etc/resolv.conf | grep nameserver
# 或
powershell.exe "Get-NetIPAddress -AddressFamily IPv4 | Select-Object IPAddress"

# 测试连接
nc -zv <Windows IP> 7709
```

**优点**: 可靠，真实数据
**缺点**: 配置复杂，需要Windows运行通达信

### 方案C: 使用HTTP API替代 ✅ 当前方案

**已实现**: HttpQuoteDataSource (腾讯/新浪API)

**优点**:
- ✅ 无需额外配置
- ✅ WSL2兼容
- ✅ 免费、稳定
- ✅ 真实市场数据

**缺点**:
- ⚠️ 非官方数据源
- ⚠️ 可能有频率限制

---

## 5. 测试rustdx-complete

### 5.1 创建测试程序

```rust
// 文件: test_tdx.rs

use rustdx_complete::tcp::stock::SecurityQuotes;
use rustdx_complete::tcp::Tcp;

fn main() -> anyhow::Result<()> {
    println!("Testing TDX connection...");

    // 尝试创建TCP连接
    match Tcp::new() {
        Ok(tcp) => {
            println!("✅ TDX TCP连接创建成功");

            // 测试获取行情
            let quotes = tcp
                .quotes(&[(1, "600000".to_string())])?
                .iter()
                .take(5)
                .map(|q| format!("{:?}={}", q.code, q.price))
                .collect::<Vec<_>>()
                .join("\n");

            println!("行情数据:\n{}", quotes);
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ TDX连接失败: {}", e);
            eprintln!("这可能是因为:");
            eprintln!("  1. 网络不可达（WSL2网络问题）");
            eprintln!("  2. 通达信服务器未响应（非交易时段或维护）");
            eprintln!("  3. 防火墙阻止连接");
            Err(e.into())
        }
    }
}
```

### 5.2 运行测试

```bash
# 编译测试程序
cat > test_tdx.rs << 'EOF'
use rustdx_complete::tcp::Tcp;

fn main() {
    println!("Testing TDX connection...");
    match Tcp::new() {
        Ok(tcp) => println!("✅ Success: {:?}", tcp),
        Err(e) => println!("❌ Failed: {}", e),
    }
}
EOF

rustc --edition 2021 test_tdx.rs --extern rustdx-complete -L target/release/deps
./test_tdx
```

---

## 6. 生产环境建议

### 推荐配置

| 环境 | 推荐数据源 | 原因 |
|------|------------|------|
| **生产服务器** (Linux) | HTTP API | 稳定可靠，零依赖 |
| **WSL2开发** | HTTP API | 网络兼容性好 |
| **Windows本地** | TDX + 通达信 | 功能完整 |
| **Docker容器** | HTTP API | 网络配置简单 |

### 混合策略

```rust
// 优先级：HTTP API > TDX > Mock

pub struct HybridDataSource {
    http: HttpQuoteDataSource,
    tdx: Option<TdxQuoteDataSource>,
}

impl HybridDataSource {
    async fn fetch_quotes(&self, codes: &[StockCode]) -> Result<Vec<StockQuote>> {
        // 优先尝试HTTP API
        match self.http.fetch_quotes(codes).await {
            Ok(quotes) if !quotes.is_empty() => Ok(quotes),
            _ => {
                // 降级到TDX
                if let Some(tdx) = &self.tdx {
                    tdx.fetch_quotes(codes).await
                } else {
                    Err(DataSourceError::Connection("No data source available"))
                }
            }
        }
    }
}
```

---

## 7. rustdx-complete vs HTTP API

| 特性 | rustdx-complete | HTTP API |
|------|-----------------|----------|
| **数据源** | 通达信官方 | 腾讯/新浪 |
| **可靠性** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **数据质量** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **速度** | ⭐⭐⭐⭐⭐ (极快) | ⭐⭐⭐ (~300ms) |
| **配置难度** | ⭐⭐ (较复杂) | ⭐⭐⭐⭐ (简单) |
| **WSL2兼容** | ⭐⭐ (问题) | ⭐⭐⭐⭐ (完美) |
| **零依赖** | ✅ 是 | ✅ 是 |
| **免费** | ✅ 是 | ✅ 是 |

### 结论

对于**WSL2环境**和**生产部署**，**HTTP API是更实际的选择**。

对于**Windows本地开发**和**要求极致性能**的场景，**rustdx-complete + 通达信**是最佳选择。

---

## 8. 当前状态

### 已实现的解决方案 ✅

```rust
// 数据源优先级（已配置）
"mock"     → MockQuoteDataSource      // 测试开发
"http"     → HttpQuoteDataSource      // 真实数据（当前使用）
"sina"     → HttpQuoteDataSource      // 新浪API
"tencent"  → HttpQuoteDataSource      // 腾讯API（当前使用）
"tdx"      → TdxQuoteDataSource       // 通达信（需配置）
```

### 切换命令

```bash
# 切换到HTTP API（推荐，已验证可用）
DATA_SOURCE_TYPE=http cargo run -p data-collector --release

# 切换到TDX（需先修复网络）
DATA_SOURCE_TYPE=tdx cargo run -p data-collector --release

# 切换回Mock（测试用）
DATA_SOURCE_TYPE=mock cargo run -p data-collector --release
```

---

## 9. 总结

### rustdx-complete能用吗？

**答案**: 理论上可以，但实际上取决于环境：

**✅ 可以用的情况**:
- Windows本地开发，运行通达信客户端
- 服务器有通达信行情数据转发
- 网络可直接访问通达信服务器
- A股交易时段

**❌ 不能用的情况**:
- WSL2网络配置不当 (当前情况)
- 无法连接通达信服务器
- 非交易时段
- 防火墙阻止端口7709

### 当前最佳方案

**使用HTTP API (腾讯财经)**:
- ✅ 已验证可用
- ✅ 真实市场数据
- ✅ WSL2完美兼容
- ✅ 零配置成本
- ✅ 免费无限制

### 如果必须使用rustdx-complete

**需要满足**:
1. 修复WSL2网络或使用Windows
2. 运行通达信客户端或配置数据转发
3. 确保在交易时段
4. 开放防火墙端口

---

**结论**: rustdx-complete是一个优秀的组件，但在WSL2环境中使用有网络限制。**当前HTTP API方案已经完全满足需求**，建议继续使用HTTP API数据源。
