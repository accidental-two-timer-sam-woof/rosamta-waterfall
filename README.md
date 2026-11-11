# rosamta

> 首版目前在設計與測試規劃階段。現有程式碼是用於驗證設計的 pilot。

`rosamta` 是一個跨平台、跨裝置的 dotfile 管理工具。除了同步與部署設定檔之外，它的核心想法是：**把多個程式共用的設定抽成「模組」，用一份設定同時產生多個 dotfile。**

例如配色、字型、shell 環境變數這類設定，往往分散在終端機、編輯器、tmux 等多個程式的設定檔中。透過模組，使用者只需維護一份設定，就能一次套用到所有相關程式。

## 核心概念

- **模組（Module）**：定義多個程式共用的設定欄位，例如配色、字型和 shell 環境。修改欄位值，就能更新引用它的輸出。
- **範本（Template）**：把模組的值轉換成特定程式的設定格式。範本可以由使用者自訂，也可以覆寫內建範本。
- **主機設定（Profile）**：依照作業系統、主機名稱或自訂標籤，決定要啟用哪些模組與範本，以及輸出的位置。

```
theme ──┬──> 終端機設定
        ├──> 編輯器配色
        └──> tmux 配色
```

## 內建模組（規劃中）

我們會針對常見類型的應用程式先實作好模組與範本：

| 模組 | 用途 |
| --- | --- |
| `theme` | 配色、透明度等外觀設定 |
| `font` | 字型、字級 |
| `shellrc` | 環境變數、alias、PATH |
| `terminal` | 終端機共通行為，如游標、捲動、快捷鍵 |

使用者也可以撰寫自己的模組與範本，管理內建模組未涵蓋的程式。

## 專案文件

1. 前期研究：[chezmoi](docs/01-research/chezmoi.md)、[Dotbot](docs/01-research/dotbot.md)、[Dotter](docs/01-research/dotter.md)
2. [軟體需求規格書](docs/02-srs/srs.md)
3. [初步設計與 UML](docs/03-preliminary-design/preliminary-design.md)
4. [設計分析](docs/04-design-analysis/design-analysis.md)
5. [詳細設計](docs/05-detailed-design/detailed-design.md)與 [pilot](prototypes/pilot/README.md)
6. [測試計畫](docs/06-test-plan/test-plan.md)

## 授權

[Apache-2.0](LICENSE)。
