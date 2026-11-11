# rosamta

> 首版目前在設計與測試規劃階段。現有程式碼是用於驗證設計的 pilot。

`rosamta` 是跨平台的 dotfile 命令列工具。它把多個程式共用的設定抽成模組，用一份設定產生並套用多個 dotfile。

初始化時可從遠端 Git repo 取得來源目錄。後續由使用者透過 Git 等工具同步各主機的來源。

## 核心概念

- **模組（Module）**：定義多個程式共用的設定欄位，例如配色、字型和 shell 環境。修改欄位值，就能更新引用它的輸出。
- **範本（Template）**：把模組的值轉換成特定程式的設定格式。範本可以由使用者自訂，也可以覆寫內建範本。
- **主機設定（Profile）**：依作業系統、主機名稱或自訂標籤選擇要啟用的輸出，也可為符合條件的主機覆寫欄位值。

每個輸出由 manifest 宣告來源和各平台的目標路徑。來源可指定範本或要原樣複製的檔案。

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
