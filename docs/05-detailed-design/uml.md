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
