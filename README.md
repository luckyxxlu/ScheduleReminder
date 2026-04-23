# ScheduleReminder

一个可在 Windows 桌面打开使用的作息提醒应用。

## 当前可用能力

- 今天 / 日历 / 提醒 / 设置 四个页面骨架
- MySQL 初始化与 migration
- 提醒模板 CRUD 领域逻辑
- 重复规则解析
- occurrence 生成
- 调度扫描、宽容、延后、missed 状态流转
- 文本提醒与关机提醒执行模型
- 托盘菜单与开机自启 / 免打扰逻辑模型

## 运行方式

默认情况下，应用会尝试使用以下 MySQL 连接：

```bash
mysql://root:root@127.0.0.1:3306/schedule_reminder
```

应用启动时会自动创建 `schedule_reminder` 数据库。

如果你想手动指定连接字符串：

```bash
set DATABASE_URL=mysql://<user>:<password>@127.0.0.1:3306/<database>
```

然后启动桌面应用：

```bash
npm run tauri -- dev
```

## 测试与构建

```bash
npm test
npm run build
cd src-tauri
cargo test --lib
```

## 自动打包与发布

- GitHub Actions 工作流：`.github/workflows/windows-x86-release.yml`
- 触发方式：
  - 手动触发 `workflow_dispatch`
- 产物：
  - Windows x86（`i686-pc-windows-msvc`）NSIS 安装包
  - 工作流会自动创建 GitHub Release，并上传可直接下载安装的 `.exe` 安装包
  - 正式分发请以 Release 页面中的安装包资产为准
