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

    bin --> platform
    bin --> core
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
    host -.-> diag
    path -.-> host
    fs -.-> path
    layout -.-> path
```

層與層的箭頭表示可以依賴任何較低的層，例如第 3 層的 `manage` 直接使用第 1 層的 `docedit`。所有 module 都使用 `diag`，第 0 層以外的這類依賴不逐一畫出。

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

對應 §4.4–§4.6。`TemplateValues` 只由 `PreflightChecker.check_inputs` 建立，`CompiledTemplate.render` 只接受它，未通過檢查的值因此無法進入範本。`TargetSet`、`ApplyPlan`、`StateLoadResult` 與 `FileSystem` 在 §4 後半定義，這裡只畫出依賴。

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

對應 §4.3、§4.7、§4.8。`WriteLock` 只能由取得鎖定的函式建立，`ManifestService` 的每個方法都要求它。`StateStore`、`WriteLock`、`SyncHook` 與 `FileSystem` 在 §4 後半定義。

```mermaid
classDiagram
    direction LR
    class Layout {
        +source HomePath
        +config HomePath
        +state HomePath
        +config_dir(host, diags)$ Checked~HomePath~
        +new(host, local, diags)$ Checked~Layout~
        +reserved() Vec~Reserved~
    }
    class ConfigLoader {
        +new(fs, host) ConfigLoader
        +load_local(config_dir, diags) Checked~LocalSettings~
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
