# 詳細設計

## 1. 設計基準

### 1.1 基準文件

本文件依 [SRS](../02-srs/srs.md)、[初步設計](../03-preliminary-design/uml.md)與[設計分析](../04-design-analysis/design-analysis.md)撰寫。需求範圍、優先序與驗收條件以 SRS 為準（04 §1）。03 的類別與循序圖、04 的規則與驗證情境，在本文件中細化為介面、格式與演算法；與兩者不同的地方，會在相關章節說明原因。

### 1.2 第一版範圍

第一版包含全部 `essential` 需求，以及下表納入的 `conditional` 需求（SRS §2.6）。下表列出全部 33 條 `conditional` 與 2 條 `optional` 需求：納入 23 條，延後 10 條 `conditional` 與 2 條 `optional`。

| 需求 | 決定 | 理由 |
| --- | --- | --- |
| IF-003 | 納入 | 預覽本來就要顯示差異，採用 unified diff 的成本與自訂格式相同，還能直接交給 `patch` 驗收 |
| IF-004 | 納入 | 自動化程式與 AI 助理只能從輸出判斷結果（SRS §2.3）。計畫與回報在程式內已是結構化資料，序列化的成本低 |
| IF-005 | 納入 | 使用者與 AI 助理須自行撰寫四種設定檔。說明可由設定格式的型別宣告產生，不必另外維護 |
| IF-006 | 納入 | FR-MOD-003 已要求內建模組宣告每個欄位，列出這些宣告即可 |
| IF-009 | 納入 | 只需判斷標準輸出與標準錯誤是否連接終端機 |
| FR-MOD-004 | 納入 | 使用者模組沿用內建模組的欄位宣告格式與 `PreflightChecker` 的檢查，不需要另一套驗證 |
| FR-TPL-004 | 納入 | 使用者覆寫內建範本後，內建模組改版時才能在寫入前發現不相容（SRS §3.2.2 簡介） |
| FR-TPL-005 | 納入 | 與 FR-TPL-004 一起提供；預檢只多一項版本比較 |
| FR-TPL-007 | 納入 | 研究中，配色的衍生格式要靠外部產生器（[chezmoi §5.2](../01-research/chezmoi.md) 缺什麼第 2 點），這是模組共用配色的主要情境。以範本的自訂函式實作（§2.3） |
| FR-TPL-008 | 納入 | 同 FR-TPL-007 |
| FR-MAN-013 | 納入 | 程式會自行改寫的設定檔需要這種擁有程度（chezmoi §5.2 缺什麼第 5 點）；規劃時只多一種動作判定 |
| FR-MAN-015 | 納入 | 03 C1 已有權限屬性，04 的 A6、A14 驗證情境也以它為前提 |
| FR-DRF-008 | 納入 | 04 §3.2 建議納入。延後前須先釐清 FR-MOD-002 是否允許保留曾接受 drift 的輸出（04 §6） |
| FR-DRF-014 | 納入 | 非互動執行時，這是刪除已修改舊輸出的唯一方式；沿用既有的清除流程 |
| FR-QRY-002 | 納入 | 延後前須先在 SRS 明訂 FR-QRY-004 的驗收方式（04 §6）。§2.3 的範本靜態分析已能取得欄位層級的引用 |
| FR-QRY-005 | 納入 | 與預覽共用 `Planner`，只是不輸出內容差異 |
| FR-INI-003 | 納入 | 04 §2 已有未指定遠端時不執行首次套用的流程，指定遠端時沿用 |
| FR-INI-005 | 納入 | 初始化本來就要記錄來源目錄的位置，後續指令才不必再指定（04 §2） |
| PERF-001 | 納入 | 作為狀態檔與比較基準（§5）的設計目標，並在 06 量測 |
| USE-004 | 納入 | `PreflightChecker` 本來就在寫入前檢查整批計畫（04 §2），改為收集所有錯誤後再回報 |
| SEC-003 | 納入 | 接管備份本來就要限制存取（04 §3.3），保留原權限只多一步 |
| MNT-002 | 納入 | 第一版沒有舊版可比較。第一版建立回歸測試用的設定集，之後的 minor 版本依此檢查 |
| MNT-003 | 納入 | 與 MNT-002 相同，第一版建立結構化輸出的範例，作為之後比對的基準 |
| FR-MAN-022 | 延後 | 互動模式須在三平台處理終端機輸入。非互動的接管授權已能處理同樣的情況（FR-MAN-019） |
| FR-DRF-010 | 延後 | 同 FR-MAN-022；非互動時可指定覆寫或接受（FR-DRF-005、FR-DRF-006） |
| FR-DRF-011 | 延後 | 只在互動模式下適用 |
| FR-DRF-017 | 延後 | 同 FR-MAN-022；非互動時可指定刪除（FR-DRF-014） |
| FR-MAN-017 | 延後 | 依賴檢查要分平台取得字型與程式的安裝資訊。依賴不影響寫入與 exit code（FR-PLN-014），延後不改變其他需求的行為 |
| FR-PLN-011 | 延後 | 同 FR-MAN-017 |
| FR-PLN-012 | 延後 | 同 FR-MAN-017 |
| FR-PLN-013 | 延後 | 同 FR-MAN-017 |
| FR-PLN-014 | 延後 | 同 FR-MAN-017 |
| FR-MAN-027 | 延後 | 使用者可用 FR-QRY-003 查到範本或來源檔的路徑，再自行開啟 |
| FR-MAN-014（optional） | 延後 | 須逐一支援各種輸出格式的解析與局部合併 |
| FR-INI-007（optional） | 延後 | 屬於互動模式，理由同 FR-MAN-022 |

延後互動模式後，04 §6 的「非嚴格模式的互動」不再適用；03 §11 提到的 `DependencyChecker` 也不建立。

延後的需求依 SRS 附錄 A.3 的規定，要記錄在 A.3 與修訂紀錄，這部分另開 PR 修訂 SRS。USE-002 的驗收條件以「啟用互動」為前提，延後互動模式後如何滿足，見 §12。

## 2. 技術選擇

本節依 04 §4 的三項選擇，分別寫出比較、定案與驗證條件。§2.2、§2.3 的驗證結果來自一次性探查：2026-11-09 在 Linux（Arch、rustc 1.99.1）執行，只涵蓋下表列出的案例。這些案例會整理成可重跑的 pilot 測試，放在 `prototypes/pilot/`，並在三平台的 CI 上執行。

### 2.1 實作語言、工具鏈與發佈

實作語言為 Rust（SRS TBD-4）。Rust 可以產生不依賴 runtime 的單一執行檔（DC-004），也能直接呼叫各平台的檔案 API，用來實作 04 §3.3 要求的重檢與操作綁定。§2.2、§2.3 選用的 crate 也都是 Rust 生態系的成熟套件。

| 項目 | 定案 | 依據 |
| --- | --- | --- |
| 工具鏈 | stable Rust、edition 2024。第一版以 rustc 1.99 為基準；`Cargo.toml` 以 `rust-version` 標出最低版本，CI 固定使用同一版本 | 結果可重現；MNT-001 的回歸測試不受編譯器版本影響 |
| Linux | `x86_64-unknown-linux-musl`，靜態連結 | 不依賴發行版的 glibc 版本（DC-004） |
| Windows | `x86_64-pc-windows-msvc`，以 `-C target-feature=+crt-static` 靜態連結 C runtime | 不需要另外安裝 Visual C++ 可轉散發套件（DC-004） |
| macOS | `x86_64-apple-darwin` 與 `aarch64-apple-darwin` | 只依賴每台 macOS 都有的系統函式庫 |
| 發佈 | CI 在三平台的 matrix 中建置，每個平台、每種架構一個執行檔 | DC-004。正式支援的平台版本由 06 決定（PORT-001） |
| git | 呼叫系統上的 `git` 執行檔 | DC-004 允許依賴 git；SSH 沿用使用者既有的金鑰與設定（AS-4、IF-012） |

git 的另外兩個選項：git2 須連結 libgit2，SSH 與 HTTPS 再分別依賴 libssh2 與 OpenSSL，難以在三平台都做成單一執行檔；gix 是純 Rust 實作，但 SSH 傳輸同樣要呼叫外部的 `ssh`。兩者都沒有比直接呼叫 `git` 更少依賴，因此不採用。

檔案操作的平台 API，Unix 預定用 rustix（`openat`、`renameat`、`unlinkat` 等以目錄 handle 為基準的呼叫），Windows 預定用 windows-sys（以 handle 開檔、查詢最終路徑、以 handle 改名）。可行性由 pilot P1–P3 驗證，結果記在 §6、§7。

驗證條件（04 §4 第 3 列）：

- 單檔替換、符號連結接管與回復：pilot P1–P3 在三平台以一般使用者權限執行。
- 單一執行檔：pilot 的 CI 為三平台建置執行檔，以 `ldd`、`otool -L`、`dumpbin /dependents` 檢查動態依賴，並在只安裝作業系統與 git 的環境執行。

### 2.2 設定格式與保留格式的編輯

manifest、模組、Profile 與本機設定使用同一種格式。這四種檔案由使用者或 AI 助理直接撰寫，rosamta 的指令也會修改它們（SRS §3.1.2）。

| 候選 | UTF-8 與註解（IF-010） | 保留格式的編輯（FR-MAN-030） | 錯誤位置（FR-MAN-029） | 熟悉程度 |
| --- | --- | --- | --- | --- |
| TOML | 支援 | toml_edit 以語法樹保留註解、空白與順序，cargo 修改 `Cargo.toml` 也用它 | 回報行號、欄位與原始片段 | Rust、Python 等工具普遍使用；Dotter 也採用 |
| YAML | 支援 | Rust 沒有像 toml_edit 這樣廣泛使用的保留格式編輯器，常用的 serde_yaml 也已停止維護。須依來源位置局部修改，自行處理各種結構的新增與刪除 | 依解析器而定 | README 的範例使用 YAML；縮排與隱式型別容易出錯 |
| KDL | 支援 | kdl crate 能保留格式 | 有 | 使用者與 AI 助理較不熟悉 |
| JSONC | 註解是 JSON 的非標準擴充 | 有能保留格式的解析器 | 有 | JSON 普及，但手寫時須處理引號與逗號 |

定案：採用 TOML，以 toml_edit 做語法樹編輯，也就是 04 §4 的第一個方向。讀取時用同一個解析器把文件轉成設定型別（`toml_edit::de`），讀取與編輯對同一份檔案的解讀才會一致。可接受的語法是鎖定版本的 toml_edit 所支援的 TOML 1.1.0。升級 toml_edit 前，須以 MNT-002 的回歸設定集確認舊檔仍能讀取。README 以 `theme.yaml` 為例，之後要改成 TOML。檔名、表格結構與欄位寫法見 §5。

探查使用 toml_edit 0.25.19（TOML 1.1.0），結果如下：

| 案例 | 結果 |
| --- | --- |
| 含中文註解、行尾註解、空行、空格與 tab 縮排的 manifest，解析後原樣輸出 | 逐位元組相同 |
| 同一份檔案改用 CRLF 換行 | 輸出全部變成 LF，整份檔案都不同 |
| CRLF 檔案先轉成 LF 再編輯，完成後轉回 CRLF | 原樣輸出逐位元組相同；以下各項編輯只改動目標行 |
| 修改一個值 | 直接指定新值時，`=` 後的空白與行尾註解都會被重設；沿用原值的前後空白與行尾註解（decor）後，只有值改變 |
| 修改 inline table 中的一個鍵 | 只改動該行 |
| 新增屬性或新增輸出 | 只新增行；新增的鍵不會沿用同一表格的縮排 |
| 移除一個輸出 | 該表格連同緊接在上方的註解與空行一起移除，其他行不變 |
| 只修改本機設定的標籤 | 只有標籤所在的行改變，行尾註解保留 |
| 語法錯誤、屬性名稱拼錯、重複鍵、重複表格、型別錯誤 | 都能回報行號與欄位位置；拼錯的屬性會列出可用的屬性名稱 |
| 多行字串中的 CRLF | 解析後的值保留 CRLF；先轉成 LF 再解析，值也會變成 LF |

由結果得出的編輯規則：修改值時沿用原值的 decor；整份使用 CRLF 的檔案，先轉成 LF 編輯，再轉回 CRLF。移除輸出時，緊接在上方的註解視為該輸出宣告的一部分。混用換行的檔案如何處理、新增的鍵是否沿用同一表格的縮排，在 §5 決定。錯誤訊息由 rosamta 補上檔案路徑。

### 2.3 範本引擎

| 候選 | 靜態取得欄位引用（FR-QRY-002–004） | 錯誤訊息（USE-003） | 預覽限制（FR-PLN-005） | 相容性與維護（MNT-001） |
| --- | --- | --- | --- | --- |
| MiniJinja | `undeclared_variables` 不需要資料即可列出引用，包含未執行的分支 | 有範本名稱、行號、原因；未定義的欄位會指出名稱 | 從空環境開始，只註冊需要的函式；載入其他範本須經過 rosamta 提供的 loader | 鎖定版本；公開的語法、函式與測試由 rosamta 決定 |
| Handlebars | 沒有公開 API，須自行走訪語法樹 | strict mode 有行號、欄位名稱 | 預設沒有外部命令，helper 由程式註冊 | Dotter 採用；條件判斷較受限 |
| Tera | 沒有列出引用變數的公開 API | 有行號 | 內建函式包含讀取環境變數的 `get_env`，須排除 | 與 MiniJinja 同為 Jinja 風格 |
| 自訂語法 | 可直接設計 | 須自行實作 | 可完全控制 | 語法、錯誤訊息與相容性都要自行維護 |

Askama 等在編譯期處理範本的引擎無法載入使用者的範本，不列入比較。

定案：採用 MiniJinja，鎖定版本（第一版為 3.1.0）。設定如下：

- 從 `Environment::empty()` 開始，不使用 HTML 跳脫。公開的過濾器、測試與函式（包括 FR-TPL-007、FR-TPL-008 的色碼轉換）逐一註冊，每一個都有回歸測試（MNT-001）。不註冊任何會讀寫檔案、讀取環境變數或執行外部命令的函式（FR-PLN-005）。
- 未定義的值採 `Strict`：輸出、迭代、屬性存取或條件判斷遇到未定義的值都會失敗。選填欄位沒有值時，以 `none` 傳入範本，不視為未定義；範本以 `is not none` 判斷是否輸出該設定（DATA-004）；`Environment::empty()` 不含任何內建測試，`none` 測試須另外註冊。這樣「未定義」只代表引用了不存在的欄位。
- 保留範本結尾的換行（`keep_trailing_newline`）。MiniJinja 預設會去掉這個換行，輸出就會和範本作者寫的內容不同。
- 範本只能透過 rosamta 提供的 loader 載入其他範本，loader 負責來源目錄的邊界檢查（04 §3.1）。

欄位引用規則：渲染與查詢共用 `undeclared_variables(true)` 的結果。`theme.foreground` 這類路徑記為對該欄位的引用；只出現模組名稱時（例如 `theme[key]` 或 `theme['cursor']`），記為引用整個模組。這種保守的做法可能多列，但不會漏列（FR-QRY-004）。已註冊的函式名稱與主機資訊的變數不算欄位引用。預檢也用這份結果比對模組的欄位宣告，在渲染前找出拼錯的欄位，包括不會執行到的分支。

探查使用 MiniJinja 3.1.0，以 Handlebars 6.4.4 對照，結果如下：

| 案例 | 結果 |
| --- | --- |
| 含條件分支、迴圈、動態索引與自訂函式的範本，列出引用 | 列出 `theme.foreground`、`theme.opacity`、`font.family`、`shellrc.windows_shell` 等路徑，包含未執行的分支；動態索引只列出 `theme`；自訂函式名稱也列在其中，須排除 |
| 範本以 `include` 載入另一個範本 | 列出的引用不含被載入範本中的欄位 |
| 引用不存在的欄位（例如 `theme.forground`） | 錯誤類型為未定義，訊息含範本名稱、行號與 `theme.forground` |
| 引用不存在的模組或函式 | 訊息含範本名稱、行號與名稱 |
| 條件判斷中引用不存在的欄位 | `Strict` 下報錯 |
| 未閉合的 `{% if %}` | 回報開頭所在的行 |
| `{{ }}` 中的字串沒有結束 | 回報的行號是檔案結尾，不是字串開始的那一行 |
| 選填欄位以 `none` 傳入並以 `is not none` 判斷 | 註冊 `none` 測試後，不輸出該設定 |
| 以 `include` 載入來源目錄外的路徑 | 沒有設定 loader 時找不到範本；設定 loader 後，由 loader 回報錯誤 |
| `Environment::empty()` 中使用未註冊的內建函式、過濾器或測試 | 回報找不到該函式、過濾器或測試 |
| Handlebars：語法錯誤、引用不存在的欄位 | 都有行號，欄位錯誤含名稱；沒有列出引用的 API |

USE-003 要求的四項內容中，範本路徑、行號與原因或欄位名稱可由 MiniJinja 的錯誤取得；修正提示由 rosamta 依錯誤類型與欄位宣告產生，例如列出名稱相近的欄位。以下兩項留給 §4：

- 字串沒有結束時，錯誤指向檔案結尾，須補上字串開始的行號。
- 靜態比對找到的錯誤欄位沒有行號，須另外定位。

是否開放 `include` 也留給 §4 決定。開放時，靜態分析須自行展開被載入的範本。

## 7. 平台假設清單

本節列出設計依賴、而 Rust 標準函式庫不保證的作業系統行為。每項假設以 `ASM-NN` 編號，列出官方文件依據、文件保證的範圍，以及 pilot 結果（`prototypes/pilot/`）。§6 的演算法只依賴本節列出的行為；06 為每個 `ASM-NN` 安排測試情境（§13）。

pilot 結果分三種：

- 通過：官方文件有保證，pilot 在三平台的結果也相符。
- 有條件：pilot 在三平台都相符，但相關行為官方文件沒有寫明，或須由設計另外確保某個前提。條件寫在「保證範圍與條件」欄。
- 不可行：這個做法無法滿足需求，須改變設計或修訂 SRS（04 §3.3 末段）。

本版沒有「不可行」的項目。P4、P5 驗證的是選定套件的行為，不屬於平台假設，結果見 §2.2、§2.3；兩者在三平台都通過。

### 7.1 驗證環境

證據來自 pilot workflow 的 CI run 42634818704，P1–P5 與發佈檢查全部通過。runner 映像檔固定版本；正式支援的平台版本由 06 決定（PORT-001）。

| 環境 | 作業系統 | 測試目錄的檔案系統 | 執行身分 |
| --- | --- | --- | --- |
| ubuntu-24.04 | kernel 7.0.0-1017-azure | ext4 | uid 1001，非 root |
| macos-15 | macOS 15.8.2（24H232）、arm64 | APFS | uid 501，非 root |
| windows-2025 | Windows Server 2025 build 26100 | NTFS | 新建的標準使用者，未提權；開發人員模式開啟與關閉各跑一次 |
| 本機（補充） | Arch Linux、kernel 7.2.14 | btrfs | 一般使用者 |

Windows 的開發人員模式以登錄值 `AllowDevelopmentWithoutDevLicense` 切換，不需重新開機就生效（見 ASM-13 的結果）。runner 預設以已提權的管理員身分執行，所以 pilot 另建只屬於 Users 群組的帳號來跑測試。本機結果只作補充，不當作基準。

### 7.2 檔案替換

| 編號 | 假設 | 依據 | 保證範圍與條件 | pilot 結果 |
| --- | --- | --- | --- | --- |
| ASM-01 | Linux 與 macOS 上，同一目錄內的 `rename` 以一次操作取代目標；程序在任何時間點被終止，目標都是完整的舊檔或新檔 | POSIX `rename`：取代期間名稱為 new 的目錄項目持續可見，且指向原本的 new 或 old | 只涵蓋程序終止，不涵蓋斷電與系統當機。macOS 的 `fsync` 不要求磁碟清空快取（須用 `F_FULLFSYNC`），REL-001、REL-002 也只要求程序終止的情境 | 通過。P2 在 Linux、macOS 各終止 100 次，輸出 1 MiB 與狀態檔 4 KiB 都沒有損毀或消失 |
| ASM-02 | Windows 上，`MoveFileExW(MOVEFILE_REPLACE_EXISTING \| MOVEFILE_WRITE_THROUGH)` 以一次操作取代目標 | `MoveFileExW` 文件說明會取代既有檔案，`MOVEFILE_WRITE_THROUGH` 等到移動寫入磁碟才返回；文件沒有說明是否為原子操作 | 依 pilot 在本機 NTFS 的結果。取代後的檔案使用暫存檔的安全描述元，也就是目錄繼承的 ACL，不保留原檔的 ACL。`ReplaceFileW` 會保留原檔屬性，但程序被終止後目標可能消失，不採用 | 有條件。兩個 Windows job 各終止 100 次，沒有損毀或消失。同一 run 中，`ReplaceFileW` 在兩個 job 共有 7 次目標消失 |
| ASM-03 | Linux 與 macOS 上，以 `O_CREAT \| O_EXCL` 和模式 `0600` 建立的暫存檔，建立當下就不允許其他帳號存取 | POSIX `open`：新檔的權限位元取自 mode 參數，再由 umask 移除位元 | umask 只會移除權限，不會增加，所以權限不會比 `0600` 寬。Windows 的暫存檔沿用目錄繼承的 ACL，保護方式在 §6 定義，以 06 的 A13 驗證 | 通過。P2 在 Linux、macOS 留下的暫存檔與替換後的目標都是 `0600` |

### 7.3 不跟隨連結與操作綁定

| 編號 | 假設 | 依據 | 保證範圍與條件 | pilot 結果 |
| --- | --- | --- | --- | --- |
| ASM-04 | 檔案身分可由 handle 取得並比較：Linux 與 macOS 用 `st_dev` 加 `st_ino`，Windows 用 volume 序號加 128 位元的 file ID | POSIX `<sys/stat.h>`：`st_ino` 與 `st_dev` 合起來在系統內唯一識別一個檔案。`FILE_ID_INFO` 文件：file ID 在同一 volume 內識別檔案 | 檔案刪除後編號可能被重複使用，所以只在同一批次內、對仍存在或剛確認過的檔案比較。網路檔案系統不在驗證範圍 | 通過。P1、P3 都用身分比較偵測替換，所有成功的替換都被偵測到 |
| ASM-05 | Linux 與 macOS 上，從 home 以 `openat(O_NOFOLLOW \| O_DIRECTORY)` 逐層開啟目錄，遇到符號連結就失敗；之後以目錄 fd 為基準的 `openat`、`renameat`、`unlinkat`、`linkat` 和 `fstatat(AT_SYMLINK_NOFOLLOW)`，都作用在該 fd 對應的目錄 | POSIX `open`：`O_NOFOLLOW` 遇到符號連結以 `ELOOP` 失敗；`O_DIRECTORY` 遇到非目錄以 `ENOTDIR` 失敗；相對路徑以 fd 對應的目錄為基準 | home 本身的路徑不在此範圍，由 §6 的正規化規則處理 | 通過。預檢後替換父目錄或上層目錄的 24 個案例，在 Linux、macOS 都被拒絕；P3 讀取連結時以 `ELOOP` 失敗，不會讀到所指檔案 |
| ASM-06 | Linux 與 macOS 上，持有目錄 fd 不能阻止其他程序改名或搬移該目錄。設計因此在操作後重新走訪路徑；若路徑已不通往持有的目錄，就經同一個 fd 復原 | POSIX `rename`：目錄正被其他程序使用時，是否以 `EBUSY` 失敗由實作決定；Linux 與 macOS 不會因目錄已被開啟而拒絕改名 | DC-002 在 Unix 的保證範圍：寫入不會落在替代的目錄或連結指向的位置。但若外部程序把原目錄整個搬到 home 外，本批寫入會短暫出現在搬出去的目錄中，重檢後隨即復原 | 有條件。最後一次檢查後替換的 24 個案例，Linux、macOS 的建立、更新、刪除都已復原，快照都判定失效；其中搬出 home 的 6 個寫入案例，寫入曾短暫出現在 home 外。另一個程序持續替換父目錄時跑 300 輪，沒有違反任何檢查條件 |
| ASM-07 | Linux 與 macOS 沒有以 handle 刪除或改名檔案的操作，核對檔案身分到 `unlinkat`、`renameat` 之間仍有空窗 | POSIX `unlinkat`、`renameat` 只接受名稱 | 這個空窗內的替換無法事先阻止，只能靠操作後的重檢發現（ASM-06）。§8 在這個空窗設同步點，由 06 驗證 | 有條件。pilot 的同步點不在這個空窗內；持續替換的 300 輪沒有違反檢查條件 |
| ASM-08 | Windows 上，以具資料存取權（`FILE_LIST_DIRECTORY`）、不含 `FILE_SHARE_DELETE` 的 handle 持有目錄時，其他程序無法改名或刪除該目錄。在這些目錄內的寫入可以用路徑操作 | `CreateFileW` 文件：沒有 `FILE_SHARE_DELETE` 時，其他程序以 delete 存取權開啟會失敗，而 delete 存取權同時涵蓋刪除與改名 | 文件沒有說明哪些存取權會參與共用檢查。pilot 發現只有 `FILE_READ_ATTRIBUTES` 和 `SYNCHRONIZE` 的 handle 擋不住改名，所以須加上 `FILE_LIST_DIRECTORY` | 有條件。兩個 Windows job 中，最後一次檢查後的 48 次替換都被擋下（sharing violation），操作照計畫完成。持續替換時，外部程序的嘗試被擋下 225–242 次 |
| ASM-09 | Windows 上，以 `FILE_FLAG_OPEN_REPARSE_POINT` 開啟會取得 reparse point 本身，不跟隨；開啟目錄後以 `GetFinalPathNameByHandleW` 核對最終路徑 | `CreateFileW` 文件：指定此旗標時不做一般的 reparse point 處理，直接開啟 reparse point | 符號連結、junction 以及其他 reparse point 一律視為連結。路徑中的目錄是 reparse point 時拒絕操作；目標是 reparse point 時，只有 `IO_REPARSE_TAG_SYMLINK` 可以接管（§6） | 通過。預檢後換成 junction 的案例都被拒絕；P3 讀取連結時開到的是連結本身，判定類型不符，不會讀到所指檔案 |
| ASM-10 | Windows 上，以 handle 設定 `FileDispositionInfo` 刪除檔案，刪除的就是核對過身分的那個檔案 | `FILE_DISPOSITION_INFO` 文件：handle 關閉時刪除該檔案 | 須以 `DELETE` 存取權開啟 | 通過。P1 的刪除案例照計畫完成 |

### 7.4 符號連結的接管與回復

原規劃以 `symlink_metadata` 判斷連結、只備份連結本身，回復時重建連結。pilot 確認：Windows 標準使用者在未開啟開發人員模式時無法建立符號連結（ASM-13），DC-001 要求標準使用者也能套用，所以回復時重建連結的做法在這個條件下不可行。改用下列做法，整個過程都不建立新的符號連結：

1. 備份：核對目標仍是預檢所見的連結後，在備份目錄為連結本身建立 hard link（ASM-11）。目標路徑上仍是原本的連結。
2. 接管：以 ASM-01、ASM-02 的方式，一次把連結換成一般檔案（ASM-12）。目標路徑上不是原本的連結，就是完整的新檔案。備份目錄裡的名稱成為這個連結唯一的名稱。
3. 回復：核對目標仍是本批寫入的檔案後，把備份 rename 回目標路徑，取代本批寫入的檔案。

回復後目標路徑上是原本的連結：檔案身分、連結內容都與接管前相同，並不是新建的連結，因此不違反 DC-003。

| 編號 | 假設 | 依據 | 保證範圍與條件 | pilot 結果 |
| --- | --- | --- | --- | --- |
| ASM-11 | 可以為符號連結本身建立 hard link，不跟隨到所指檔案。Linux 與 macOS 用不帶 `AT_SYMLINK_FOLLOW` 的 `linkat`；Windows 先以 `FILE_FLAG_OPEN_REPARSE_POINT` 開啟連結，再以 `NtSetInformationFile(FileLinkInformation)` 建立，`RootDirectory` 指定持有的備份目錄 handle | POSIX `linkat`：未設 `AT_SYMLINK_FOLLOW` 且 path1 是符號連結時，為符號連結本身建立 hard link。`FILE_LINK_INFORMATION` 文件：`RootDirectory` 為新連結所在目錄的 handle，此時 `FileName` 只寫檔名；設定時不需要特定存取權 | hard link 不能跨檔案系統（POSIX `EXDEV`），所以備份目錄須與目標在同一個檔案系統。Windows 上路徑中的 mount point 也是 reparse point，已依 ASM-09 拒絕。Linux 與 macOS 的 home 內可能有掛載點，跨檔案系統時如何處理由 §6 決定。Linux 開啟 `fs.protected_hardlinks` 時，連結須屬於目前的使用者 | 有條件。兩個平台、兩種所指位置（home 內與 home 外）的備份，身分都與原連結相同。Linux 以 `/dev/shm` 測試跨檔案系統，以 `EXDEV` 失敗，原連結不變，備份目錄沒有留下檔案；macOS runner 沒有第二個檔案系統，略過 |
| ASM-12 | rename 作用在連結本身：目標位置是連結時，被取代的是連結，所指檔案不變；來源是連結時，移動的也是連結本身 | POSIX `rename`：old 是符號連結時改名的是連結本身；new 是符號連結時移除的是連結本身。Windows 的回復以 `FILE_FLAG_OPEN_REPARSE_POINT` 開啟備份，再以 `FileRenameInformation` 改名 | Windows 接管時用 `MoveFileExW` 取代目標位置上的連結，文件沒有寫明不跟隨連結，依 pilot 的結果 | 有條件。三平台接管後目標都是一般檔案，所指檔案的內容、大小與修改時間不變；回復後連結的身分與內容都和接管前相同 |
| ASM-13 | Linux 與 macOS 的一般使用者可以建立符號連結；Windows 的標準使用者須開啟開發人員模式，並在建立時指定 `SYMBOLIC_LINK_FLAG_ALLOW_UNPRIVILEGED_CREATE`，否則須具備 `SeCreateSymbolicLinkPrivilege`。設計不依賴建立符號連結 | `CreateSymbolicLinkW` 文件：未提權的程序要建立符號連結，須先開啟開發人員模式並指定此旗標。「Create symbolic links」使用者權限（`SeCreateSymbolicLinkPrivilege`）預設只給 Administrators 群組 | rosamta 只在讀取與搬移時處理連結，不建立連結（ASM-11、ASM-12）。Windows 不論開發人員模式是否開啟，接管與回復的方式都相同 | 通過。Windows 開發人員模式關閉時建立被拒（`ERROR_PRIVILEGE_NOT_HELD`，1314），開啟時成功。兩種情況下 P3 的接管與回復都成功；關閉時，測試用的連結由管理員預先建立，再由標準使用者移入 |

ASM-11、ASM-12 的其他 pilot 案例：備份前若目標已換成一般檔案，備份會被拒絕，備份目錄不留檔案。回復前若本批寫入的檔案已被外部程式取代，回復會被拒絕，外部檔案與備份都保留（04 §3.3）。

### 7.5 發佈

| 編號 | 假設 | 依據 | 保證範圍與條件 | pilot 結果 |
| --- | --- | --- | --- | --- |
| ASM-14 | §2.1 的發佈目標只依賴作業系統內建的函式庫：Linux 為 static-pie，不依賴任何共享函式庫；Windows 只依賴系統 DLL；macOS 只依賴 `libSystem` | Apple QA1118：Apple 不支援靜態連結的執行檔，只保證動態連結的系統函式庫維持相容，所以 macOS 執行檔連結 `libSystem`，這是每台 macOS 都有的系統函式庫 | Windows 依賴 `kernel32`、`advapi32`、`ntdll` 與 API set `api-ms-win-core-synch-l1-2-0`。DC-004 驗收要求的「在只安裝作業系統與 git 的乾淨環境執行」還沒做，交給 06 | 有條件。四個發佈目標都建置成功，並以 `ldd`、`otool -L`、`dumpbin /dependents` 檢查依賴；toml_edit、MiniJinja 是純 Rust，不增加動態依賴 |

### 7.6 文件出處

- POSIX.1-2024：[`rename`](https://pubs.opengroup.org/onlinepubs/9799919799/functions/rename.html)、[`link`／`linkat`](https://pubs.opengroup.org/onlinepubs/9799919799/functions/link.html)、[`open`／`openat`](https://pubs.opengroup.org/onlinepubs/9799919799/functions/open.html)、[`unlink`／`unlinkat`](https://pubs.opengroup.org/onlinepubs/9799919799/functions/unlink.html)、[`<sys/stat.h>`](https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_stat.h.html)
- macOS：[`fsync(2)`](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fsync.2.html)、[QA1118 Statically linked binaries on Mac OS X](https://developer.apple.com/library/archive/qa/qa1118/_index.html)
- Windows：[`MoveFileExW`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw)、[`CreateFileW`](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew)、[`GetFinalPathNameByHandleW`](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfinalpathnamebyhandlew)、[`FILE_ID_INFO`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_id_info)、[`FILE_DISPOSITION_INFO`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_disposition_info)、[`FILE_LINK_INFORMATION`](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_link_information)、[`FILE_RENAME_INFORMATION`](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_rename_information)、[`CreateSymbolicLinkW`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createsymboliclinkw)、[Create symbolic links 使用者權限](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/security-policy-settings/create-symbolic-links)
