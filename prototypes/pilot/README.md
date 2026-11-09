# 詳細設計 pilot

這個 Cargo 專案用來驗證[詳細設計](../../docs/05-detailed-design/detailed-design.md)中，必須實際執行才知道答案的項目。實作階段不沿用這裡的程式碼。

| 編號 | 驗證項目 | 測試 |
| --- | --- | --- |
| P2 | 單檔原子替換：強制終止程序 100 次，目標只會是完整的舊檔或新檔；未宣告權限時保留原權限（REL-001、REL-002） | `tests/p2_atomic_replace.rs` |

## 執行

```sh
cargo test --locked -- --nocapture
```

每項結果會輸出一行 `PILOT-RESULT`。P2 可用環境變數 `PILOT_P2_KILLS`（預設 100）調整終止次數，用 `PILOT_SEED` 重現同一組終止時間點。

工具鏈版本固定在 `rust-toolchain.toml`。CI（`.github/workflows/pilot.yml`）在 Linux、macOS、Windows 以一般使用者身分執行測試；Windows 會另建標準使用者來執行（`ci/run-as-standard-user.ps1`）。CI 也會建置 05 §2.1 的各個發佈目標，檢查執行檔只依賴作業系統內建的函式庫。
