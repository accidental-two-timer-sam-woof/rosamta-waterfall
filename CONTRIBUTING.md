# 貢獻指南

本專案採用瀑布式（waterfall）開發，各階段文件與程式碼都透過 Pull Request 進入 `main`。送出 PR 前請先讀完本文件。

## 流程概覽

1. 從最新的 `main` 開出分支
2. 在分支上完成工作，每個 commit 都是一個完成的任務
3. 推上分支並開 PR，依範本填寫
4. 經 review 修改後，整理好 commit 歷史
5. 以 rebase merge 合併進 `main`，合併後刪除分支

## 分支命名

```
<type>/<short-description>
```

- `type` 與 commit 的 type 相同，見下方 [Type](#type)
- 描述用英文、全小寫、以 `-` 連接，簡短說明這個分支要做的事
- 有對應 issue 時，可在描述前加上編號

```
feat/module-loader
fix/12-empty-key-crash
docs/srs-profile-matching
```

## Commit 規範

### 一個 commit 就是一個完成的任務

`main` 上的每個 commit 都應該能單獨拿出來看、單獨 checkout、單獨 revert。

- **不要 commit 做到一半的東西**。不要有 `wip`、`tmp`、`fix typo` 之類只是存檔用的 commit；這些請在合併前整理掉（見 [整理 commit 歷史](#整理-commit-歷史)）
- **一個 commit 只做一件事**。標題需要用 `and` 才講得完，通常代表應該拆成兩個 commit
- **程式碼 commit 要能通過編譯與測試**。語言與建置工具確定後，會在這裡補上實際要跑的指令
- **文件 commit 要是一段完整的修改**，例如補完一個章節、修正一組相關的需求，而不是寫到一半的段落

### 格式

採用 [Conventional Commits](https://www.conventionalcommits.org/zh-hant/v1.0.0/)，commit 訊息一律使用英文。

```
<type>(<scope>): <subject>

<body>
```

`(<scope>)` 可省略，見下方 [Scope](#scope)。

**標題（subject）**

- 50 字元以內為佳，最多不超過 72 字元
- 小寫開頭，結尾不加句號
- 只寫重點：讀標題就要知道這個 commit 做了什麼

**內文（body）**

- 選填。標題已經講清楚就不要寫，大多數 commit 應該只有標題
- 寫「為什麼」，不寫「做了什麼」——做了什麼看 diff 就知道
- 修改既有內容時（例如調整已寫好的需求或設計），在內文說明原因。修正錯字這類一看就懂的修改不在此限
- 與標題之間空一行，每行在 72 字元處換行

**範例**

不好的寫法：

```
update files
```

```
fix: fixed the bug where the template engine crashes when the module file has an empty key and also updated docs
```

好的寫法：

```
fix(template): handle empty keys in module files
```

```
docs(srs): add requirements for profile matching
```

```
refactor(module)!: rename `colors` field to `palette`

`colors` was ambiguous with terminal color settings in shellrc.yaml.
Existing module files must rename the field.
```

### Type

| type | 用途 |
| --- | --- |
| `feat` | 新功能 |
| `fix` | 修正錯誤 |
| `docs` | 文件（含各階段規格文件） |
| `refactor` | 不改變行為的程式重構 |
| `test` | 新增或修改測試 |
| `build` | 建置系統、相依套件 |
| `chore` | 其他雜務，例如設定檔、專案結構 |

破壞性變更（breaking change）在 type 或 scope 後加上 `!`，並在內文說明影響。

### Scope

scope 選填。修改階段文件時，可以用 scope 標出是哪一份文件，方便從 `git log` 追蹤每份文件的修改歷程：

| scope | 對應目錄 |
| --- | --- |
| `research` | `docs/01-research` |
| `srs` | `docs/02-srs` |
| `preliminary-design` | `docs/03-preliminary-design` |
| `design-analysis` | `docs/04-design-analysis` |
| `detailed-design` | `docs/05-detailed-design` |
| `test-plan` | `docs/06-test-plan` |
| `user-manual` | `docs/07-user-manual` |

程式碼的 scope 可用主要改動的 crate 或 module 名稱，例如 `cli`、`core`、`platform`。

## 實作期間

從儲存庫根目錄執行以下檢查：

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

實作若偏離 05，請在同一個 PR 更新[詳細設計](docs/05-detailed-design/detailed-design.md)及受影響的圖，記下改動與原因。新增或變更使用者可以執行的操作時，同步補寫[操作手冊](docs/07-user-manual/user-manual.md)中的指令與例子。

## Pull Request

### 開 PR 前

- 先 rebase 到最新的 `main`，不要把 `main` merge 進你的分支
- 確認每個 commit 都符合上面的規範
- 依 PR 範本填寫，不需要的區塊直接刪除

### PR 描述範例

```markdown
## 變更說明

- 實作模組檔的讀取，依詳細設計 3.2 節「模組載入」
- 加上必填欄位與型別的驗證
- 新增對應的單元測試

## 補充

- 設計中缺少欄位時會給預設值，實作改為直接報錯，避免使用者打錯欄位名稱卻沒發現。本 PR 已同步修改詳細設計
- 錯誤訊息目前只有英文，之後再補中文

## 類型

- [ ] 文件
- [x] 程式碼
- [ ] 其他（專案設定、協作規範等）

## 檢查清單

- [x] 已 rebase 到最新的 `main`
- [x] 每個 commit 都是一個完成的任務，沒有 wip、fixup 等未整理的 commit
- [x] commit 訊息符合 Conventional Commits，標題精簡
- [x] 程式碼 commit 皆可通過編譯與測試
- [x] 文件遵守盤古之白
```

### Review 與修改

review 期間的修改可以先用 fixup commit 推上去，方便 reviewer 只看新的變動：

```sh
git commit --fixup=<要修正的 commit>
```

### 整理 commit 歷史

本專案使用 **rebase merge**，分支上的每個 commit 都會原封不動進入 `main`。合併前請把 fixup 與做到一半的 commit 整理掉：

```sh
git fetch origin
git rebase -i --autosquash origin/main
git push --force-with-lease
```

推送時請用 `--force-with-lease`，不要用 `--force`，避免覆蓋掉別人推上來的東西。

## 文件撰寫規範

文件以繁體中文撰寫。

### 盤古之白

中文與英文、數字之間加一個半形空格，俗稱「盤古之白」。完整規則可參考[中文文案排版指北](https://github.com/sparanoid/chinese-copywriting-guidelines)，以下列出重點：

```
不好：使用rosamta管理3台主機的dotfile
好的：使用 rosamta 管理 3 台主機的 dotfile
```

- 中文與行內程式碼之間同樣要空格：`` 使用 `theme.yaml` 設定配色 ``
- 全形標點與其他字元之間不加空格：`設定檔（dotfile）`，不是 `設定檔 （dotfile）`
- 數字與單位之間加空格（`10 MB`），但 `%` 與 `°` 例外（`50%`）

### 標點

- 中文句子使用全形標點：，。、；：「」（）
- 句子主體是英文時（例如整句引用），使用半形標點

### 避免使用 emoji

文件與 commit 訊息中請避免使用 emoji。不是每種字型、終端機都能正確顯示，在 `git log` 或等寬字型的對齊中也常出現寬度錯亂。
