# MCP 服务器参考（`tl-mcp-server`）

让 AI Agent（Claude / Cursor 等）直接运行 m 代码并取回图窗数据。

- 二进制：`tl-mcp-server`（`apps/tl-mcp-server`）
- 传输：**stdio**，逐行 JSON-RPC 2.0
- 协议版本：`2025-06-18`

```bash
cargo run -p tl-mcp-server      # 直接启动（读 stdin，写 stdout）
```

## 客户端配置（示例）

```json
{
  "mcpServers": {
    "tensorlab": {
      "command": "F:\\软件开发\\TensorLab\\target\\release\\tl-mcp-server.exe"
    }
  }
}
```

## 支持的方法

| 方法 | 说明 |
|---|---|
| `initialize` | 握手。返回 `protocolVersion`、`capabilities.tools`、`serverInfo` |
| `notifications/initialized` | 通知。**不产生响应**（JSON-RPC 2.0 要求） |
| `tools/list` | 返回 5 个工具及其 `inputSchema` |
| `tools/call` | 调用工具 |

未知方法 → 返回 `{"jsonrpc":"2.0","id":<id>,"result":null}`（不崩溃）。

### `initialize` 响应

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2025-06-18",
    "capabilities": { "tools": {} },
    "serverInfo": { "name": "tensorlab", "version": "0.1.0" }
  }
}
```

## 工具

所有工具返回 MCP 的 content 数组形式：`[{ "type": "text", "text": "..." }]`。

### `run`

执行磁盘上的 `.m` 脚本。

```json
{ "name": "run", "arguments": { "path": "demos/hello_world.m" } }
```

- 成功：控制台文本
- 路径不可读：文本以 `error: ` 开头
- 脚本运行出错：文本内含 `ERROR: <message>`

### `eval`

执行一段 m 代码。

```json
{ "name": "eval", "arguments": { "code": "2 + 3*4" } }
```

返回控制台文本。表达式语句的内核回显形如 `ans = 14`，故文本中包含结果值。
运行出错时文本内附 `ERROR: ...`、警告附 `WARNING: ...`。

### `plot`

执行代码并**返回图窗数据**。

```json
{ "name": "plot", "arguments": { "code": "figure; plot(1:5);" } }
```

返回文本形如：

```
<控制台文本>
figures: [ { "createdAt": "...", "figure": { "layout": {...}, "metadata": {...}, "plots": [...] } } ]
```

`figures:` 之后是 FigureScene 的 JSON 数组（已剥离 `*_source` 冗余字段，见 [decisions.md](decisions.md) ADR-007）。
Agent 可直接消费该结构做图形理解或二次渲染。

### `vars`

列出工作区变量，返回 **JSON 数组**（`VariableInfo` 的 camelCase 形式）：

```json
[ { "name": "x", "className": "double", "shape": [1,5], "bytes": 40,
    "preview": "[1 2 3 4 5]", "isGpu": false } ]
```

`preview` 上限 16 个元素（内核 `WORKSPACE_PREVIEW_LIMIT`）；非数值类型为空串。

### `reset`

清空工作区与图窗（`close all; clear;`），返回 `workspace reset.`。

## 协议健壮性（均有测试覆盖）

| 输入 | 行为 |
|---|---|
| 空行 / 纯空白行 | 忽略 |
| 非 JSON 行 | 忽略，不产生响应，会话继续 |
| 通知（无 `id`） | 不产生响应 |
| 未知工具 | 返回文本 `error: unknown tool: <name>` |
| 未知方法 | 返回 `result: null` |
| 多请求连续下发 | 逐个应答，`id` 一一对应 |
| stdin EOF | 服务器正常退出 |

## 实现要点

- 引擎跑在 **64 MiB 大栈线程**上（内核 parser/eval 深递归会爆主线程栈，见 ADR-006）
- 会话内**复用同一个 `Engine`**，因此 `eval` → `vars` → `plot` 之间工作区是持续的
- `tools/call` 的未知工具**不**返回 JSON-RPC error，而是文本 `error: ...`（便于 Agent 直接阅读）

## 手工测试

```powershell
@'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"eval","arguments":{"code":"2 + 3*4"}}}
'@ | target\debug\tl-mcp-server.exe
```

或直接跑黑盒测试：`cargo test -p tl-mcp-server --test mcp_stdio`
