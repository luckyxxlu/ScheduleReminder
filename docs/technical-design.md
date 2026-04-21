# ScheduleReminder Technical Design

## 1. 文档目标

本文档定义 ScheduleReminder 的技术架构、模块拆分、数据结构、调度方案和前后端接口设计，作为首版开发实现依据。

## 2. 技术栈

### 2.1 总体技术选型
- 桌面容器：Tauri
- 前端：React + TypeScript
- 后端：Rust
- 数据库：MySQL

### 2.2 前端约束
- 前端统一使用 TypeScript。
- 页面、组件、hooks、store、工具函数统一使用 `*.ts` 和 `*.tsx`。
- 不使用 JavaScript 作为前端业务实现语言。

## 3. 总体架构

应用采用前后端分层结构：

- React + TypeScript 负责界面渲染、页面路由、表单交互和状态展示。
- Rust + Tauri 负责提醒调度、MySQL 持久化、托盘、系统通知和系统级能力。
- MySQL 保存提醒模板、提醒实例、操作日志和应用设置。

推荐数据流：

1. 前端调用 Tauri command 创建或更新提醒模板。
2. Rust 将模板写入 SQLite。
3. Rust 调度器根据模板生成未来提醒实例。
4. 到达触发时间后由 Rust 发出系统通知，并通过事件通知前端刷新界面。
5. 用户在弹窗或页面中执行动作后，前端调用 command 更新实例状态并写入操作日志。

## 4. 目录结构建议

```text
src/
  app/
    routes/
    providers/
  pages/
    today/
    calendar/
    reminders/
    settings/
  components/
    common/
    reminder/
    calendar/
    layout/
  features/
    reminder-template/
    reminder-occurrence/
    settings/
  hooks/
  lib/
    api/
    date/
    format/
    validation/
  store/
  types/
  styles/

src-tauri/
  src/
    commands/
    db/
    migrations/
    models/
    scheduler/
    notification/
    tray/
    settings/
    state/
    events/
    main.rs
```

## 5. 前端设计

## 5.1 页面划分

### TodayPage
职责：
- 展示当前时间、下一条提醒、今日时间线、宽容中的提醒。
- 处理今日页快捷操作，如新建提醒、完成提醒、跳过提醒。

### CalendarPage
职责：
- 渲染月视图。
- 查看选中日期的提醒详情。
- 从日期视图进入编辑提醒。

### RemindersPage
职责：
- 展示全部提醒模板。
- 支持筛选、启用/暂停、编辑、复制。

### SettingsPage
职责：
- 维护应用级设置，如默认宽容时间、托盘、通知、开机自启和主题。

## 5.2 前端状态管理

建议使用轻量 store，不做过度抽象。推荐状态拆分：

- `useAppStore`
  - 当前时间
  - 应用初始化状态
  - 当前主题

- `useReminderTemplateStore`
  - 模板列表
  - 当前筛选条件
  - 模板增删改状态

- `useReminderOccurrenceStore`
  - 今日提醒
  - 下一条提醒
  - 宽容中的提醒
  - 日历当前日期范围数据

- `useSettingsStore`
  - 默认宽容时间
  - 通知开关
  - 托盘设置
  - 开机自启
  - 免打扰时段

## 5.3 前端类型定义建议

```ts
export type ReminderStatus =
  | 'pending'
  | 'grace'
  | 'completed'
  | 'skipped'
  | 'missed'

export interface ReminderTemplate {
  id: string
  title: string
  category: string | null
  eventType: 'text' | 'system_action'
  eventPayload: Record<string, string | number | boolean | null>
  repeatRule: RepeatRule
  defaultGraceMinutes: number
  notifySound: boolean
  enabled: boolean
  note: string | null
  createdAt: string
  updatedAt: string
}

export interface ReminderOccurrence {
  id: string
  templateId: string
  scheduledAt: string
  graceDeadlineAt: string
  snoozedUntil: string | null
  status: ReminderStatus
  handledAt: string | null
}

export interface AppSettings {
  defaultGraceMinutes: number
  startupWithWindows: boolean
  trayEnabled: boolean
  theme: 'system' | 'light' | 'dark'
  quietHoursEnabled: boolean
  quietHoursStart: string | null
  quietHoursEnd: string | null
}
```

## 5.4 前端 API 封装建议

前端统一在 `src/lib/api/tauri.ts` 中封装 command 调用，页面不直接散落调用 `invoke`。

示例职责：
- `listReminderTemplates()`
- `createReminderTemplate(input)`
- `updateReminderTemplate(input)`
- `toggleReminderTemplate(input)`
- `listOccurrencesByDateRange(input)`
- `completeOccurrence(input)`
- `snoozeOccurrence(input)`
- `skipOccurrence(input)`
- `getAppSettings()`
- `updateAppSettings(input)`

## 6. Rust 后端设计

## 6.1 模块拆分

### `commands/`
对外暴露给 Tauri 前端调用的命令入口。

### `db/`
负责 MySQL 连接初始化、迁移执行、Repository 封装。

### `models/`
维护核心实体和数据库映射结构。

### `scheduler/`
负责提醒实例生成、近期提醒扫描、状态流转和补算逻辑。

### `notification/`
封装 Windows 通知触发逻辑。

### `tray/`
封装系统托盘菜单和托盘事件处理。

### `settings/`
负责应用设置读写、开机自启、免打扰策略判断。

### `events/`
负责向前端广播状态刷新事件。

### `state/`
保存 Tauri 应用共享状态，如数据库连接池、调度器句柄和配置缓存。

## 6.2 共享状态建议

```rust
pub struct AppState {
    pub db: Database,
    pub scheduler: SchedulerHandle,
    pub settings_cache: RwLock<AppSettings>,
}
```

## 7. 数据库设计

## 7.0 MySQL 使用约束

- 建议使用 MySQL 8.0 及以上版本。
- 后端通过连接池访问 MySQL。
- 应用启动时必须执行数据库连通性检查。
- migration 由后端启动流程统一执行。
- 开发、测试、生产环境使用独立数据库实例或独立 schema。

## 7.1 表结构

### `reminder_templates`

```sql
CREATE TABLE reminder_templates (
  id VARCHAR(64) PRIMARY KEY NOT NULL,
  title VARCHAR(255) NOT NULL,
  category VARCHAR(64) NULL,
  event_type VARCHAR(32) NOT NULL,
  event_payload_json JSON NOT NULL,
  repeat_rule_json JSON NOT NULL,
  default_grace_minutes INTEGER NOT NULL,
  notify_sound TINYINT(1) NOT NULL DEFAULT 1,
  note TEXT NULL,
  enabled TINYINT(1) NOT NULL DEFAULT 1,
  created_at DATETIME(3) NOT NULL,
  updated_at DATETIME(3) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
```

### `reminder_occurrences`

```sql
CREATE TABLE reminder_occurrences (
  id VARCHAR(64) PRIMARY KEY NOT NULL,
  template_id VARCHAR(64) NOT NULL,
  scheduled_at DATETIME(3) NOT NULL,
  grace_deadline_at DATETIME(3) NOT NULL,
  snoozed_until DATETIME(3) NULL,
  status VARCHAR(32) NOT NULL,
  handled_at DATETIME(3) NULL,
  created_at DATETIME(3) NOT NULL,
  updated_at DATETIME(3) NOT NULL,
  FOREIGN KEY(template_id) REFERENCES reminder_templates(id),
  UNIQUE(template_id, scheduled_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE INDEX idx_occurrences_template_id ON reminder_occurrences(template_id);
CREATE INDEX idx_occurrences_scheduled_at ON reminder_occurrences(scheduled_at);
CREATE INDEX idx_occurrences_status ON reminder_occurrences(status);
```

### `reminder_action_logs`

```sql
CREATE TABLE reminder_action_logs (
  id VARCHAR(64) PRIMARY KEY NOT NULL,
  occurrence_id VARCHAR(64) NOT NULL,
  action VARCHAR(64) NOT NULL,
  action_at DATETIME(3) NOT NULL,
  payload_json JSON NULL,
  FOREIGN KEY(occurrence_id) REFERENCES reminder_occurrences(id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE INDEX idx_action_logs_occurrence_id ON reminder_action_logs(occurrence_id);
```

### `app_settings`

```sql
CREATE TABLE app_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  default_grace_minutes INTEGER NOT NULL,
  startup_with_windows TINYINT(1) NOT NULL DEFAULT 0,
  tray_enabled TINYINT(1) NOT NULL DEFAULT 1,
  theme VARCHAR(32) NOT NULL DEFAULT 'system',
  quiet_hours_enabled TINYINT(1) NOT NULL DEFAULT 0,
  quiet_hours_start VARCHAR(8) NULL,
  quiet_hours_end VARCHAR(8) NULL,
  updated_at DATETIME(3) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
```

由于 MySQL 不支持 `CHECK (id = 1)` 作为可靠单例约束策略，建议应用层固定使用 `id = 1` 进行 upsert。

## 7.2 重复规则存储

为避免首版数据库设计过重，重复规则建议先以 MySQL JSON 保存到 `repeat_rule_json`。

建议格式：

```json
{
  "type": "weekly",
  "interval": 1,
  "weekdays": [1, 2, 3, 4, 5]
}
```

首版支持：
- `none`
- `daily`
- `workdays`
- `weekly`

## 8. 调度器设计

## 8.1 目标

调度器由 Rust 常驻管理，不依赖前端定时器，保证以下能力：

- 创建提醒后生成未来实例。
- 应用重启后恢复调度状态。
- 系统休眠恢复后补算错过的提醒。
- 宽容期到期后正确进入 `missed`。

## 8.2 调度策略

建议采用“窗口预生成 + 周期扫描”模式：

- 为每个启用模板预生成未来 7 到 14 天实例。
- 调度器每 30 秒或 60 秒扫描一次即将到期实例。
- 每次扫描时处理：
  - 即将触发的 `pending`
  - 宽容已结束的 `grace`
  - 需要补生成的未来实例

## 8.3 状态流转逻辑

### 通知派发
- 条件：`now >= scheduled_at` 且当前状态为 `pending`
- 行为：
  - 发送 Windows App Notification
  - 写入一条操作日志，例如 `notification_dispatched`
  - 发出前端刷新事件
  - 实例状态直接更新为 `grace`

### 宽容中
- 条件：提醒已进入宽容期，且当前时间小于 `snoozed_until ?? grace_deadline_at`
- 行为：
  - 保持 `grace`
  - 等待用户操作

### 超时未处理
- 条件：`now > snoozed_until ?? grace_deadline_at` 且当前状态为 `grace`
- 行为：
  - 状态更新为 `missed`
  - 记录操作日志
  - 可按设置决定是否进行二次通知

## 8.4 稍后提醒与宽容 10 分钟

不建议直接修改原始模板时间。

建议方式：
- 对当前 occurrence 使用 `snoozed_until` 记录实际延后截止时间。
- `宽容 10 分钟`：将 `snoozed_until` 设置为当前时间加 10 分钟。
- `稍后提醒`：将 `snoozed_until` 设置为当前时间加用户选择的 5/10/15/30 分钟。
- `grace_deadline_at` 表示原始宽容截止时间，`snoozed_until` 表示当前实际等待截止时间，二者语义分离。

## 8.5 事件类型与执行模型

### 文本提醒事件
- `event_type = text`
- 通过 Windows App Notification 展示标题、正文和快捷动作。
- 用户可点击通知主体或按钮进入应用处理。

示例 `event_payload_json`：

```json
{
  "message": "23:30 前请准备休息"
}
```

### 系统动作事件
- `event_type = system_action`
- 首版建议支持 `shutdown`。
- 通知内容必须明确展示动作类型和预计执行结果。
- 系统动作执行前，应允许用户取消、延后或跳过。
- 默认实现应采用“提醒确认后执行”，而不是静默直接执行。

示例 `event_payload_json`：

```json
{
  "action": "shutdown",
  "message": "到时间了，准备关机休息"
}
```

## 9. Command 设计

## 9.1 提醒模板相关

- `list_reminder_templates`
- `create_reminder_template`
- `update_reminder_template`
- `delete_reminder_template`
- `toggle_reminder_template_enabled`
- `duplicate_reminder_template`

### `create_reminder_template` 输入示例

```json
{
  "title": "睡觉",
  "category": "rest",
  "repeatRule": {
    "type": "daily",
    "interval": 1
  },
  "defaultGraceMinutes": 15,
  "notifySound": true,
  "note": "23:30 前准备休息"
}
```

## 9.2 提醒实例相关

- `list_occurrences_by_date_range`
- `list_today_occurrences`
- `get_next_occurrence`
- `complete_occurrence`
- `skip_occurrence`
- `snooze_occurrence`
- `open_occurrence_detail`

### `snooze_occurrence` 输入示例

```json
{
  "occurrenceId": "occ_123",
  "minutes": 10,
  "reason": "grace_action"
}
```

## 9.3 设置相关

- `get_app_settings`
- `update_app_settings`
- `set_launch_on_startup`

## 10. 前后端事件设计

前端除了主动拉取，也应监听 Rust 推送事件。

建议事件：
- `reminder-triggered`
- `reminder-updated`
- `settings-updated`
- `scheduler-resynced`

事件用途：
- 当有提醒触发时，今天页和提醒列表可立即刷新。
- 当系统唤醒后完成补算时，日历和今日页自动更新。

通知激活处理参考 Windows App Notification 模型：
- 通知可携带 arguments。
- 用户点击通知主体或通知按钮后，应用根据 arguments 执行打开详情、完成、延后或系统动作确认。

## 11. 通知与托盘设计

## 11.1 通知策略

首版支持：
- Windows 系统通知
- 应用内提醒弹窗
- 可选声音提示

通知内容建议包含：
- 标题
- 原定提醒时间
- 宽容截止时间
- 快捷动作提示

对于系统动作事件：
- 通知正文应明确动作，例如“将在确认后执行关机”。
- 若使用按钮动作，建议包括 `延后`、`取消`、`打开详情`。

## 11.2 托盘菜单建议

- 打开今天页
- 快速新建提醒
- 暂停全部提醒 1 小时
- 打开设置
- 退出应用

## 11.3 关闭行为

- 点击窗口关闭按钮时默认最小化到托盘。
- 在设置中允许用户选择“关闭即退出”。

## 12. 时间处理策略

- 数据库存储统一使用 ISO 8601 文本。
- 前端展示时按本地时区格式化。
- 首版只考虑单时区本地桌面使用场景，不实现跨时区调度。

## 13. 表单校验建议

前端和后端都需要校验。

至少校验：
- 标题不能为空。
- 宽容时间不能小于 0。
- 重复规则必须合法。
- 免打扰时间段开始和结束值必须完整成对。

## 14. 测试建议

## 14.0 开发流程约束

- 后端开发采用测试先行方式。
- 对 Rust 模块、调度器逻辑、数据库 repository、系统动作执行器和辅助脚本，必须先写测试，再写实现。
- 测试未完成时，不应提交对应后端功能实现。
- 功能实现完成后，必须运行测试并以测试结果作为首要验收依据。

## 14.1 前端
- 组件渲染测试
- 表单校验测试
- 状态切换测试
- 时间线和日历展示测试

## 14.2 后端
- 重复规则解析测试
- 提醒实例生成测试
- 状态流转测试
- 系统重启恢复测试
- 宽容期超时测试
- 稍后提醒与 `snoozed_until` 计算测试
- `notification_dispatched` 日志写入测试
- 系统动作事件测试，例如 `shutdown` 提醒确认、取消和延后测试
- MySQL migration 执行测试
- MySQL 连接失败与重试/报错测试
- MySQL 唯一约束和事务回滚测试

后端测试覆盖要求：
- 覆盖正常路径
- 覆盖边界条件
- 覆盖异常路径
- 覆盖关键状态流转和数据库唯一约束

## 14.3 联调重点
- 新建提醒后是否立即出现在今天页和提醒页
- 到点后是否正常触发通知
- 托盘常驻时是否仍可触发通知
- 应用重启后是否丢失待触发提醒

## 15. 实现优先级

### Phase 1
- SQLite 初始化
- 提醒模板 CRUD
- 设置读写

### Phase 2
- 先完成提醒能力相关测试用例
- 调度器基础扫描
- 提醒实例生成
- 到点通知

### Phase 3
- 今天页
- 提醒管理页
- 设置页

### Phase 4
- 月历视图
- 日期详情面板
- 提醒交互弹窗

### Phase 5
- 托盘
- 开机自启
- 免打扰
- 稳定性优化

## 16. 决策摘要

- 产品聚焦提醒，不做复杂待办。
- 前端统一使用 TypeScript，不使用 JavaScript。
- 提醒调度统一在 Rust 侧实现。
- MySQL 作为持久化方案。
- 首版重复规则保持简单，避免过度设计。
