# 詳細設計 UML

本文件放[詳細設計](detailed-design.md)的圖，文字說明以 detailed-design.md 為準。圖以 Mermaid 11 撰寫；Mermaid 沒有套件圖，以 flowchart 表示，箭頭由依賴的一方指向被依賴的一方。

## 1. 套件圖 P1：crate

對應 §3.1。`rosamta-platform` 到 `rosamta-core` 的箭頭同時表示實作 core 定義的 trait。`rosamta-testkit` 只供測試使用，哪些測試使用它見 §8。

```mermaid
flowchart TB
    bin["rosamta<br/>執行檔：命令列介面"]
    platform["rosamta-platform<br/>unix、windows、host、git"]
    core["rosamta-core<br/>服務與資料型別<br/>trait：FileSystem、GitClient"]
    testkit["rosamta-testkit<br/>故障注入（只作 dev-dependency）"]
    rustix(["rustix"])
    winsys(["windows-sys"])
    libs(["toml_edit、minijinja、serde、serde_json、sha2"])
    binlibs(["clap、similar、serde、serde_json"])

    bin --> platform
    bin --> core
    bin --> binlibs
    platform -- "實作 FileSystem、GitClient" --> core
    platform -- "cfg(unix)" --> rustix
    platform -- "cfg(windows)" --> winsys
    core --> libs
    testkit --> core
    testkit --> platform
```

## 2. 套件圖 P2：core 的 module 分層

對應 §3.2。每個方框是一層，箭頭表示上層可以依賴下層。同層之間允許的依賴以虛線畫出，其餘同層依賴都不允許；每個 module 可依賴的完整清單見 §3.2 的表格。

```mermaid
flowchart TB
    subgraph L3["第 3 層：子指令"]
        apply["apply"]
        manage["manage"]
        query["query"]
        init["init"]
    end
    subgraph L2["第 2 層：流程"]
        plan["plan"]
        preflight["preflight"]
        recovery["recovery"]
        exec["exec"]
    end
    subgraph L1["第 1 層：資料"]
        config["config"]
        docedit["docedit"]
        resolve["resolve"]
        template["template"]
        state["state"]
    end
    subgraph L0["第 0 層：基礎"]
        diag["diag"]
        name["name"]
        host["host"]
        path["path"]
        fs["fs<br/>trait FileSystem"]
        layout["layout"]
    end

    L3 --> L2
    L2 --> L1
    L1 --> L0

    init -.-> apply
    preflight -.-> plan
    exec -.-> plan
    exec -.-> recovery
    resolve -.-> config
    template -.-> config
    diag -.-> name
    diag -.-> path
    path -.-> host
    fs -.-> path
    layout -.-> path
```

層與層的箭頭表示可以依賴任何較低的層，例如第 3 層的 `manage` 直接使用第 1 層的 `docedit`。`name`、`host`、`path` 以外的 module 都使用 `diag`，這類依賴不逐一畫出；這三個 module 不使用 `diag`，因為 `diag` 的來源位置要用到名稱與路徑，反過來會形成循環。

## 3. 詳細類別圖 D1：設定資料型別

對應 §4.2，細化 03 的 C1、C2。圖中省略 `Located` 的包裝與 `SourceLocation` 欄位；enum 以 `<<enumeration>>` 標示，成員是它的變體。`FieldType::check` 是欄位值從 `RawValue` 轉成 `FieldValue` 的唯一途徑。

```mermaid
classDiagram
    direction TB
    class EffectiveConfig {
        +manifest Manifest
        +modules ModuleSet
        +profile Profile
        +assignments Vec~Assignment~
        +manifest_file Option~LoadedFile~
        +is_complete() bool
        +is_declared(name) bool
    }
    class Manifest {
        +get(name) Option~OutputDefinition~
        +outputs() Iterator
        +removed_builtins() Iterator
    }
    class OutputDefinition {
        +name OutputName
        +kind OutputKind
        +source SourceSpec
        +targets PlatformTargets
        +ownership Ownership
        +permissions Option~Permissions~
        +takeover bool
    }
    class SourceSpec {
        <<enumeration>>
        Template(TemplateRef)
        Verbatim(SourceRelPath)
    }
    class TemplateRef {
        <<enumeration>>
        Builtin(BuiltinTemplate)
        User(SourceRelPath)
    }
    class PlatformTargets {
        +get(platform) Option~RawPath~
    }
    class ModuleSet {
        +get(name) Option~ModuleDecl~
        +field(ref) Option~FieldDecl~
    }
    class ModuleDecl {
        +name ModuleName
        +builtin bool
        +version Option~u32~
    }
    class FieldDecl {
        +name FieldName
        +ty FieldType
        +required bool
        +description Option~String~
        +default Option~FieldValue~
    }
    class FieldType {
        <<enumeration>>
        String
        Integer
        Decimal
        Boolean
        Color
        Enum
        List
        Map
        +check(raw) Result~FieldValue, Violation~
    }
    class Assignment {
        +field FieldRef
        +value RawValue
    }
    class Profile
    class ProfileRule {
        +os Option~Platform~
        +hostname Option~String~
        +tags BTreeSet~Tag~
        +enable Vec~OutputName~
        +matches(host, local) bool
    }
    class LocalSettings {
        +source Option~RawPath~
        +tags BTreeSet~Tag~
        +file Option~LoadedFile~
    }
    class HostContext {
        +platform Platform
        +hostname String
        +home PathBuf
        +env(name) Option~OsStr~
    }
    class RawValue
    class FieldValue

    EffectiveConfig *-- Manifest
    EffectiveConfig *-- ModuleSet
    EffectiveConfig *-- Profile
    EffectiveConfig *-- "0..*" Assignment : 層級 2
    Manifest *-- "0..*" OutputDefinition
    OutputDefinition *-- SourceSpec
    OutputDefinition *-- PlatformTargets
    SourceSpec --> TemplateRef
    ModuleSet *-- "0..*" ModuleDecl
    ModuleDecl *-- "0..*" FieldDecl
    FieldDecl *-- FieldType
    FieldType ..> FieldValue : 產生
    Assignment *-- RawValue
    FieldType ..> RawValue : 檢查
    Profile *-- "0..*" ProfileRule
    ProfileRule *-- "0..*" Assignment : 層級 3
    LocalSettings *-- "0..*" Assignment : 層級 4
    ProfileRule ..> HostContext : 比對
    ProfileRule ..> LocalSettings : 取得標籤
```

## 4. 詳細類別圖 D2：值的解析、範本與預檢

對應 §4.4–§4.6。`TemplateValues` 只由 `PreflightChecker.check_inputs` 建立，`CompiledTemplate.render` 只接受它，未通過檢查的值因此無法進入範本。`TargetSet`、`ApplyPlan`、`StateLoadResult` 與 `FileSystem` 在 §4.10–§4.12 定義（D5、D6），這裡只畫出依賴。

```mermaid
classDiagram
    direction TB
    class ValueResolver {
        +resolve(config, host, local, diags)$ ResolvedValues
    }
    class ResolvedValues {
        +get(field) Option~Resolved~
        +is_conflicted(field) bool
    }
    class Resolved {
        +value RawValue
        +source ValueSource
        +overridden Vec~ValueSource~
    }
    class ValueSource {
        +layer Layer
        +at SourceLocation
        +rule Option~usize~
    }
    class Layer {
        <<enumeration>>
        BuiltinDefault
        UserModule
        Profile
        Local
    }
    class TemplateEngine {
        +new() TemplateEngine
        +compile(text, diags) Checked~CompiledTemplate~
        +global_names()$ str[]
    }
    class SourceLoader {
        +load(output, diags) Checked~LoadedSource~
        +references(output, diags) Checked~References~
    }
    class LoadedSource {
        <<enumeration>>
        Template(CompiledTemplate)
        Verbatim(bytes, observation)
    }
    class CompiledTemplate {
        +meta() TemplateMeta
        +references() References
        +unknown_references(modules) Vec~UnknownReference~
        +render(values, diags) Checked~bytes~
    }
    class TemplateMeta {
        +program Option~String~
        +format_version Option~String~
        +requires Map~ModuleName, u32[]~
    }
    class References {
        +fields BTreeSet~FieldRef~
        +whole_modules BTreeSet~ModuleName~
        +mentions_module(m) bool
        +mentions_field(f) bool
    }
    class TemplateValues
    class AnalysisContext {
        +host HostContext
        +layout Layout
        +local LocalSettings
        +config EffectiveConfig
        +values ResolvedValues
        +enabled EnabledOutputs
        +sources Map~OutputName, LoadedSource~
        +state StateLoadResult
    }
    class PreflightChecker {
        +check_inputs(ctx, diags)$ Checked~TemplateValues~
        +check_targets(ctx, targets, diags)$ Checked
        +check_plan(ctx, plan, diags)$ Checked
    }
    class TargetSet
    class ApplyPlan
    class FileSystem {
        <<interface>>
    }

    ValueResolver ..> ResolvedValues : 產生
    ResolvedValues *-- "0..*" Resolved
    Resolved *-- ValueSource
    ValueSource --> Layer
    SourceLoader --> TemplateEngine
    SourceLoader ..> FileSystem : 讀取來源
    SourceLoader ..> LoadedSource : 產生
    LoadedSource --> CompiledTemplate
    CompiledTemplate *-- TemplateMeta
    CompiledTemplate *-- References
    CompiledTemplate ..> TemplateValues : 渲染時讀取
    PreflightChecker ..> AnalysisContext
    PreflightChecker ..> TargetSet
    PreflightChecker ..> ApplyPlan
    PreflightChecker ..> TemplateValues : 建立
    AnalysisContext --> ResolvedValues
    AnalysisContext --> LoadedSource
```

## 5. 詳細類別圖 D3：設定載入與管理

對應 §4.3、§4.7、§4.8。`WriteLock` 只能由取得鎖定的函式建立，`ManifestService` 的每個方法都要求它。`StateStore`、`WriteLock`、`SyncHook` 與 `FileSystem` 在 §4.10、§4.11 定義（D5）。

```mermaid
classDiagram
    direction LR
    class Layout {
        +source HomePath
        +config HomePath
        +state HomePath
        +config_dir(host, diags)$ Checked~HomePath~
        +new(host, source, diags)$ Checked~Layout~
        +reserved() Vec~Reserved~
    }
    class ConfigLoader {
        +new(fs, host) ConfigLoader
        +load_local(config_dir, diags) Checked~LocalSettings~
        +load_layout(diags) Checked~LocalSettings, Layout~
        +load(layout, diags) EffectiveConfig
    }
    class TomlEditor {
        +parse(text, origin, diags)$ Checked~TomlEditor~
        +get(path) Option~EditValue~
        +set(path, value)
        +remove(path) bool
        +remove_table(path) bool
        +to_text() String
    }
    class ManifestService {
        +add(lock, req, diags) Checked~OperationResult~
        +update(lock, name, patch, diags) Checked~OperationResult~
        +forget(lock, name, diags) Checked~OperationResult~
    }
    class AddRequest {
        +target PathBuf
        +name Option~OutputName~
    }
    class OutputPropertyPatch {
        +source PatchValue~SourceSpec~
        +target Vec~PlatformPatch~
        +ownership PatchValue~Ownership~
        +permissions PatchValue~Permissions~
        +takeover PatchValue~bool~
    }
    class PatchValue~T~ {
        <<enumeration>>
        Keep
        Set: T
        Reset
    }
    class OperationResult {
        +output OutputName
        +changes Vec~FileChange~
        +unchanged bool
    }
    class EffectiveConfig
    class FileSystem {
        <<interface>>
    }
    class SyncHook {
        <<interface>>
    }
    class StateStore
    class WriteLock

    ConfigLoader ..> Layout
    ConfigLoader ..> FileSystem : 讀取
    ConfigLoader ..> EffectiveConfig : 產生
    ManifestService ..> AddRequest
    ManifestService ..> OutputPropertyPatch
    OutputPropertyPatch *-- PatchValue
    ManifestService ..> OperationResult : 產生
    ManifestService ..> ConfigLoader : 載入 manifest
    ManifestService ..> TomlEditor : 修改文字
    ManifestService ..> WriteLock : 要求
    ManifestService ..> StateStore : 停止管理
    ManifestService ..> FileSystem : 建立、取代
    ManifestService ..> SyncHook
```

## 6. 詳細類別圖 D4：查詢

對應 §4.9。`QueryService` 只使用 `FileSystem` 的讀取與列出目錄，不需要 `WriteLock`。

```mermaid
classDiagram
    direction TB
    class QueryService {
        +list_outputs(diags) Vec~OutputListing~
        +find_outputs(query, diags) Checked~FindResult~
        +describe_output(key, diags) Checked~OutputDescription[]~
        +describe_value(field, diags) Checked~ResolvedValueDescription~
    }
    class RefQuery {
        <<enumeration>>
        Module: ModuleName
        Field: FieldRef
    }
    class OutputKey {
        <<enumeration>>
        Name: OutputName
        Path: PathBuf
    }
    class FindResult {
        +outputs Vec~OutputSummary~
        +undetermined Vec~OutputSummary~
    }
    class OutputSummary {
        +name OutputName
        +target TargetDisplay
        +enabled bool
    }
    class OutputListing {
        +kind ListingKind
        +source Option~SourceSpec~
        +enabled_by Vec~SourceLocation~
    }
    class OutputDescription {
        +source SourceSpec
        +references References
    }
    class ResolvedValueDescription {
        +field FieldRef
        +declared_at SourceLocation
        +value Option~RawValue~
        +source Option~ValueSource~
        +overridden Vec~ValueSource~
    }
    class EffectiveConfig
    class ResolvedValues
    class SourceLoader
    class FileSystem {
        <<interface>>
    }

    QueryService --> EffectiveConfig
    QueryService --> ResolvedValues
    QueryService --> SourceLoader : 取得引用
    QueryService ..> FileSystem : 只讀
    QueryService ..> RefQuery
    QueryService ..> OutputKey
    QueryService ..> FindResult
    QueryService ..> OutputListing
    QueryService ..> OutputDescription
    QueryService ..> ResolvedValueDescription
    FindResult *-- "0..*" OutputSummary
    OutputListing *-- OutputSummary
    OutputDescription *-- OutputSummary
```

## 7. 詳細類別圖 D5：檔案系統與狀態

對應 §4.10、§4.11。會改變檔案的方法都要求 `WriteLock`，`WriteLock` 只能由 `acquire` 建立。綁定的操作分成 `prepare`、`execute`、`confirm` 三次呼叫，同步點落在呼叫之間（E1）。

```mermaid
classDiagram
    direction TB
    class FileSystem {
        <<interface>>
        +inspect(at, content) Result~FileInspection~
        +list_dir(at) Result~DirEntry[]~
        +location_key(at) Result~LocationKey~
        +prepare_read(at) Result~Prepared~
        +read_file(at) Result~bytes, Observation~
        +ensure_private_dir(at) Result
        +lock(file) Result~LockGuard~
        +temp_name(at) HomePath
        +append_line(lock, file, line) Result
        +prepare(lock, op) Result~Prepared~
    }
    class Prepared {
        <<interface>>
        +execute() Result~Executed~
    }
    class Executed {
        <<interface>>
        +confirm() Result~OpOutcome~
    }
    class FsOp {
        <<enumeration>>
        CreateFile
        ReplaceFile
        DeleteFile
        SetMode
        CreateDir
        RemoveDir
        Capture
        Backup
        RestoreLink
        MoveDir
        RemoveTree
    }
    class OpOutcome {
        +result Observation
        +written Option~Version~
        +captured Option~CaptureInfo~
    }
    class Observation {
        +at HomePath
        +parents Vec~FileId~
        +entry EntryState
    }
    class EntryState {
        <<enumeration>>
        Missing
        Present
    }
    class FileId {
        +persistent() PersistentId
        +same_device(other) bool
    }
    class Version {
        +sha256 Sha256
        +mode Option~u32~
    }
    class LocationKey {
        +relation(other) Relation
    }
    class FsError {
        +kind FsErrorKind
        +at HomePath
        +applied Applied
    }
    class WriteLock {
        +acquire(fs, state_dir, diags)$ Checked~WriteLock~
    }
    class SyncHook {
        <<interface>>
        +reach(point, op, at)
    }
    class StateStore {
        +load() StateLoadResult
        +commit(lock, base, new, temp) Result~LoadedState~
    }
    class StateLoadResult {
        <<enumeration>>
        Loaded
        Corrupt
        Newer
        Unreadable
    }
    class LoadedState {
        +file StateFile
        +observation Observation
    }
    class StateFile {
        +records() Iterator
        +put(record)
        +remove(target) bool
        +to_bytes() bytes
    }
    class ManagedOutputRecord {
        +output OutputName
        +target HomePath
        +parent PersistentId
        +observed ComparisonMetadata
        +desired ComparisonMetadata
        +forget_pending bool
    }
    class ComparisonMetadata {
        +sha256 Sha256
        +mode Option~Permissions~
    }

    FileSystem ..> FsOp : 接收
    FileSystem ..> Prepared : 產生
    Prepared ..> Executed : 產生
    Executed ..> OpOutcome : 產生
    FileSystem ..> Observation : 產生
    FileSystem ..> LocationKey : 產生
    FileSystem ..> FsError : 失敗時
    FileSystem ..> WriteLock : 寫入時要求
    WriteLock ..> FileSystem : 建立時使用
    OpOutcome *-- Observation
    Observation *-- EntryState
    Observation *-- "0..*" FileId
    EntryState --> Version
    StateStore ..> FileSystem
    StateStore ..> SyncHook
    StateStore ..> StateLoadResult : 產生
    StateLoadResult --> LoadedState
    LoadedState *-- StateFile
    LoadedState *-- Observation
    StateFile *-- "0..*" ManagedOutputRecord
    ManagedOutputRecord *-- "2" ComparisonMetadata
```

## 8. 詳細類別圖 D6：規劃、執行與回復

對應 §4.12–§4.15，細化 03 的 C3 與 C4 的套用部分。`DesiredContent` 只由 `Planner` 建立，`ApprovedPlan` 只由 `ApplyService` 在預檢與模式檢查通過後建立；`Executor` 只接收 `ApprovedPlan`，只寫入 `DesiredContent` 的位元組。

```mermaid
classDiagram
    direction TB
    class ApplyService {
        +preview(req, diags) Checked~ApplyPlan~
        +apply(lock, req, diags) Checked~ApplyReport~
    }
    class ApplyRequest {
        +choices Choices
        +diff bool
        +started_at SystemTime
    }
    class Planner {
        +targets(config, enabled) TargetSet
        +plan(input, diags) Checked~ApplyPlan~
    }
    class TargetSet {
        +entries Vec~TargetEntry~
        +reserved Vec
    }
    class ApplyPlan {
        +items() PlanItem[]
        +drift_conflicts() Iterator
        +strict_blocked() bool
        +state() LoadedState
    }
    class ApprovedPlan
    class PlanItem {
        +output OutputName
        +target HomePath
        +origin ItemOrigin
        +action Action
        +disposition Disposition
        +drift bool
        +observation Observation
        +desired Option~DesiredContent~
        +record_before Option~ManagedOutputRecord~
        +record RecordChange
    }
    class Action {
        <<enumeration>>
        Create
        Update
        Delete
        Unchanged
    }
    class Disposition {
        <<enumeration>>
        Proceed
        Takeover
        Resolve(Choice)
        Retain(RetainReason)
    }
    class RecordChange {
        <<enumeration>>
        Keep
        Rename
        Remove
        Put
    }
    class DesiredContent {
        +bytes() bytes
        +metadata() ComparisonMetadata
    }
    class PreflightChecker
    class Executor {
        +execute(lock, plan, started_at, diags) ApplyReport
    }
    class RecoveryManager {
        +begin(fs, layout, sync, lock, started_at, state)$ Result~Self, FsError~
        +temp_name(at) Result~HomePath~
        +capture(expect, content) Result~SnapshotId~
        +record(done)
        +rollback(diags) RecoveryReport
        +finish(diags)
        +clean_stale(fs, host, lock, diags)$
        +report_stale(fs, host, diags)$
    }
    class ApplyReport {
        +batch Option~BatchId~
        +items Vec~ItemResult~
        +failure Option~ExecError~
        +recovery Option~RecoveryReport~
        +outcome() ApplyOutcome
    }
    class ItemResult {
        +target HomePath
        +action Action
        +status ItemStatus
        +backup Option~HomePath~
    }
    class RecoveryReport {
        +failures Vec~RecoveryFailure~
        +state StateRecovery
    }
    class StateStore
    class FileSystem {
        <<interface>>
    }

    ApplyService ..> ApplyRequest
    ApplyService ..> Planner
    ApplyService ..> PreflightChecker
    ApplyService ..> Executor
    ApplyService ..> StateStore : 載入
    ApplyService ..> ApprovedPlan : 建立
    Planner ..> TargetSet : 產生
    Planner ..> ApplyPlan : 產生
    Planner ..> DesiredContent : 建立
    Planner ..> FileSystem : 只讀
    PreflightChecker ..> TargetSet
    PreflightChecker ..> ApplyPlan
    ApprovedPlan *-- ApplyPlan
    ApplyPlan *-- "0..*" PlanItem
    PlanItem --> Action
    PlanItem --> Disposition
    PlanItem --> RecordChange
    PlanItem *-- "0..1" DesiredContent
    Executor ..> ApprovedPlan
    Executor ..> RecoveryManager
    Executor ..> StateStore : 提交
    Executor ..> FileSystem
    Executor ..> ApplyReport : 產生
    RecoveryManager ..> FileSystem
    RecoveryManager ..> RecoveryReport : 產生
    ApplyReport *-- "0..*" ItemResult
    ApplyReport *-- "0..1" RecoveryReport
```

## 9. 詳細類別圖 D7：初始化

對應 §4.16。`GitClient` 只把 repo clone 到 `InitializationService` 經由 `FileSystem` 建立的暫存目錄；第一次套用重用 `ApplyService`。

```mermaid
classDiagram
    direction LR
    class InitializationService {
        +initialize(lock, req, diags) Checked~InitializationResult~
    }
    class InitializationRequest {
        +repo Option~String~
        +source Option~PathBuf~
        +tags Vec~Tag~
        +values Vec~FieldRef, RawValue~
        +apply Option~ApplyRequest~
    }
    class InitializationResult {
        +source HomePath
        +source_created bool
        +changes Vec~FileChange~
        +apply Option~ApplyReport~
    }
    class GitClient {
        <<interface>>
        +clone_into(url, dest) Result
    }
    class ApplyService
    class ConfigLoader
    class TomlEditor
    class Layout
    class FileSystem {
        <<interface>>
    }
    class WriteLock

    InitializationService ..> InitializationRequest
    InitializationService ..> InitializationResult : 產生
    InitializationService ..> GitClient : clone
    InitializationService ..> FileSystem : 暫存目錄、移入、本機設定
    InitializationService ..> ConfigLoader : 讀取本機設定
    InitializationService ..> TomlEditor : 合併本機設定
    InitializationService ..> Layout
    InitializationService ..> ApplyService : 第一次套用
    InitializationService ..> WriteLock : 要求
```

## 10. 循序圖 E1：一個項目的執行

對應 §4.10、§4.13、§4.14 與 §6.6，以「更新」為例。同步點（§8.2）以 `SyncHook` 的呼叫表示；任一步失敗時停止，交給 `RecoveryManager.rollback`。

```mermaid
sequenceDiagram
    participant E as Executor
    participant R as RecoveryManager
    participant F as FileSystem
    participant H as SyncHook
    E->>R: temp_name(target)
    R->>F: append_line(journal)
    R-->>E: 暫存檔名稱
    E->>R: capture(observation, true)
    R->>F: prepare(Capture)
    F-->>R: Prepared（已核對父目錄與目標）
    R->>H: reach(BeforeCapture, Capture)
    R->>F: execute()
    F-->>R: 副本與實際讀到的 SHA-256
    R->>H: reach(AfterCapture, Capture)
    R->>R: SHA-256 與觀測的版本比較
    R->>F: confirm()
    R-->>E: SnapshotId
    E->>F: prepare(ReplaceFile)
    F-->>E: Prepared
    E->>H: reach(BeforeOperation, Replace)
    E->>F: execute()
    F-->>E: Executed（交換後已核對舊檔）
    E->>H: reach(AfterOperation, Replace)
    E->>F: confirm()
    F-->>E: OpOutcome（結果的觀測、寫入的版本）
    E->>E: 寫入的版本與 DesiredContent 比較
    E->>R: record(Replaced)
```
