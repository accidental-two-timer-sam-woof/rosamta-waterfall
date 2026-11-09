# 詳細設計 pilot

這個 Cargo 專案用來驗證[詳細設計](../../docs/05-detailed-design/detailed-design.md)中，必須實際執行才知道答案的項目。實作階段不沿用這裡的程式碼。

| 編號 | 驗證項目 | 測試 |
| --- | --- | --- |
| P1 | 重檢與實際操作綁定：從 home 逐層開目錄、不跟隨連結，透過持有的 handle 操作；另一個程序在預檢後或最後一次檢查後替換父目錄時，不會改到替代位置（DC-002、SEC-002，04 §5 A10、A16、A20） | `tests/p1_bound_ops.rs` |
| P2 | 單檔原子替換：強制終止程序 100 次，目標只會是完整的舊檔或新檔；未宣告權限時保留原權限（REL-001、REL-002） | `tests/p2_atomic_replace.rs` |
| P4 | toml_edit 保留格式的編輯與錯誤位置（05 §2.2） | `tests/p4_toml_edit.rs` |
| P5 | MiniJinja 的靜態引用與錯誤訊息（05 §2.3） | `tests/p5_minijinja.rs` |

## 執行

```sh
cargo test --locked -- --nocapture
```

每項結果會輸出一行 `PILOT-RESULT`。P2 可用環境變數 `PILOT_P2_KILLS`（預設 100）調整終止次數，用 `PILOT_SEED` 重現同一組終止時間點。

工具鏈版本固定在 `rust-toolchain.toml`。CI（`.github/workflows/pilot.yml`）在 Linux、macOS、Windows 以一般使用者身分執行測試；Windows 會另建標準使用者來執行（`ci/run-as-standard-user.ps1`）。CI 也會建置 05 §2.1 的各個發佈目標，檢查執行檔只依賴作業系統內建的函式庫。
