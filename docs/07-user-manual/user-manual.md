# 07 操作手冊

## 1. 建置

建置需要 rustup 與平台的連結工具。Windows 使用 MSVC 工具鏈，需要 Visual Studio Build Tools 的 C++ 建置工具與 Windows SDK。第一次建置會下載 `rust-toolchain.toml` 指定的 Rust 1.99.1 與鎖檔中的套件。

在儲存庫根目錄執行：

```sh
cargo build --locked --release
cargo run --locked --release -- --version
```

執行檔位於 `target/release/rosamta`，Windows 的檔名為 `target\release\rosamta.exe`。下列例子假設已將執行檔放入 PATH，也可以用 `cargo run --locked --` 取代開頭的 `rosamta`。

## 2. 第一次使用

```sh
rosamta init --source '~/dotfiles'
```

這會準備來源目錄與空的 `manifest.toml`，並在本機設定記錄自訂位置。省略 `--source` 時，預設來源為 `~/.local/share/rosamta`。本機設定預設在 `~/.config/rosamta/local.toml`，套用狀態預設在 `~/.local/state/rosamta/state.json`。可以用 `XDG_DATA_HOME`、`XDG_CONFIG_HOME`、`XDG_STATE_HOME` 調整預設位置，每個值須為 home 內的絕對目錄。

在來源目錄建立 `files/editor.conf`，寫入想部署的內容。在 `manifest.toml` 宣告目標：

```toml
[outputs.editor]
copy = "files/editor.conf"
target = "~/.config/my-editor/rosamta.conf"
```

在來源目錄的 `profile.toml` 啟用輸出：

```toml
[[rules]]
enable = ["editor"]
```

先執行 `rosamta diff` 確認路徑與差異，再用 `rosamta apply` 套用。原樣複製會保留來源檔的位元組。若已有設定檔而想從它開始，見第 5 節。

## 3. 使用遠端來源與主機設定

```sh
rosamta init https://github.com/your-account/dotfiles.git --source '~/dotfiles' --no-apply
rosamta diff
rosamta apply
```

遠端初始化支援 HTTPS 和 SSH。目標來源目錄須尚未存在，Git 必須可從 PATH 執行。指定遠端但省略 `--no-apply` 時會在初始化後直接執行第一次套用。Git 需要互動驗證時會失敗，請先完成 Git 或 SSH 的驗證設定。若錯誤列出暫存目錄，先確認其中資料再決定如何處理。

可以用 `--tag work` 加入本機標籤，用 `--set 'font.size=14'` 設定本機覆寫。兩個選項都可以重複使用。字串可寫成 `--set 'font.family=JetBrains Mono'`，若要保留字面值 `true` 則可寫 `--set 'prompt.style="true"'`。也能直接編輯 `local.toml`。

## 4. 模組、範本與查詢

設定的優先順序從低到高為內建預設、`modules/*.toml` 的 `[values.*]`、符合條件的 Profile 規則、本機設定。同一層內重複設定同一欄位會回報錯誤。

```toml
# modules/colors.toml
[values.theme]
foreground = "#c0caf5"
background = "#1a1b26"
```

範本使用 MiniJinja。在 `manifest.toml` 用 `template = "templates/editor.j2"` 取代 `copy`，範本可讀取 `{{ theme.foreground }}` 等值。內建輸出為 `ghostty`、`alacritty`、`posix-shell`、`powershell`，使用前同樣要在適用平台的 Profile 規則中啟用。shell 輸出是獨立設定檔，由使用者的 shell 啟動檔載入。

| 指令 | 用途 |
| --- | --- |
| `rosamta list` | 列出所有輸出、目標與啟用狀態 |
| `rosamta query output editor` | 查看指定輸出的來源與引用 |
| `rosamta query refs theme.foreground` | 搜尋引用此欄位的輸出，含未啟用輸出 |
| `rosamta query value theme.foreground` | 查看此主機的值及覆寫來源 |
| `rosamta schema builtin theme` | 列出內建配色欄位 |
| `rosamta schema manifest` | 查看 manifest 的格式與範例 |
| `rosamta schema module`、`profile`、`local` | 查看其他三種 TOML 格式 |

`help`、`--help`、`--version` 與 `schema` 可在離線且尚未初始化的環境使用。用 `rosamta help query refs` 查看特定子指令。

## 5. 納入、修改與停止管理

```sh
rosamta add '~/.gitconfig' --name gitconfig
rosamta modify gitconfig --ownership create-only
rosamta modify gitconfig --reset ownership
rosamta forget gitconfig
```

`add` 把一般檔案複製到來源目錄的 `files/`，並在 manifest 記錄目前平台的目標。接著在 Profile 的 `enable` 加入名稱，再執行 `diff`。目標已與來源相同時，`apply` 只登記狀態。停止管理會移除宣告和狀態紀錄，保留家目錄中的目標與來源副本。

## 6. 預覽、接管與 drift

`diff` 顯示動作與文字差異，`status` 只列出需要處理的項目。兩者都不寫入目標或狀態檔。對未受管理、內容不同的一般檔案，可在 manifest 宣告 `takeover = true`，原檔會保存在狀態目錄的 `backups/` 下。

受管理的檔案若有外部修改，預設保留。用 `--accept '~/路徑'` 可記下修改後的內容，用 `--overwrite '~/路徑'` 則重新套用來源內容。兩個選項都有 `-all` 形式，指定路徑的選項優先。先用 `diff` 查看選項效果。加上 `--strict` 後，只要有 drift，`apply` 就會阻擋整批寫入。停用或移除輸出時，原樣的舊目標會清除。已被修改的舊目標要用 `--delete-modified '~/路徑'` 或 `--delete-modified-all` 才能刪除。

## 7. 結果、診斷與回復

| Exit code | 行為 |
| --- | --- |
| `0` | 指令成功 |
| `1` | 參數或執行失敗，或套用項目全數保留 |
| `2` | 套用同時有完成與保留的項目。第一次套用也採用此規則 |

`--format json` 可放在子指令前後。標準輸出是一份 JSON 結果，診斷以 JSON Lines 寫到標準錯誤。參數錯誤、說明和版本資訊使用文字。使用 `help` 子指令時，`--format` 放在 `help` 前面。需要擷取串流時，可以直接執行二進位檔，避免混入 Cargo 的建置訊息。

套用途中失敗時，先看失敗項與回復結果。若有未回復的項目，核對列出的目標、`backups/` 和 `txn/` 內的資料，再重試。程序中止後留下的快照會保留並在套用時提示。
