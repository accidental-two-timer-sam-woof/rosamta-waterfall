# 07 操作手冊

## 1. 建置

建置前請準備 `rustup` 和平台的連結工具。Windows 使用 MSVC 工具鏈，需要 Visual Studio Build Tools 的 C++ 建置工具與 Windows SDK。第一次建置會下載 `rust-toolchain.toml` 指定的 Rust 1.99.1 與鎖檔中的套件。

在儲存庫根目錄執行：

```sh
cargo build --locked
cargo run --locked -- --version
```

執行檔位於 `target/debug/rosamta`。Windows 的檔名是 `target\debug\rosamta.exe`。

## 2. 查看指令說明

```sh
cargo run --locked -- --help
cargo run --locked -- help query refs
cargo run --locked -- apply --help
cargo run --locked -- schema manifest --help
```

`cargo run` 的第一個 `--` 後面是傳給 `rosamta` 的參數。建置完成後，可以直接執行二進位檔並帶入同樣的參數。說明和版本資訊由執行檔直接輸出，執行時可以在空白 home 或離線環境使用。

| 說明入口 | 用途 |
| --- | --- |
| `init --help` | 來源目錄、儲存庫 URL、本機標籤與初次套用的參數 |
| `diff --help`、`status --help`、`apply --help` | 預覽、摘要與套用的參數，以及如何指定 drift 處理方式 |
| `query --help`、`list --help` | 查詢參照、輸出、欄位值或列出輸出 |
| `add --help`、`modify --help`、`forget --help` | 納入既有檔案、修改屬性及停止管理 |
| `schema --help` | 可查看說明的設定格式和內建模組類別 |

## 3. 讀取結果與錯誤

| Exit code | 此開發版的行為 |
| --- | --- |
| `0` | 已顯示說明或版本資訊，輸出到標準輸出 |
| `1` | 參數有誤，或指令尚未接上服務，診斷送到標準錯誤 |

`schema` 可在離線、尚未初始化的環境執行。用 `rosamta schema manifest` 查看鍵名、型別、預設值和範例。`rosamta schema builtin theme` 會列出配色模組的欄位。`schema module`、`schema profile`、`schema local` 分別列出其他三種設定格式。

來源目錄已有 `manifest.toml` 時，可以使用 `rosamta list` 查看已宣告的輸出，或用 `rosamta query output <名稱或路徑>` 查看來源與引用。`rosamta query refs theme.foreground` 也會搜尋未啟用的輸出。`rosamta query value theme.foreground` 列出欄位值與覆寫來源。這些指令接受 `--format json`。

`rosamta diff` 顯示每個目標的動作和文字差異，`rosamta status` 只列出有變更或需處理的項目。兩者都可加上 `--strict` 模擬嚴格模式。預覽和摘要不會寫入目標或狀態檔。

`rosamta apply` 套用目前的設定。若目標已被外部修改，預設保留該檔案。可用 `--accept '~/路徑'` 記下外部修改而不改寫來源，或用 `--overwrite '~/路徑'` 寫回設定產生的內容。`--strict` 在有 drift 時阻擋整批套用。一般檔案宣告 `takeover = true` 後，接管時會在狀態目錄的 `backups/` 保留原檔副本。

套用成功回傳 `0`，同時有完成與保留的項目回傳 `2`，失敗或全數保留回傳 `1`。如果診斷提到交易快照保留在 `txn/`，請先檢查診斷所列的路徑與備份。

尚未接上服務的指令會回傳 `cli.not-implemented`，例如 `init`。

需要檢查 JSON 輸出時，可以執行 `cargo run --locked -- apply --format json`。請預期 exit code 為 `1`。`rosamta` 的標準輸出是一份含 `"result": null` 的 JSON，標準錯誤另有一行 JSON 診斷。參數錯誤、說明和版本資訊使用文字輸出。使用 `help` 子指令時，`--format` 放在 `help` 前面。要分開擷取 `rosamta` 的兩個串流，請直接執行建置後的二進位檔，避開 Cargo 的建置訊息。
