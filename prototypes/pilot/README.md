# 詳細設計 pilot

這個 Cargo 專案用來驗證[詳細設計](../../docs/05-detailed-design/detailed-design.md)中，必須實際執行才知道答案的項目。實作階段不沿用這裡的程式碼。

乾淨環境的發佈驗收使用首版成品。

| 編號 | 驗證項目 | 測試 |
| --- | --- | --- |
| P1 | 重檢與實際操作綁定：從 home 逐層開目錄、不跟隨連結，透過持有的 handle 操作；另一個程序在預檢後或最後一次檢查後替換父目錄時，不會改到替代位置（DC-002、SEC-002，04 §5 A10、A16、A20） | `tests/p1_bound_ops.rs` |
| P2 | 單檔原子替換：強制終止程序 100 次，目標只會是完整的舊檔或新檔；未宣告權限時保留原權限（REL-001、REL-002） | `tests/p2_atomic_replace.rs` |
| P3 | 符號連結的接管與回復：在備份目錄為連結本身建立 hard link，再以一次 rename 換成一般檔案；回復時把備份 rename 回原位。所指檔案不變，過程中不建立新的符號連結；另記錄目前的使用者能否建立符號連結（SEC-002、DC-001、DC-003，04 §5 A3） | `tests/p3_symlink.rs` |
| P4 | toml_edit 保留格式的編輯與錯誤位置（05 §2.2） | `tests/p4_toml_edit.rs` |
| P5 | MiniJinja 的靜態引用與錯誤訊息（05 §2.3） | `tests/p5_minijinja.rs` |
| P6 | 05 §6 新增的平台操作：不覆蓋的建立與目錄改名；Linux、macOS 的交換式改名，以及據此做的取代與刪除會找出最後一次核對後的改動並放回原位；Windows 持有目標核對時，其他程序無法改名、刪除或寫入；Windows DACL 的建立與複製；多程序鎖定；以 `O_NONBLOCK` 開啟 FIFO（ASM-16–ASM-21） | `tests/p6_platform_ops.rs` |
| P7 | 初始化的 `git clone`：在建立並持有暫存目錄後，另一個程序把上層目錄換成指向 home 外的連結、換成 home 內的其他目錄，或整個搬到 home 外，再執行 clone。比較以路徑交給 git，與 Linux、macOS 以 `fchdir` 進入持有的目錄後 `git clone -- <url> .`、Windows 持有每一層 handle 的做法；clone 後重新走訪能否發現替換。另驗證撤回來源目錄時依賴的狀態變更時間：改寫內容並還原修改時間後是否仍會改變，以及能否被程式設回（DC-002，04 §5 A7） | `tests/p7_git_clone.rs` |

## 執行

```sh
cargo test --locked -- --nocapture
```

每項結果會輸出一行 `PILOT-RESULT`。P2 可用環境變數 `PILOT_P2_KILLS`（預設 100）調整終止次數，用 `PILOT_SEED` 重現同一組終止時間點。

工具鏈版本固定在 `rust-toolchain.toml`。CI（`.github/workflows/pilot.yml`）在 Linux、macOS、Windows 以一般使用者身分執行測試；Windows 會另建標準使用者來執行（`ci/run-as-standard-user.ps1`），並分別在開發人員模式開啟與關閉時各跑一次。關閉時標準使用者無法建立符號連結，P3 的 fixture 改從管理員預先建立的連結移入（`ci/make-link-pool.ps1`）。CI 也會建置 05 §2.1 的各個發佈目標，檢查執行檔只依賴作業系統內建的函式庫。
