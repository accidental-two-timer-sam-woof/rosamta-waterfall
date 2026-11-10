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

USE-003 要求的四項內容中，範本路徑、行號與原因或欄位名稱可由 MiniJinja 的錯誤取得；修正提示由 rosamta 依錯誤類型與欄位宣告產生，例如列出名稱相近的欄位。以下兩項的處理見 §4.5：

- 字串沒有結束時，錯誤指向檔案結尾，須補上字串開始的行號。
- 靜態比對找到的錯誤欄位沒有行號，須另外定位。

開放 `include` 時，靜態分析須自行展開被載入的範本；§4.5 決定第一版不開放。

## 3. 套件結構

本節把 03 C1–C4 的類別分配到 crate 與 module，並規定依賴方向。圖見 [uml.md](uml.md) 的 P1、P2；各類別的公開介面見 §4。

### 3.1 crate 劃分

程式以 Cargo workspace 分成四個 crate：

| crate | 種類 | 內容 | 依賴的 workspace crate | 外部 crate |
| --- | --- | --- | --- | --- |
| `rosamta` | 執行檔 | 命令列介面（§9）。啟動時收集主機資訊，建立 platform 的實作，交給 core 的服務 | core、platform | 參數解析與 diff 的套件在 §9 選定；serde_json |
| `rosamta-core` | 函式庫 | 03 的全部服務與資料型別，以及 `FileSystem`、`GitClient` 兩個 trait | 無 | toml_edit、minijinja、serde、serde_json、sha2 |
| `rosamta-platform` | 函式庫 | `FileSystem` 的 Unix 與 Windows 實作、`GitClient` 的實作（呼叫 `git` 執行檔），以及主機資訊的收集 | core | rustix（Unix）、windows-sys（Windows） |
| `rosamta-testkit` | 函式庫，只作 dev-dependency | 可注入故障的 `FileSystem`、在同步點執行注入的 `SyncHook`、建立測試用 home 的工具（§8） | core、platform | — |

外部 crate 一律以 `=` 鎖定版本。toml_edit、minijinja、rustix、windows-sys 沿用 pilot 驗證過的版本；其餘在實作開始時鎖定，升級時以 MNT-002 的回歸設定集檢查。

依賴方向是 `rosamta` → `rosamta-platform` → `rosamta-core`，執行檔也直接使用 core。trait 定義在 core，實作放在 platform，也就是依賴反轉：core 的服務只認得 trait，§8 與 06 可以把 platform 的實作換成 testkit 的實作，不必修改 core。

不採用「單一 crate 加 `#[cfg]`」，理由有二：

- core 的 `Cargo.toml` 不列 rustix 與 windows-sys，core 誤用平台 API 就無法編譯，不必靠 review 發現。
- core 在三平台編譯出同一份邏輯，平台差異集中在 platform 的 `unix`、`windows` 兩個 module，PORT-001 的測試重點因此明確。

core 不直接使用 `std::fs`、`std::process`、`std::net`、`std::env`。檔案操作經過 `FileSystem`，git 經過 `GitClient`；作業系統、主機名稱、home 與環境變數由執行檔在啟動時收集成 `HostContext`（C2）傳入，整次執行使用同一份（04 §2）。這項限制以 clippy 的 `disallowed-methods`、`disallowed-types` 設定在 core 檢查，CI 把警告當作錯誤。如此 core 沒有建立網路連線的途徑，唯一的對外連線是初始化時 `GitClient` 呼叫的 `git`，IF-013 只需檢查這一處。

讀取來源目錄也經過 `FileSystem`：04 §3.1 要求實際讀取的檔案位於來源目錄內，A9、A24 會在開檔前替換路徑，所以來源與目標使用同一套不跟隨連結的開檔方式（§6）。只依檔案系統而定的行為，例如目錄是否區分大小寫，也由 `FileSystem` 回答（§6），core 不依作業系統猜測。

### 3.2 core 的 module

core 的 module 分成四層。依賴只能指向較低的層，或下表「可依賴」欄列出的同層 module，不得形成循環。

| 層 | module | 03 的類別與型別 | 責任 | 可依賴 |
| --- | --- | --- | --- | --- |
| 0 基礎 | `diag` | `Diagnostics` | 診斷與來源位置（檔案、行、欄），所有 module 共用 | — |
| 0 | `host` | `HostContext`、`Platform` | 主機資訊與環境變數的快照 | `diag` |
| 0 | `path` | 目標路徑的型別 | `~` 與環境變數的展開、home 內的相對路徑、路徑比較（§6） | `diag`、`host` |
| 0 | `fs` | `FileSystem`、`FileInspection` | trait，以及它使用的檔案身分、類型、內容指紋與目錄 handle；鎖定 `WriteLock`；同步點與 `SyncHook` trait（§8.2） | `diag`、`path` |
| 0 | `layout` | `Layout` | 程式使用的位置（§5.1）與保留位置（§5.2） | `diag`、`host`、`path` |
| 1 資料 | `config` | `ConfigLoader`、`EffectiveConfig`、`Manifest`、`OutputDefinition`、`ModuleDecl`、`FieldDecl`、`Assignment`、`Profile`、`ProfileRule`、`LocalSettings`、`TemplateRef` | 讀取四種設定檔與內建資源，合併內建輸出與覆寫（§5.3–§5.9） | 第 0 層 |
| 1 | `docedit` | `TomlEditor` | 保留格式的 TOML 編輯（§5.3） | `diag` |
| 1 | `resolve` | `ValueResolver`、`ResolvedValues`、`ValueSource` | 四層覆寫與值的來源 | 第 0 層、`config` |
| 1 | `template` | `TemplateEngine`、`SourceLoader`、`TemplateMeta`、`References`、`TemplateValues` | MiniJinja 環境、範本 loader、靜態引用、原樣複製來源的讀取 | 第 0 層、`config` |
| 1 | `state` | `StateStore`、`StateFile`、`ManagedOutputRecord`、`ComparisonMetadata`、`StateLoadResult` | 狀態檔的格式、載入與原子提交（§5.10） | 第 0 層 |
| 2 流程 | `plan` | `Planner`、`ApplyPlan`、`PlanItem` | 計畫與比較基準的判定（§5.11） | 第 0 層、`config`、`resolve`、`template`、`state` |
| 2 | `preflight` | `PreflightChecker`、`AnalysisContext` | 整批預檢 | 第 0 層、`config`、`resolve`、`template`、`state`、`plan` |
| 2 | `recovery` | `RecoveryManager`、`RecoveryReport` | 回復快照的保存與還原 | 第 0 層、`state` |
| 2 | `exec` | `Executor`、`ApplyReport`、`ItemResult` | 執行計畫、製作接管備份（§5.12） | 第 0 層、`state`、`plan`、`recovery` |
| 3 子指令 | `apply` | `ApplyService`、`ApplyRequest` | 預覽、摘要與套用的流程 | 第 0 層、`config`、`resolve`、`template`、`state`、第 2 層 |
| 3 | `manage` | `ManifestService`、`AddRequest`、`OutputPropertyPatch`、`OperationResult` | 納入、修改屬性、停止管理（§5.13） | 第 0 層、`config`、`docedit`、`state` |
| 3 | `query` | `QueryService`、`OutputSummary`、`OutputListing`、`OutputDescription`、`ResolvedValueDescription` | 引用、輸出與欄位值的查詢 | 第 0 層、`config`、`resolve`、`template` |
| 3 | `init` | `InitializationService`、`GitClient`、`InitializationRequest`、`InitializationResult` | 初始化 | 第 0 層、`config`、`docedit`、`apply` |

與 03 C4 不同的依賴：

- `ConfigLoader` 經由 `FileSystem` 讀取來源目錄，理由見 §3.1。C4 只畫出 `Planner`、`Executor` 等對 `FileSystem` 的依賴。
- `PreflightChecker` 使用 `template` 的靜態引用，在渲染前找出拼錯的欄位（§2.3）。
- `QueryService` 使用 `template` 的靜態引用回答引用查詢。04 §2 已要求查詢共用範本的引用資訊，C4 沒有畫出。
- `ManifestService` 使用 `config` 取得內建輸出的名稱，使用 `docedit` 修改設定檔。C4 只畫出 `StateStore` 與 `FileSystem`。
- 摘要（FR-QRY-005）是 `ApplyService` 的一種模式，不屬於 `QueryService`，因為它與預覽共用 `Planner`（§1.2）。

依賴方向由 module 的 `use` 決定，編譯器不會檢查層級。§11 逐一核對每個 module 的 `use`，CI 另以腳本檢查 core 各 module 引用的 `crate::` 路徑都在上表的範圍內。

### 3.3 platform 與執行檔的 module

| crate | module | 責任 |
| --- | --- | --- |
| `rosamta-platform` | `unix` | `FileSystem` 的 Linux 與 macOS 實作，以 rustix 呼叫 `openat`、`renameat`、`unlinkat`、`linkat` 等以目錄 fd 為基準的操作（§7.3） |
| `rosamta-platform` | `windows` | `FileSystem` 的 Windows 實作，以 windows-sys 呼叫 `CreateFileW`、`SetFileInformationByHandle`、`MoveFileExW` 等操作（§7.3） |
| `rosamta-platform` | `host` | 收集 `HostContext`：作業系統、主機名稱（IF-011）、home 與環境變數 |
| `rosamta-platform` | `git` | `GitClient` 的實作，以 `std::process` 執行系統上的 `git`（§2.1） |
| `rosamta` | `cli` 等 | 子指令、參數與輸出格式（§9） |

`unix` 與 `windows` 以 `#[cfg]` 選擇，對外只提供同一個 `FileSystem` 實作的建構函式。

## 4. 類別詳細規格

本節把 03 C1–C4 的類別細化成 Rust 的公開介面。每個類別寫出責任、介面、前置與後置條件、錯誤，以及它對其他 module 隱藏的設計決定：這些決定改變時，只需要修改該 module。§4.1–§4.9 是設定、範本、管理與查詢；§4.10–§4.16 是檔案系統、狀態、規劃、回復、執行、套用與初始化。詳細類別圖見 [uml.md](uml.md) 的 D1–D7。

介面以 Rust 簽名表示，省略 derive、私有欄位與 `pub(crate)` 的項目。生命週期 `'a` 表示服務借用呼叫端建立的依賴（§8.1）。實作時可以調整名稱，但型別表達的限制必須保留，例如「先取得鎖定才能修改設定檔」。

與 03 不同的地方：

| 03 | 本節 | 理由 |
| --- | --- | --- |
| C1 `Field` 同時有宣告與值 | 分成欄位宣告 `FieldDecl` 與設定值 `Assignment` | 值來自四個層級的不同檔案（§5.5–§5.8），不屬於模組的宣告 |
| C1 `sourceKind` 與 `sourcePath` | 一個 enum `SourceSpec` | 範本與原樣複製只能選一種，以型別排除兩者都有或都沒有的情況 |
| C1 `Template` | 來源 `TemplateRef` 與中繼資料 `TemplateMeta` | 中繼資料寫在範本裡（§5.9），讀到範本才知道 |
| C2 `ResolvedValues.values` 為 `FieldValue` | 存未檢查的 `RawValue`，檢查後才轉成 `TemplateValues` | 04 §2 把型別檢查交給 `PreflightChecker`；以型別保證未通過檢查的值不會進入範本 |
| C4 `ConfigLoader.load()` | `load_local` 與 `load` | 來源目錄的位置記在本機設定（§5.1），必須先讀本機設定 |
| C4 `PreflightChecker.validate(context, optionalPlan)` | `check_inputs`、`check_targets`、`check_plan` 三個階段 | 三個階段需要的資料不同。USE-004 要求欄位值有錯時仍找出目標碰撞，碰撞的檢查因此不能等到渲染之後 |
| C4 `QueryService` | 加上 `list_outputs` | FR-MAN-026 的列出輸出是唯讀的，與其他查詢共用 Profile 判定 |
| C4 管理方法的參數 | 加上 `&WriteLock`；`add` 改收 `AddRequest` | 以型別確保修改前已取得鎖定（§6.7）；納入時可以指定輸出名稱 |
| C4 回傳 `Diagnostics` | 以 `&mut Diagnostics` 收集（§4.1） | 錯誤與警告走同一條路，USE-004 才能一次列出全部 |
| C3 `PlanItem` 的字串成員 | enum，加上觀測、期望內容與紀錄的變更（§4.12） | 執行時依觀測核對，以期望內容寫入；03 C3 末段要求補齊這個傳遞契約 |
| C3 `ApplyReport.exitCode` | `outcome()`，exit code 由 §9 對應 | core 不決定 exit code，與 §4.1 一致 |
| C4 `Planner.build(config, values, state)` | `targets` 與 `plan` | 目標碰撞須在渲染前檢查（§4.6） |
| C4 `FileSystem` 的 `inspect`、`backup`、`applyAtomically` | `inspect` 與三次呼叫的綁定操作 `prepare`、`execute`、`confirm` | 同步點須落在核對與系統呼叫之間（§8.2）；備份與快照也是綁定的操作 |
| C4 `RecoveryManager` 的 `begin()`、`capture(path)`、`commit()` | `begin` 接收狀態的基準，`capture` 接收觀測，`commit` 改名為 `finish`；加上 `record`、`temp_name`、`clean_stale` | 快照要依觀測核對（A16）；回復要知道本批做過的每個操作 |
| C4 `StateStore.saveAtomically(state)` | `commit(lock, base, new, temp)` | 被取代的必須是載入的版本（A23） |
| C4 `ApplyService.run(request)`、`Executor.execute(plan)` | `preview` 與 `apply`；`execute` 只接收 `ApprovedPlan` | 預覽沒有鎖定，也就無法寫入；以型別確保只執行通過檢查的計畫 |
| C4 `GitClient.cloneToStaging(url)` | `clone_into(url, dest)` | 暫存目錄由 `FileSystem` 建立，才能核對身分（§6.3） |

### 4.1 共通約定

**診斷**（`diag`）：使用者輸入的問題與 I/O 失敗都以診斷回報，不用 Rust 的 panic。panic 只表示程式本身的錯誤，例如違反內部的不變量。

```rust
pub enum Severity { Error, Warning }

pub struct SourceLocation {
    pub origin: Origin,
    pub line: Option<u32>,   // 從 1 起算；對應原檔的行（§5.3）
    pub column: Option<u32>, // 從 1 起算，以 Unicode 字元計
}

pub enum Origin {
    Source(SourceRelPath),   // 來源目錄中的檔案
    Home(HomePath),          // home 內的其他檔案，例如本機設定
    Builtin(&'static str),   // 內建資源，例如 "builtin:ghostty"
    Environment(String),     // 環境變數的名稱
    Argument(String),        // 命令列參數
}

pub struct Diagnostic {
    pub severity: Severity,
    pub code: DiagCode,
    pub message: String,
    pub primary: Option<SourceLocation>,
    pub related: Vec<(SourceLocation, String)>, // 其他相關位置與說明，例如重複的另一處
    pub subjects: Vec<Subject>,                 // 涉及的輸出、模組、欄位或路徑
    pub hint: Option<String>,                   // 修正方式的提示（USE-003）
}

pub struct Diagnostics { /* 依加入順序保存 */ }

impl Diagnostics {
    pub fn error(&mut self, d: Diagnostic) -> Reported;
    pub fn warning(&mut self, d: Diagnostic);
    pub fn has_errors(&self) -> bool;
    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic>;
}

/// 只能由 `Diagnostics::error` 建立，表示錯誤已經記錄。
pub struct Reported(());
pub type Checked<T> = Result<T, Reported>;
```

- 每個可能遇到使用者輸入問題的函式都接收 `&mut Diagnostics`。回傳 `Checked<T>` 的函式失敗時，至少已記錄一個錯誤；呼叫端用 `?` 往上傳，不必再回報一次。能在錯誤之後繼續檢查的函式，例如 `ConfigLoader::load`，直接回傳結果，錯誤留在 `Diagnostics` 中。
- `DiagCode` 是 enum，每個值有固定的字串，例如 `config.unknown-key`，供 IF-004 的結構化輸出使用，同一個 major 版本內不改名（MNT-003）。各類別可能產生的代碼列在該類別的「錯誤」中。
- `message` 的語言與輸出格式由 §9 決定。core 不組合絕對路徑：`Origin` 保存 home 內的相對位置，由執行檔依 `HostContext` 轉成使用者看到的路徑。
- 除了套用，子指令的 exit code 只看 `has_errors()`：有錯誤為 `1`，否則為 `0`（IF-007）。套用的 exit code 見 §9。

**名稱**：以 newtype 表示，建構時檢查 §5.3 的格式，之後不必再檢查。

```rust
pub struct OutputName(String); // [A-Za-z0-9][A-Za-z0-9_-]*
pub struct ModuleName(String); // [A-Za-z_][A-Za-z0-9_]*
pub struct FieldName(String);  // 同 ModuleName
pub struct Tag(String);        // [A-Za-z0-9_-]+
pub struct FieldRef { pub module: ModuleName, pub field: FieldName } // 顯示為 theme.foreground

impl std::str::FromStr for OutputName { type Err = NameError; /* … */ }
// 其餘名稱型別同樣實作 FromStr；FieldRef 以第一個 "." 分開模組與欄位

pub struct Located<T> { pub value: T, pub at: SourceLocation }
```

**路徑**（`path`）：§6.1 的展開與正規化，以三種型別區分處理的階段。

```rust
/// manifest 的 target、本機設定的 source 與 XDG_* 變數的原文，已檢查語法（§5.4）。
pub struct RawPath { /* 文字片段、~、${NAME} 的序列 */ }
impl RawPath {
    pub fn parse(text: &str) -> Result<RawPath, PathSyntaxError>;
    pub fn expand(&self, host: &HostContext) -> Result<HomePath, PathError>; // §6.1 第 1–5 步
}

/// home 內正規化後的相對路徑，不含 home 本身；顯示為 ~/a/b。
pub struct HomePath { /* 元件的序列，至少一個 */ }

/// 來源目錄內的相對路徑：manifest 的 template 與 copy。
pub struct SourceRelPath { /* 元件的序列，至少一個 */ }
impl SourceRelPath {
    pub fn parse(text: &str) -> Result<SourceRelPath, PathSyntaxError>;
}

impl HomePath {
    /// 命令列給的絕對路徑，例如納入管理的檔案（§6.1 第 3–5 步）。
    pub fn from_absolute(path: &Path, host: &HostContext) -> Result<HomePath, PathError>;
}
```

`SourceRelPath::parse` 只接受以 `/` 分隔的相對路徑，不得含 `\`；去掉 `.` 與空元件，`..` 移除前一個元件，超出來源目錄時失敗（FR-MAN-005、04 §3.1）。元件的名稱限制與 §6.1 第 5 步相同，三個平台都檢查，來源目錄才能在三個平台使用。這是純文字的處理，實際讀取時路徑中的連結由 `FileSystem` 拒絕（§6.1）。

**主機資訊**（`host`）：

```rust
pub enum Platform { Linux, Macos, Windows }

pub struct HostContext {
    pub platform: Platform,
    pub hostname: String,          // IF-011
    pub home: std::path::PathBuf,  // home 的原值（§6.1）
    env: Env,
}
impl HostContext {
    pub fn new(platform: Platform, hostname: String, home: PathBuf,
               env: impl IntoIterator<Item = (OsString, OsString)>) -> Self;
    /// Windows 上不分大小寫比對名稱，與作業系統相同。
    pub fn env(&self, name: &str) -> Option<&OsStr>;
}
```

**本節用到的後半介面**：`FileSystem` 的讀取、列出目錄、建立、取代與刪除，`Observation`（§6.3 的觀測），`WriteLock`（§6.7 的鎖定），`SyncHook`（§8.2），以及 `StateStore`、`StateLoadResult`、`TargetSet`、`ApplyPlan`，都在 §4.10–§4.12 定義。本節只依賴下列行為：

- 讀取：以 home 或來源目錄為起點，依 §6.1 走訪，只開啟一般檔案，回傳內容與觀測。路徑中有連結、檔案是連結或其他類型時失敗，並說明類型。
- 列出目錄：回傳每個項目的名稱與類型，不跟隨連結。
- 建立、取代、刪除：依 §6.3 綁定；取代時要求被取代的檔案仍是指定的觀測版本。
- `WriteLock` 只能由取得鎖定的函式建立，持有到程序結束。

### 4.2 設定資料型別

C1、C2 的資料型別，放在 `config` module。所有型別都是讀取後不再修改的值；修改設定檔經由 `TomlEditor`（§4.7），寫入後重新載入。

```rust
pub struct EffectiveConfig {
    pub manifest: Manifest,
    pub modules: ModuleSet,
    pub profile: Profile,
    pub assignments: Vec<Assignment>,  // 覆寫層級 2：modules/ 中的 [values.*]
    pub manifest_file: Option<LoadedFile>,
    broken: BTreeSet<Origin>,          // 無法完整讀取的檔案
}
impl EffectiveConfig {
    pub fn is_complete(&self) -> bool;
    /// 包含因錯誤而未進入 manifest 的輸出，用來避免連帶的「沒有這個輸出」警告。
    pub fn is_declared(&self, name: &OutputName) -> bool;
}

pub struct LoadedFile { pub origin: Origin, pub text: String, pub observation: Observation }

pub struct Manifest {
    outputs: BTreeMap<OutputName, OutputDefinition>,      // 合併內建宣告與覆寫後的有效輸出
    removed_builtins: BTreeMap<OutputName, SourceLocation>, // removed = true 的內建輸出（§5.13）
}
impl Manifest {
    pub fn get(&self, name: &OutputName) -> Option<&OutputDefinition>;
    pub fn outputs(&self) -> impl Iterator<Item = &OutputDefinition>; // 依名稱排序
    pub fn removed_builtins(&self) -> impl Iterator<Item = (&OutputName, &SourceLocation)>;
}

pub struct OutputDefinition {
    pub name: OutputName,
    pub kind: OutputKind,                    // Builtin、BuiltinOverridden、User
    pub declared_at: SourceLocation,         // [outputs.x] 或 [builtin.x] 的表格標頭；未覆寫的內建輸出為 Builtin
    pub source: Located<SourceSpec>,
    pub targets: PlatformTargets,
    pub ownership: Located<Ownership>,
    pub permissions: Option<Located<Permissions>>,
    pub takeover: Located<bool>,
}

pub enum SourceSpec { Template(TemplateRef), Verbatim(SourceRelPath) }
pub enum TemplateRef { Builtin(BuiltinTemplate), User(SourceRelPath) }
pub enum Ownership { Full, CreateOnly }      // FR-MAN-012、FR-MAN-013
pub struct Permissions(u16);                 // 0..=0o777（§5.4）

pub struct PlatformTargets { linux: Option<Located<RawPath>>,
                             macos: Option<Located<RawPath>>,
                             windows: Option<Located<RawPath>> }
impl PlatformTargets {
    pub fn get(&self, platform: Platform) -> Option<&Located<RawPath>>;
}
```

每個屬性都帶有位置。覆寫過的內建輸出，屬性的位置指向 `[builtin.*]` 中寫下它的那一行；沿用內建宣告的屬性，位置是 `Origin::Builtin`。FR-PLN-009 回報碰撞時，使用 `target` 的位置。

```rust
pub struct ModuleSet { modules: BTreeMap<ModuleName, ModuleDecl> }
impl ModuleSet {
    pub fn get(&self, name: &ModuleName) -> Option<&ModuleDecl>;
    pub fn field(&self, r: &FieldRef) -> Option<&FieldDecl>;
    pub fn iter(&self) -> impl Iterator<Item = &ModuleDecl>;
}

pub struct ModuleDecl {
    pub name: ModuleName,
    pub builtin: bool,
    pub version: Option<u32>,                   // FR-TPL-004
    pub fields: BTreeMap<FieldName, FieldDecl>,
    pub declared_at: SourceLocation,
}

pub struct FieldDecl {
    pub name: FieldName,
    pub ty: FieldType,
    pub required: bool,
    pub description: Option<String>,
    pub default: Option<FieldValue>,            // 只有內建模組有（§5.6）
    pub declared_at: SourceLocation,
}

pub enum FieldType {
    String,
    Integer { min: Option<i64>, max: Option<i64> },
    Decimal { min: Option<f64>, max: Option<f64> },
    Boolean,
    Color,
    Enum { options: Vec<String> },
    List { items: Box<FieldType>, min_items: Option<usize>, max_items: Option<usize> },
    Map { keys: KeyFormat, values: Box<FieldType> },
}
pub enum KeyFormat { Identifier, Word }

impl FieldType {
    /// §5.5 的檢查與正規化，例如色碼轉成小寫。
    pub fn check(&self, raw: &Located<RawValue>) -> Result<FieldValue, Violation>;
}

/// 設定檔中的值，尚未依欄位宣告檢查。不含 toml_edit 的型別。
pub enum RawValue {
    String(String), Integer(i64), Float(f64), Boolean(bool), Datetime(String),
    Array(Vec<Located<RawValue>>), Table(BTreeMap<String, Located<RawValue>>),
}

/// 通過檢查的值。
pub enum FieldValue {
    String(String), Integer(i64), Decimal(f64), Boolean(bool), Color([u8; 3]),
    Enum(String), List(Vec<FieldValue>), Map(BTreeMap<String, FieldValue>),
}

pub struct Violation {
    pub at: SourceLocation,  // 違反條件的值，清單元素或對照表的值時指向該元素
    pub element: Vec<ElementKey>, // 例如 ansi[3]、env.EDITOR
    pub condition: String,   // 違反的條件，例如「1 到 72 的整數」
}

pub struct Assignment {
    pub field: FieldRef,
    pub value: Located<RawValue>, // 位置是值所在的行（FR-PLN-008）
    pub key_at: SourceLocation,   // 鍵所在的行，供重複時列出（FR-PRF-012）
}
```

`FieldType` 是遞迴的：`List` 的元素與 `Map` 的值都是一個 `FieldType`，對應 §5.5 中 `items`、`values` 以欄位宣告的寫法描述。

```rust
pub struct Profile { pub rules: Vec<ProfileRule> }      // profile.toml 不存在時為空

pub struct ProfileRule {
    pub index: usize,                 // 在檔案中的順序，只用來顯示；結果不受它影響（FR-PRF-011）
    pub os: Option<Platform>,
    pub hostname: Option<String>,
    pub tags: BTreeSet<Tag>,
    pub enable: Vec<Located<OutputName>>,
    pub values: Vec<Assignment>,      // 覆寫層級 3
    pub declared_at: SourceLocation,  // [[rules]] 所在的行
}
impl ProfileRule {
    /// os 相同、hostname 不分 ASCII 大小寫相同、tags 都在本機標籤中；未寫的項目不限制。
    pub fn matches(&self, host: &HostContext, local: &LocalSettings) -> bool;
}
impl Profile {
    /// 每個啟用的輸出，以及啟用它的條件。名稱不在 manifest 中的也列入，由預檢警告。
    pub fn enabled(&self, host: &HostContext, local: &LocalSettings) -> EnabledOutputs;
}
pub struct EnabledOutputs { by_output: BTreeMap<OutputName, Vec<usize>> }

pub struct LocalSettings {
    pub source: Option<Located<RawPath>>,
    pub tags: BTreeSet<Tag>,
    pub values: Vec<Assignment>,      // 覆寫層級 4
    pub file: Option<LoadedFile>,     // 本機設定不存在時為 None
}
```

`ProfileRule::matches` 與 `Profile::enabled` 是純函式，Profile 的 AND、OR 規則（FR-PRF-007）只寫在這裡，`ValueResolver`、`Planner` 與 `QueryService` 都呼叫它們。

**內建資源**（`config::builtin`）：內建模組、內建輸出與內建範本是編進執行檔的文字（DC-004）。內建模組與內建輸出以 §5.5、§5.4 的 TOML 格式撰寫，由同一個解析器讀取，內建輸出寫成 `[outputs.*]` 表格。

```rust
pub enum BuiltinTemplate { Ghostty, Alacritty, PosixShell, Powershell } // 清單見 §5.6
impl BuiltinTemplate {
    pub fn from_name(name: &str) -> Option<Self>;  // builtin: 之後的名稱
    pub fn name(self) -> &'static str;
    pub fn text(self) -> &'static str;
}
```

內建資源有錯誤是程式的錯誤，不是使用者的錯誤。單元測試逐一確認內建資源載入時沒有任何診斷，所以執行時遇到這類錯誤會 panic。

### 4.3 ConfigLoader

**責任**：讀取本機設定、manifest、Profile 與使用者模組，合併內建宣告，建立 `EffectiveConfig`。保留每個值的位置，一個檔案有錯時繼續檢查其他檔案。

**位置**：`layout` module 依 §5.1 決定三個目錄，`ConfigLoader` 使用它。

```rust
pub struct Layout { pub source: HomePath, pub config: HomePath, pub state: HomePath }
pub struct BatchId(String); // §5.1 的批次編號，可能帶 -2、-3；由 RecoveryManager 選定（§4.13）
impl Layout {
    /// 設定目錄，依 XDG_CONFIG_HOME 或預設位置（§5.1）。
    pub fn config_dir(host: &HostContext, diags: &mut Diagnostics) -> Checked<HomePath>;
    /// 狀態目錄，依 XDG_STATE_HOME 或預設位置。取得鎖定前使用（§6.7）。
    pub fn state_dir(host: &HostContext, diags: &mut Diagnostics) -> Checked<HomePath>;
    /// 三個目錄，來源目錄依本機設定的 source。
    pub fn new(host: &HostContext, local: &LocalSettings, diags: &mut Diagnostics) -> Checked<Layout>;
    /// 指定來源目錄；初始化使用（§4.16）。
    pub fn with_source(host: &HostContext, source: HomePath, diags: &mut Diagnostics) -> Checked<Layout>;
    pub fn local_settings(&self) -> HomePath;   // <設定目錄>/local.toml
    pub fn manifest(&self) -> HomePath;
    pub fn profile(&self) -> HomePath;
    pub fn modules(&self) -> HomePath;
    pub fn state_file(&self) -> HomePath;
    pub fn lock_file(&self) -> HomePath;
    pub fn txn_dir(&self, batch: &BatchId) -> HomePath;     // <狀態目錄>/txn/<批次編號>
    pub fn backup_dir(&self, batch: &BatchId) -> HomePath;  // <狀態目錄>/backups/<批次編號>
    pub fn reserved(&self) -> Vec<Reserved>;    // §5.2 的保留位置
}
```

**介面**：

```rust
pub struct ConfigLoader<'a> { fs: &'a dyn FileSystem, host: &'a HostContext }

impl<'a> ConfigLoader<'a> {
    pub fn new(fs: &'a dyn FileSystem, host: &'a HostContext) -> Self;

    /// 讀取 <設定目錄>/local.toml。不存在時回傳空的本機設定。
    pub fn load_local(&self, config_dir: &HomePath, diags: &mut Diagnostics)
        -> Checked<LocalSettings>;

    /// 讀取來源目錄。永遠回傳結果；有錯誤的部分不進入結果，記在 diags。
    pub fn load(&self, layout: &Layout, diags: &mut Diagnostics) -> EffectiveConfig;
}

/// 解析 manifest 的文字，不經過檔案系統。ManifestService 用它確認修改後的內容。
pub fn parse_manifest(text: &str, origin: &Origin, diags: &mut Diagnostics) -> Checked<Manifest>;
```

呼叫順序：`Layout::config_dir` → `load_local` → `Layout::new` → `load`。`layout` 在第 0 層，不能使用 `FileSystem`，所以 `Layout::new` 只做文字的處理；三個目錄是否互相包含要依 §6.2 的位置鍵判定，由初始化負責（§5.2、§4.16）。

**前置條件**：會寫入的子指令已取得鎖定（§6.7），讀到的檔案在指令結束前不會被其他 rosamta 程序改寫。預覽與查詢不取得鎖定，讀到的是當下的檔案。

**後置條件**：

- 讀取規則依 §5.3：UTF-8、略過 BOM、解析前把 CRLF 轉成 LF、不認得的鍵是錯誤並列出可用的鍵、名稱依 §5.3 的格式。
- 每個檔案經由 `FileSystem` 讀取並記錄觀測。manifest 與本機設定的文字與觀測保存在 `LoadedFile`，供 `ManifestService` 與初始化修改時確認版本（§6.3）。
- 本機設定不存在時為空；`profile.toml` 不存在時 `Profile` 為空；`modules/` 不存在時沒有使用者模組；`manifest.toml` 不存在是錯誤（§5.4）。
- `modules/` 依 §5.5 遞迴讀取所有 `*.toml`，其他副檔名略過。檔案依路徑排序後讀取，但結果與順序無關：同一模組在兩個檔案宣告是錯誤，不由順序決定。
- 內建輸出依 §5.4 與覆寫合併：以屬性為單位，`target` 的表格形式依平台合併；`removed = true` 的內建輸出不進入 `Manifest::get`，只列在 `removed_builtins`（§5.13）。
- 語法錯誤時，該檔案只回報語法錯誤，整個檔案不進入結果。語法正確時，每個輸出表格、模組宣告與 Profile 條件分別檢查，一個表格的錯誤不影響其他表格。有錯誤的輸出不進入 `Manifest`，但 `is_declared` 仍認得它的名稱。
- 有任何錯誤時 `is_complete()` 為 `false`。套用與查詢依此決定是否繼續（§4.6、§4.9）。
- `target` 與 `source` 只檢查語法（`RawPath::parse`），三個平台的 `target` 都檢查。展開要用到目前主機的環境變數，由預檢處理（§4.6）。

**FR-MAN-004 的第一處**：TOML 不允許重複的表格與鍵，toml_edit 在遇到第二處時回報語法錯誤。`ConfigLoader` 取出錯誤中的鍵名，把檔案截到第二處所在行之前再解析一次。截斷的部分在第一個錯誤之前，所以仍是合法的 TOML，從中可以找到第一處的位置。兩處都列在診斷中。第二處在同一行內（例如 inline table 中的重複鍵）時，兩處都指向這一行。

**錯誤**：

| 代碼 | 情況 |
| --- | --- |
| `config.read-failed` | 讀取失敗，附作業系統的錯誤；或檔案、路徑中有連結（§6.1） |
| `config.not-utf8` | 無法以 UTF-8 解碼，指出所在的行 |
| `config.syntax` | TOML 語法錯誤 |
| `config.duplicate` | 重複的輸出、表格或鍵，列出每一處（FR-MAN-004） |
| `config.unknown-key` | 不認得的鍵，列出該位置可用的鍵（FR-MAN-029） |
| `config.invalid-value` | 型別錯誤、名稱格式不符、`permissions` 超出範圍、`target` 語法錯誤等 |
| `config.missing-key` | 缺少必要的鍵，例如 `[outputs.*]` 沒有 `target`，欄位宣告沒有 `type` |
| `config.source-choice` | `template` 與 `copy` 都有或都沒有 |
| `config.unknown-builtin` | `[builtin.*]` 的名稱不是內建輸出，列出相近的名稱（§5.4） |
| `config.builtin-name` | `[outputs.*]` 使用內建輸出的名稱，提示改用 `[builtin.*]` |
| `config.removed-with-properties` | `removed = true` 的表格有其他屬性（§5.13） |
| `config.duplicate-module` | 同一個模組在兩個檔案宣告，列出每一處（§5.5） |
| `config.invalid-field-decl` | 欄位宣告不合理：`options` 為空或重複、`min` 大於 `max`、`min`/`max` 用在不適用的型別等 |
| `config.missing-manifest` | 來源目錄沒有 `manifest.toml` |

**隱藏的設計決定**：設定檔使用 TOML 與 toml_edit，其他 module 只看到 `RawValue`、`Located` 等本 module 的型別；BOM、換行與行號的對應；逐表格檢查的方式；找出重複的第一處的方法；內建資源的嵌入方式與合併演算法。

### 4.4 ValueResolver

**責任**：依四個覆寫層級決定每個欄位的值，保留值的來源，找出同一層級的重複設定（C2、FR-PRF-009–012）。

**介面**：

```rust
pub struct ValueResolver;

impl ValueResolver {
    pub fn resolve(config: &EffectiveConfig, host: &HostContext, local: &LocalSettings,
                   diags: &mut Diagnostics) -> ResolvedValues;
}

pub enum Layer { BuiltinDefault = 1, UserModule = 2, Profile = 3, Local = 4 }

pub struct ValueSource {
    pub layer: Layer,
    pub at: SourceLocation,     // 內建預設值為 Origin::Builtin
    pub rule: Option<usize>,    // Profile 層：是哪一個條件
}

pub struct Resolved {
    pub value: Located<RawValue>,
    pub source: ValueSource,
    pub overridden: Vec<ValueSource>, // 被較高層級取代的較低層級設定，由高到低
}

pub struct ResolvedValues {
    values: BTreeMap<FieldRef, Resolved>,
    conflicted: BTreeSet<FieldRef>,   // 有同層重複的欄位
}
impl ResolvedValues {
    pub fn get(&self, field: &FieldRef) -> Option<&Resolved>;
    pub fn is_conflicted(&self, field: &FieldRef) -> bool;
    pub fn iter(&self) -> impl Iterator<Item = (&FieldRef, &Resolved)>;
}
```

C2 的 `values` 與 `provenance` 兩個 map 合併成一個：值與來源一定同時存在。

**前置條件**：`config` 與 `local` 由 `ConfigLoader` 產生。`config` 不完整時仍可呼叫，結果只涵蓋讀到的部分。

**後置條件**：

- 層級 1 是內建模組欄位的 `default`；層級 2 是 `config.assignments`；層級 3 是 `matches` 成立的條件中的 `values`；層級 4 是 `local.values`。
- 每個欄位取有設定的最高層級。較高層級取代整個值，清單與對照表不合併：FR-PRF-010 以欄位為單位，合併清單需要另外定義刪除元素的寫法。
- 同一層級有兩個以上的設定時回報錯誤，列出每一處的檔案與行（FR-PRF-012）。該欄位記為 `conflicted`，不取這一層的值；`PreflightChecker` 因此不會再為它回報「缺少必填值」。
- 結果與檔名、模組名稱及條件的順序無關（FR-PRF-011）：同層重複是錯誤，不需要決定勝負。
- 引用未宣告的模組或欄位的設定也照樣解析，由 `PreflightChecker` 回報，這樣錯誤訊息可以列出名稱相近的欄位。
- 不檢查值的型別。未設定任何值的欄位不在結果中。

**錯誤**：`values.duplicate`（同層重複，FR-PRF-012）。

**隱藏的設計決定**：層級的順序與「整個值取代」的規則。

### 4.5 範本與原樣複製

`template` module 讀取範本與原樣複製的來源、編譯範本、取得欄位引用並渲染。MiniJinja 只出現在這個 module 內。

**介面**：

```rust
pub struct TemplateEngine { /* MiniJinja 的 Environment */ }

impl TemplateEngine {
    pub fn new() -> Self;
    pub fn compile(&self, text: TemplateText, diags: &mut Diagnostics) -> Checked<CompiledTemplate>;
    /// 範本的全域名稱：host 與註冊的函式（§5.5 的模組名稱限制）。
    pub fn global_names() -> &'static [&'static str];
}

pub struct TemplateText { pub origin: Origin, pub text: String, pub observation: Option<Observation> }

pub struct CompiledTemplate { /* 中繼資料、引用、編譯結果 */ }

impl CompiledTemplate {
    pub fn meta(&self) -> &TemplateMeta;
    pub fn references(&self) -> &References;
    /// 引用了 modules 中不存在的模組、欄位或 host 屬性，附上所在的行（USE-003）。
    pub fn unknown_references(&self, modules: &ModuleSet) -> Vec<UnknownReference>;
    pub fn render(&self, values: &TemplateValues, diags: &mut Diagnostics) -> Checked<Vec<u8>>;
}

pub struct TemplateMeta {                    // §5.9
    pub program: Option<String>,             // C1 的 targetProgram
    pub format_version: Option<String>,      // C1 的 targetFormatVersion
    pub requires: BTreeMap<ModuleName, Located<Vec<u32>>>,
}

pub struct References {
    pub fields: BTreeSet<FieldRef>,          // theme.foreground 這類引用
    pub whole_modules: BTreeSet<ModuleName>, // 只出現模組名稱的引用（§2.3）
}
impl References {
    pub fn mentions_module(&self, m: &ModuleName) -> bool; // 有該模組的欄位，或整個模組
    pub fn mentions_field(&self, f: &FieldRef) -> bool;    // 有該欄位，或整個模組
}

/// 交給範本的值：每個模組的每個宣告欄位都有一項，選填欄位沒有值時為 None；
/// 另有 host 的作業系統、主機名稱與本機標籤。
pub struct TemplateValues { /* 模組 → 欄位 → Option<FieldValue>，以及 host */ }

pub struct SourceLoader<'a> { fs: &'a dyn FileSystem, layout: &'a Layout, engine: &'a TemplateEngine }

impl<'a> SourceLoader<'a> {
    pub fn new(fs: &'a dyn FileSystem, layout: &'a Layout, engine: &'a TemplateEngine) -> Self;
    /// 讀取並編譯範本，或讀取原樣複製的來源檔。
    pub fn load(&self, output: &OutputDefinition, diags: &mut Diagnostics) -> Checked<LoadedSource>;
    /// 只取得引用。原樣複製不開啟來源檔（04 A9），直接回傳空的引用。
    pub fn references(&self, output: &OutputDefinition, diags: &mut Diagnostics) -> Checked<References>;
}

pub enum LoadedSource {
    Template(CompiledTemplate),
    Verbatim { bytes: Vec<u8>, observation: Observation },
}
```

**前置條件**：`render` 的 `values` 已通過 `check_inputs`。`TemplateValues` 的建構函式是 `pub(crate)`，core 中只有 `check_inputs` 呼叫它，§11 逐一核對；core 以外的程式無法建立它。

**後置條件**：

- `TemplateEngine::new` 從 `Environment::empty()` 開始，未定義的值為 `Strict`，不做 HTML 跳脫，保留結尾換行，只註冊下表的名稱（§2.3）。
- 渲染的變數：每個模組一個同名變數，值是它的欄位；另有 `host`。選填欄位沒有值時是 `none`（DATA-004）。
- `render` 的結果是渲染出的 UTF-8 位元組，不做其他轉換。原樣複製的 `bytes` 是來源檔的位元組（FR-TPL-006）。
- 範本與原樣複製的來源都從來源目錄的 handle 走訪讀取（§6.1）；`builtin:` 範本取自 `BuiltinTemplate::text`，不讀檔案。
- 引用依 §2.3 從 `undeclared_variables(true)` 取得：去掉全域名稱後，第一段是模組；有第二段時記為欄位引用，更深的部分（例如 `shellrc.env.EDITOR` 的 `EDITOR`）不再細分；只有第一段時記為整個模組。

範本可用的名稱。這是範本語言的公開介面，同一個 major 版本內只增不減，每一個都有回歸測試（MNT-001）：

| 名稱 | 種類 | 說明 |
| --- | --- | --- |
| `host.os` | 變數 | `"linux"`、`"macos"` 或 `"windows"` |
| `host.hostname` | 變數 | 主機名稱（IF-011） |
| `host.tags` | 變數 | 本機設定的標籤，依名稱排序 |
| `rgb(color)` | 函式 | 色碼的 R、G、B，0–255 的整數清單（FR-TPL-007） |
| `rgb_unit(color)` | 函式 | 色碼的 R、G、B，0–1 的小數清單（FR-TPL-008） |
| `none` | 測試 | 選填欄位沒有值（§2.3） |
| `lower`、`upper` | 過濾器 | 轉成小寫、大寫 |
| `replace(old, new)` | 過濾器 | 取代所有出現處 |
| `split(sep)`、`join(sep)` | 過濾器 | 切開字串、以分隔字串連接清單 |
| `items` | 過濾器 | 對照表轉成依名稱排序的（名稱，值）清單 |
| `length` | 過濾器 | 字串、清單或對照表的長度 |
| `tojson` | 過濾器 | JSON 表示。控制字元（含 U+007F）一律寫成 `\uXXXX`，字串的結果因此也是合法的 TOML 字串 |
| `sh_quote` | 過濾器 | POSIX shell 的單引號寫法 |
| `ps_quote` | 過濾器 | PowerShell 的單引號寫法 |

`host` 不提供 home 與環境變數：範本的輸出依主機改變的部分越少，預覽越容易預期；需要時可在之後的 minor 版本加入，不破壞相容性。

**`include` 不開放**：第一版的範本不能載入其他範本。`include`、`import`、`from` 與 `extends` 經由 loader 時一律失敗，訊息說明第一版不支援，並提示把共用的內容寫進同一個範本。理由：

- 開放後，引用分析要自行展開被載入的範本（§2.3）。範本名稱可以是變數，無法靜態展開，FR-QRY-004 就無法保證。
- 內建範本彼此獨立，不需要共用片段；使用者的範本可以用 `{% macro %}` 在同一個檔案中重複使用。
- 之後加入 `include` 不影響既有的範本。

loader 在渲染時才被呼叫，所以本次沒有執行到的分支中的 `include` 不會被發現，到了會執行的主機才回報。

**USE-003 的位置與提示**：

- 編譯與渲染的錯誤取 MiniJinja 錯誤中的範本名稱、行號與原因，範本名稱換成 `Origin`。
- 字串沒有結束時，MiniJinja 的錯誤指向檔案結尾（§2.3）。錯誤是語法錯誤、且行號是最後一行時，rosamta 掃描 `{{ }}` 與 `{% %}` 中的引號，找到沒有結束的字串，補上它開始的行號。
- `unknown_references` 的結果沒有位置（§2.3）。rosamta 在範本的 `{{ }}` 與 `{% %}` 中搜尋這個路徑的寫法，例如 `theme.forground` 或中間有空白的 `theme . forground`，列出每一個出現的行。找不到時只列範本路徑。不使用 MiniJinja `unstable_machinery` 的語法樹：它的 span 不是穩定的 API。
- 提示依錯誤種類產生：不存在的欄位或模組，列出編輯距離 2 以內的名稱，沒有時列出該模組的所有欄位；語法錯誤，依 MiniJinja 的錯誤種類給固定的提示，例如補上對應的 `{% endif %}`；未註冊的函式、過濾器或測試，列出上表中名稱相近的項目。

**錯誤**：

| 代碼 | 情況 |
| --- | --- |
| `template.read-failed` | 讀取來源失敗、來源不是一般檔案，或路徑中有連結（§6.1、A9） |
| `template.not-utf8` | 範本不是 UTF-8。原樣複製沒有這項限制 |
| `template.meta` | 中繼資料的 TOML 有錯，行號對應範本的實際行 |
| `template.syntax` | 範本語法錯誤 |
| `template.render` | 渲染時的錯誤，例如未定義的值、`rgb` 收到不是色碼的字串 |
| `template.include` | 使用了 `include`、`import`、`from` 或 `extends` |
| `template.unknown-reference` | 引用了不存在的模組、欄位或 `host` 屬性（由 `PreflightChecker` 依 `unknown_references` 回報） |

**隱藏的設計決定**：範本引擎是 MiniJinja 與它的設定；引用的取得方式與路徑的切法；錯誤位置的補充方式；`include` 的處理。換掉範本引擎時，只要保留上表的名稱與語法，其他 module 不受影響。

### 4.6 PreflightChecker

**責任**：寫入前檢查整批輸入與計畫，收集所有不通過的檢查（FR-PLN-007、USE-004）。只檢查，不寫入，也不決定 exit code。

**介面**：

```rust
pub struct AnalysisContext<'a> {
    pub host: &'a HostContext,
    pub layout: &'a Layout,
    pub local: &'a LocalSettings,
    pub config: &'a EffectiveConfig,
    pub values: &'a ResolvedValues,
    pub enabled: &'a EnabledOutputs,
    pub sources: &'a BTreeMap<OutputName, LoadedSource>, // 啟用輸出中成功載入的來源
    pub state: &'a StateLoadResult,
}

pub struct PreflightChecker;

impl PreflightChecker {
    /// 第一階段：不需要觀測目標的檢查。全部通過時回傳交給範本的值。
    pub fn check_inputs(ctx: &AnalysisContext, diags: &mut Diagnostics) -> Checked<TemplateValues>;
    /// 第二階段：目標路徑的碰撞與保留位置。
    pub fn check_targets(ctx: &AnalysisContext, targets: &TargetSet, diags: &mut Diagnostics) -> Checked<()>;
    /// 第三階段：依計畫中觀測到的目標與來源。
    pub fn check_plan(ctx: &AnalysisContext, plan: &ApplyPlan, diags: &mut Diagnostics) -> Checked<()>;
}
```

`TargetSet` 與 `ApplyPlan` 由 `Planner` 產生（§4.12）。`TargetSet` 是每個啟用輸出在目前平台的目標，包含正規化後的路徑、位置鍵（§6.2）與 `target` 的宣告位置，以及保留位置的位置鍵；它不需要渲染，欄位值有錯時也能建立。

**前置條件**：每個階段的輸入都已產生。`ApplyService` 依序呼叫：`check_inputs` 與 `check_targets` 都會執行，不因前者失敗而略過；兩者都通過才渲染並建立計畫，再呼叫 `check_plan`。如此 USE-004 的三種錯誤（欄位值、目標碰撞、manifest 屬性）會在同一次回報（§4.15）。

**後置條件**：每個階段都執行該階段的所有檢查，不在第一個錯誤停止。已經由前面的步驟回報的問題不重複回報，例如 `ConfigLoader` 回報過的檔案，或 `conflicted` 的欄位。

| 階段 | 檢查 | 需求 | 代碼 |
| --- | --- | --- | --- |
| 輸入 | 狀態檔損毀或由較新的版本寫入（§5.10） | REL-004 | `preflight.state` |
| 輸入 | 使用者模組的名稱與內建模組或範本的全域名稱相同（§5.5） | FR-MOD-004 | `preflight.module-name` |
| 輸入 | 設定值引用未宣告的模組或欄位，列出相近的名稱（§5.5） | FR-PLN-008 | `preflight.unknown-field` |
| 輸入 | 每一個設定值都符合欄位的型別、範圍與格式，包括不成立的 Profile 條件中的值 | FR-PLN-008、DATA-001 | `preflight.invalid-value` |
| 輸入 | 必填欄位有最終值；缺少時指出欄位宣告所在的檔案（04 §2） | FR-PLN-008 | `preflight.missing-value` |
| 輸入 | Profile 的 `enable` 列出不存在的輸出（警告，§5.7） | — | `preflight.unknown-output` |
| 輸入 | 啟用的輸出有目前平台的 `target` | FR-MAN-008 | `preflight.no-target` |
| 輸入 | 啟用輸出的 `target` 能展開，結果在 home 內（§6.1） | FR-MAN-009、FR-MAN-010 | `preflight.target-path` |
| 輸入 | 啟用輸出的來源都已載入、編譯。錯誤由 `SourceLoader::load` 回報（§4.5），這裡只確認每個啟用輸出都在 `sources` 中 | FR-PLN-007、USE-003 | `template.*` |
| 輸入 | 範本的 `requires` 與模組的格式版本相容（§5.9） | FR-TPL-005 | `preflight.incompatible-template` |
| 輸入 | 範本沒有引用不存在的模組、欄位或 `host` 屬性，包括不會執行的分支（§2.3） | USE-003 | `template.unknown-reference` |
| 目標 | 兩個輸出的位置鍵相同，或一個是另一個的前段，列出每個輸出的名稱與 `target` 的位置 | FR-PLN-009、FR-PLN-010 | `preflight.collision` |
| 目標 | 目標位於保留位置、是保留目錄的上層，或檔名含 `.rosamta-tmp.`（§5.2） | FR-PLN-007、DATA-005 | `preflight.reserved` |
| 目標 | 目標的某一層上層目錄是連結（§6.1） | DC-002、SEC-002 | `preflight.linked-parent` |
| 目標 | 走訪目標的上層目錄失敗，例如權限不足（§4.12） | FR-PLN-007 | `preflight.target-path` |
| 計畫 | 必須操作的目標或來源的類型不受支援，包括目標的某一層上層不是目錄（04 §3.1、§4.12） | FR-PLN-007、DC-003 | `preflight.unsupported-type` |
| 計畫 | 要接管的連結與狀態目錄不在同一個檔案系統（§6.5） | SEC-002 | `preflight.backup-device` |

嚴格模式的 drift 衝突不屬於預檢，由 `ApplyService` 的模式檢查處理（04 §2、§4.15）。

`check_inputs` 檢查所有層級的值，而不只是最終值：值的型別與主機無關，在一台主機上被覆寫的錯誤值，到了另一台主機仍會出錯，因此在每台主機都回報。必填的檢查只看最終值。

**錯誤**：上表的代碼。`check_inputs` 有任何錯誤時回傳 `Err`，否則回傳 `TemplateValues`；其他兩個階段同樣以 `Checked<()>` 表示是否通過。

**隱藏的設計決定**：檢查的順序與分組；名稱相近的判定方式（編輯距離）；如何避免重複回報。

### 4.7 TomlEditor

**責任**：以保留格式的方式修改 TOML 設定檔（FR-MAN-030），供 `ManifestService` 與初始化使用。只處理文字，不讀寫檔案，也不知道設定的意義。

**介面**（`docedit`）：

```rust
pub struct TomlEditor { /* toml_edit 的文件、換行方式、是否有 BOM */ }

pub enum EditValue {
    String(String), Integer(i64), Float(f64), Bool(bool),
    Array(Vec<EditValue>), InlineTable(Vec<(String, EditValue)>),
}

impl TomlEditor {
    /// 混用 LF 與 CRLF 時拒絕（§5.3）。
    pub fn parse(text: &str, origin: &Origin, diags: &mut Diagnostics) -> Checked<Self>;
    pub fn get(&self, path: &[&str]) -> Option<EditValue>;
    pub fn has_table(&self, path: &[&str]) -> bool;
    /// 設定值；表格不存在時建立。
    pub fn set(&mut self, path: &[&str], value: EditValue);
    pub fn remove(&mut self, path: &[&str]) -> bool;
    /// 移除整個表格，連同緊接在上方的註解（§5.3）。
    pub fn remove_table(&mut self, path: &[&str]) -> bool;
    /// 還原換行方式與 BOM 後的完整文字。
    pub fn to_text(&self) -> String;
}
```

`path` 是鍵的序列，例如 `["outputs", "nvim", "target", "linux"]`。

**後置條件**：§5.3 的編輯規則全部在這裡實作：修改值時沿用原值的 decor；修改 inline table 中的鍵時維持 inline table；CRLF 的檔案先轉成 LF 再轉回；保留 BOM；新增的鍵沿用同一表格最後一個鍵的縮排；新增的表格加在檔案結尾，前面空一行；移除表格時一併移除上方的註解。`to_text` 與原文相比，只有被修改的鍵、表格所在的行不同。pilot P4 的案例改寫成這個型別的單元測試。

**錯誤**：`docedit.mixed-line-endings`（混用換行，指出第一個不同的行）；`config.syntax`（無法解析）。

**隱藏的設計決定**：toml_edit 的 decor 與表格位置的處理，以及 §5.3 第 3、4 點的換行與縮排規則。

### 4.8 ManifestService

**責任**：納入管理、修改輸出屬性、停止管理（FR-MAN-023–025），只修改這次操作涉及的部分（FR-MAN-030）。不改動任何目標檔。

**介面**（`manage`）：

```rust
pub struct ManifestService<'a> {
    fs: &'a dyn FileSystem, host: &'a HostContext, layout: &'a Layout,
    state: &'a StateStore<'a>, sync: &'a dyn SyncHook,
}

impl<'a> ManifestService<'a> {
    pub fn new(fs: &'a dyn FileSystem, host: &'a HostContext, layout: &'a Layout,
               state: &'a StateStore<'a>, sync: &'a dyn SyncHook) -> Self;
    pub fn add(&self, lock: &WriteLock, req: AddRequest, diags: &mut Diagnostics)
        -> Checked<OperationResult>;
    pub fn update(&self, lock: &WriteLock, name: &OutputName, patch: OutputPropertyPatch,
                  diags: &mut Diagnostics) -> Checked<OperationResult>;
    pub fn forget(&self, lock: &WriteLock, name: &OutputName, diags: &mut Diagnostics)
        -> Checked<OperationResult>;
}

pub struct AddRequest {
    pub target: PathBuf,            // 絕對路徑；相對路徑由執行檔依工作目錄轉換
    pub name: Option<OutputName>,   // 未指定時由檔名產生
}

pub enum PatchValue<T> { Keep, Set(T), Reset }   // Reset：使用者輸出移除該鍵；內建輸出回到內建值

pub struct OutputPropertyPatch {
    pub source: PatchValue<SourceSpec>,
    pub target: Vec<(Option<Platform>, PatchValue<RawPath>)>, // None 表示三個平台
    pub ownership: PatchValue<Ownership>,
    pub permissions: PatchValue<Permissions>,
    pub takeover: PatchValue<bool>,
}

pub struct OperationResult {
    pub output: OutputName,
    pub changes: Vec<FileChange>,   // 建立或修改的檔案，例如來源副本、manifest、狀態檔
    pub unchanged: bool,            // 指定的值與現況相同，沒有寫入
}
```

**共同的前置條件**：已取得鎖定。manifest 能完整載入，沒有任何錯誤：toml_edit 無法編輯有語法錯誤的檔案，而在有錯的 manifest 上修改，結果是否正確也無法確認。其他設定檔的錯誤不影響管理指令。

**共同的後置條件**：

- manifest 經由 `TomlEditor` 修改，再以 `parse_manifest` 解析修改後的文字；有錯誤時不寫入。這樣指令不會寫出自己無法讀取的 manifest。
- manifest 依 §6.3 以「取代」寫入，被取代的必須是載入時的版本；使用者在指令執行期間修改了 manifest 時，指令失敗，保留使用者的版本。
- 不改動任何目標檔。

**add**（FR-MAN-023）：

1. 依 §6.1 把 `target` 轉成 `HomePath`。它必須在 home 內、不在保留位置（§5.2），而且是一般檔案，路徑中沒有連結。
2. 任何有效輸出（包括內建輸出）在目前平台的目標與它的位置鍵相同時失敗，回報該輸出的名稱。
3. 名稱：使用 `req.name`；未指定時取檔名，去掉開頭的 `.`，把不符合輸出名稱格式的字元換成 `-`，合併連續的 `-` 並去掉頭尾的 `-`，例如 `.gitconfig` 為 `gitconfig`、`init.lua` 為 `init-lua`；結果為空時用 `file`。名稱已被使用者輸出、內建輸出或已移除的內建輸出使用時，產生的名稱依序加上 `-2`、`-3`，到 `-99` 仍被占用就失敗；指定的名稱被占用時直接失敗。
4. 經由 `FileSystem` 讀取目標的內容並記錄觀測，依 §5.4 在 `files/` 下以「建立」寫入副本，需要時建立上層目錄。預定位置被占用時，包括在最後一次確認後才被占用，改用下一個名稱（§6.3）。
5. 在 manifest 結尾新增 `[outputs.<名稱>]`，寫入 `copy` 與目前平台的 `target.<平台>`，路徑以 `~/` 開頭、以 `/` 分隔。只寫目前平台：檔案只在這個平台確認過，其他平台的路徑可能不同；在其他平台啟用時，FR-MAN-008 會指出缺少的目標路徑。
6. 取代 manifest。失敗時依 §6.3 刪除本次建立的副本。

結果列出副本與 manifest。輸出不會自動啟用，回報提示在 Profile 中啟用；啟用後，目標與副本相同，依 FR-MAN-021 直接納管，動作為「不變」。

**update**（FR-MAN-024）：

1. 使用者輸出修改 `[outputs.<名稱>]`；未移除的內建輸出修改或新增 `[builtin.<名稱>]`，只寫入這次指定的屬性。已移除的內建輸出、不存在的輸出都失敗，後者列出相近的名稱。
2. 新的值依 §5.4 檢查：`permissions` 的範圍、`target` 與來源路徑的語法。
3. 設定 `source` 時，寫入 `template` 或 `copy`，並移除同一表格中的另一個。
4. 設定所有平台的 `target` 時寫成字串；設定單一平台時寫成 `target.<平台>`。原本是字串形式而只改一個平台時，改寫成表格形式，其他兩個平台沿用原字串，三個平台的結果才與修改前一致（只有指定的平台改變）。
5. `Reset`：使用者輸出移除該鍵，回到 §5.4 的預設值；內建輸出移除覆寫中的該鍵，回到內建宣告。`[builtin.*]` 因此變空時，移除整個表格。使用者輸出不能 `Reset` 必要的 `target` 與來源。
6. 修改後的文字與原文相同時不寫入，`unchanged` 為 `true`。

**forget**（FR-MAN-025）依 §5.13 的三個步驟：

1. 載入狀態檔。損毀時失敗，不改動任何檔案（REL-004）。找出名稱相同的所有紀錄：同一個輸出可能有兩筆紀錄（§5.10）。
2. 有紀錄、而且有任何一筆沒有 `forget_pending` 時，全部加上 `forget_pending`，提交狀態檔。
3. 輸出仍在 manifest 中時：使用者輸出移除 `[outputs.<名稱>]` 與上方的註解；內建輸出把 `[builtin.<名稱>]` 改成只有 `removed = true`，原有的其他屬性一併移除。取代 manifest。
4. 有紀錄時，刪除這些紀錄，提交狀態檔。

輸出不在 manifest 中，但有帶 `forget_pending` 的紀錄時，表示上次在第 3 步之後中止，只執行第 4 步。輸出不在 manifest 中也沒有紀錄時失敗。狀態檔的寫入依 `StateStore` 的提交規則（§4.11），被取代的必須是第 1 步載入的版本。

**錯誤**：

| 代碼 | 情況 |
| --- | --- |
| `manage.manifest-invalid` | manifest 有錯誤，附上載入時的診斷 |
| `manage.not-in-home`、`manage.reserved` | 納入的路徑不在 home 內，或位於保留位置 |
| `manage.not-regular-file` | 納入的路徑不存在、不是一般檔案，或路徑中有連結 |
| `manage.already-managed` | 已有輸出以這個路徑為目標 |
| `manage.name-taken` | 指定的名稱已被使用，或產生的名稱到 `-99` 仍被占用 |
| `manage.copy-location` | 來源副本的名稱到 `-99` 仍被占用 |
| `manage.unknown-output` | 沒有這個輸出，列出相近的名稱 |
| `manage.builtin-removed` | 修改已移除的內建輸出，提示先刪除 `[builtin.*]` 表格 |
| `manage.invalid-patch` | 修改的值不合法，或 `Reset` 必要的屬性 |
| `manage.concurrent-edit` | manifest 或狀態檔在指令執行期間被修改 |
| `manage.write-failed` | 寫入失敗，附作業系統的錯誤；說明已建立的副本是否已刪除 |
| `docedit.mixed-line-endings`、`preflight.state` | 見 §4.7、§4.6 |

**隱藏的設計決定**：輸出名稱與來源副本名稱的產生規則；寫入的順序與失敗時的處理；停止管理的三個步驟。

### 4.9 QueryService

**責任**：唯讀的查詢：列出輸出（FR-MAN-026）、依模組或欄位找出引用的輸出（FR-QRY-001、FR-QRY-002、FR-QRY-004）、說明輸出的來源與引用（FR-QRY-003），以及欄位的最終值與來源（FR-PRF-013）。不寫入任何檔案（FR-QRY-006）。

**介面**（`query`）：

```rust
pub struct QueryService<'a> {
    fs: &'a dyn FileSystem, host: &'a HostContext, layout: &'a Layout,
    config: &'a EffectiveConfig, local: &'a LocalSettings,
    values: &'a ResolvedValues, sources: &'a SourceLoader<'a>,
}

impl<'a> QueryService<'a> {
    pub fn new(/* 同上的欄位 */) -> Self;
    pub fn list_outputs(&self, diags: &mut Diagnostics) -> Vec<OutputListing>;
    pub fn find_outputs(&self, query: &RefQuery, diags: &mut Diagnostics) -> Checked<FindResult>;
    pub fn describe_output(&self, key: &OutputKey, diags: &mut Diagnostics) -> Checked<Vec<OutputDescription>>;
    pub fn describe_value(&self, field: &FieldRef, diags: &mut Diagnostics) -> Checked<ResolvedValueDescription>;
}

pub enum RefQuery { Module(ModuleName), Field(FieldRef) }
pub enum OutputKey { Name(OutputName), Path(PathBuf) } // Path 為絕對路徑

pub struct OutputSummary {
    pub name: OutputName,
    pub target: TargetDisplay,  // 目前平台展開後的路徑；沒有目前平台的 target，或無法展開時說明原因
    pub enabled: bool,
}

pub struct OutputListing {
    pub summary: OutputSummary,
    pub kind: ListingKind,               // 使用者、內建、覆寫過的內建、已移除的內建
    pub source: Option<SourceSpec>,      // 已移除的內建輸出為 None
    pub ownership: Option<Ownership>,
    pub permissions: Option<Permissions>,
    pub takeover: Option<bool>,
    pub enabled_by: Vec<SourceLocation>, // 啟用它的條件
}

pub struct FindResult {
    pub outputs: Vec<OutputSummary>,      // 引用該模組或欄位的輸出
    pub undetermined: Vec<OutputSummary>, // 無法讀取或編譯範本，無法判定的輸出
}

pub struct OutputDescription {
    pub summary: OutputSummary,
    pub source: SourceSpec,
    pub references: References,           // 原樣複製為空
}

pub struct ResolvedValueDescription {
    pub field: FieldRef,
    pub declared_at: SourceLocation,
    pub value: Option<Located<RawValue>>, // 選填欄位在四個層級都沒有值時為 None
    pub source: Option<ValueSource>,
    pub overridden: Vec<ValueSource>,
}
```

**前置條件**：`config.is_complete()` 為 `true`；否則執行檔不建立 `QueryService`，只回報載入的錯誤，回傳 `1`。設定不完整時，查詢結果可能漏掉輸出或值的來源，FR-QRY-004 不允許這種結果被當成完整的答案。不取得鎖定（§6.7）。

**後置條件**：

- `list_outputs` 列出所有有效輸出與已移除的內建輸出，依名稱排序；未啟用的輸出也列出（FR-MAN-026）。
- `find_outputs` 對有效 manifest 中的**每一個**輸出取得引用，不論是否啟用、是否有目前平台的目標（FR-QRY-004）。原樣複製不開啟來源檔。範本無法讀取或編譯時，該輸出列在 `undetermined`，並回報錯誤，結果不會被當成完整的答案。模組或欄位沒有宣告時失敗。
- `describe_output` 以名稱查詢時回傳一項。以路徑查詢時，把路徑依 §6.1 轉成 home 內的路徑，再依 §6.2 的位置鍵與每個輸出在目前平台的目標比較，列出所有相符的輸出（碰撞時可能不只一個）；沒有相符的輸出時失敗。比較時走訪目錄但不開啟檔案。
- `describe_value` 回傳 `ValueResolver` 的結果與欄位宣告的位置。欄位在同一層級重複時，回報 `values.duplicate` 的錯誤，不給最終值。
- 所有方法只使用 `FileSystem` 的讀取與列出目錄，不清理殘留檔案（§6.5）。

**錯誤**：`query.unknown-module`、`query.unknown-field`、`query.unknown-output`（列出相近的名稱）、`query.no-match`（沒有輸出以該路徑為目標）、`query.path`（路徑不在 home 內或無法轉換），以及 §4.5 的 `template.*`。

**隱藏的設計決定**：路徑查詢的比對方式；引用無法判定時的處理。

### 4.10 FileSystem 與同步點

`fs` module 定義檔案系統的 trait、觀測與位置鍵的型別、鎖定，以及同步點。平台的實作在 `rosamta-platform`（§3.3），core 只認得這裡的型別。

**檔案身分與觀測**：

```rust
/// 一次執行內有效的檔案身分（ASM-04）：Linux 與 macOS 為 st_dev 加 st_ino，Windows 為 volume 序號加 128 位元 file ID。
pub struct FileId { /* 由 platform 建立 */ }
impl FileId {
    /// 寫進狀態檔的部分（§5.10）：Linux 與 macOS 不含 st_dev（ASM-15）。
    pub fn persistent(&self) -> PersistentId;
    /// 是否在同一個檔案系統（§6.5 連結的接管）。
    pub fn same_device(&self, other: &FileId) -> bool;
}
pub struct PersistentId { pub volume: Option<u64>, pub file: u128 }

pub struct Sha256([u8; 32]);
pub struct Version { pub sha256: Sha256, pub mode: Option<u32> } // mode 只在 Linux 與 macOS 有值，是實際的權限位元

pub enum EntryKind { Regular, Directory, Symlink, Other(OtherKind) }
pub enum OtherKind { Fifo, Device, Socket, ReparsePoint } // ReparsePoint：IO_REPARSE_TAG_SYMLINK 以外（§6.3）

/// §6.3 的觀測。
pub struct Observation {
    pub at: HomePath,
    pub parents: Vec<FileId>, // home 之下到父目錄，已存在的每一層
    pub entry: EntryState,
}
pub enum EntryState {
    /// 目標不存在。parents 比路徑的層數少時，表示上層目錄也不存在。
    /// blocker：第一個不存在的層之前，有一層存在但不是目錄（也不是連結）。
    Missing { blocker: Option<(HomePath, EntryKind)> },
    /// version 只有一般檔案才有。
    Present { kind: EntryKind, id: FileId, version: Option<Version> },
}

pub struct FileInspection { pub observation: Observation, pub content: Option<Vec<u8>> }
pub enum Content { Keep, Discard } // 一般檔案的內容是否保留在結果中

pub struct DirEntry { pub name: String, pub kind: EntryKind }

/// 管理與初始化回報的檔案變更（§4.8、§4.16）。
pub struct FileChange { pub at: HomePath, pub kind: ChangeKind }
pub enum ChangeKind { Created, Modified, Removed }
```

`Observation` 只能由 `FileSystem` 產生，或由 `fs` module 依已有的觀測推導，例如「本批建立的目錄」（見下方的綁定操作）。§5.11 的判定以它為輸入，單元測試以 `pub(crate)` 的建構函式直接建立（§8.1）。

**位置鍵**（§6.2）：

```rust
pub struct LocationKey { /* 已存在部分的身分，以及其後的名稱 */ }
pub enum Relation { Same, Ancestor, Descendant, Unrelated }
impl LocationKey {
    pub fn relation(&self, other: &LocationKey) -> Relation;
}
```

`relation` 依 §6.2 判定：兩者最深的已存在元件身分相同，而且其後的名稱相同時為 `Same`；一個位於另一個之內時為 `Ancestor` 或 `Descendant`。名稱是否轉成小寫，在建立位置鍵時依目錄是否區分大小寫決定，比較時不再處理。

**介面**：

```rust
pub trait FileSystem {
    // 唯讀：走訪不跟隨連結（§6.1），不需要鎖定
    fn inspect(&self, at: &HomePath, content: Content) -> Result<FileInspection, FsError>;
    fn list_dir(&self, at: &HomePath) -> Result<Vec<DirEntry>, FsError>;
    fn location_key(&self, at: &HomePath) -> Result<LocationKey, FsError>;
    fn case_sensitive(&self, dir: &HomePath) -> Result<bool, FsError>;
    /// 讀取一般檔案的綁定版本：prepare 走訪並核對類型，execute 經由持有的 handle 開檔讀取，
    /// confirm 重新走訪（§6.3）。其他類型以 FsErrorKind::NotRegular 失敗。
    fn prepare_read<'a>(&'a self, at: &HomePath) -> Result<Box<dyn Prepared + 'a>, FsError>;
    /// prepare_read 的三次呼叫連續執行，中間沒有同步點。
    fn read_file(&self, at: &HomePath) -> Result<(Vec<u8>, Observation), FsError> { /* … */ }

    // 程式自己的目錄與鎖定（§5.1、§6.7）
    fn ensure_private_dir(&self, at: &HomePath) -> Result<(), FsError>;
    fn lock(&self, file: &HomePath) -> Result<Box<dyn LockGuard>, FsError>;
    /// 與 at 同一目錄的暫存檔名稱：.<名稱>.rosamta-tmp.<程序編號>.<序號>（§5.1）。
    fn temp_name(&self, at: &HomePath) -> HomePath;
    /// 附加一行到 journal（§6.5）並 fsync。
    fn append_line(&self, lock: &WriteLock, file: &HomePath, line: &str) -> Result<(), FsError>;

    // 綁定的操作（§6.3）：三次呼叫
    fn prepare<'a>(&'a self, lock: &'a WriteLock, op: FsOp<'a>) -> Result<Box<dyn Prepared + 'a>, FsError>;
}

pub trait Prepared { fn execute(self: Box<Self>) -> Result<Box<dyn Executed>, FsError>; }
pub trait Executed { fn confirm(self: Box<Self>) -> Result<OpOutcome, FsError>; }
pub trait LockGuard {}
```

會改變檔案的方法都要求 `&WriteLock`，所以不取得鎖定的預覽與查詢無法寫入（FR-PLN-004、FR-QRY-006），由型別保證，不必靠 review。`ensure_private_dir` 是唯一的例外：取得鎖定之前，必須先有放鎖定檔的狀態目錄（§6.7）。它只建立 rosamta 自己的目錄，不碰任何輸出。

**綁定的操作**：每個操作分成三次呼叫，對應 §6.3 的步驟。core 在呼叫之間到達同步點（§8.2）：

| 呼叫 | §6.3 的步驟 | 之後的同步點 |
| --- | --- | --- |
| `prepare` | 1、2：從 home 走訪、核對父目錄與目標，持有 handle | `BeforeOperation`；擷取快照與製作備份為 `BeforeCapture` |
| `execute` | 3：系統呼叫；取代與刪除另含交換或移開後的核對，不符時換回原位 | `AfterOperation`；擷取快照與製作備份為 `AfterCapture` |
| `confirm` | 4、5：再從 home 走訪，確認路徑仍通往持有的目錄，目標是預期的結果；路徑已改變時經由 handle 復原 | — |

```rust
pub enum FsOp<'a> {
    CreateFile   { expect: &'a Observation, content: Source<'a>, mode: NewMode, temp: &'a HomePath },
    ReplaceFile  { expect: &'a Observation, content: Source<'a>, mode: NewMode, temp: &'a HomePath },
    DeleteFile   { expect: &'a Observation, temp: &'a HomePath },
    SetMode      { expect: &'a Observation, mode: u32 },
    CreateDir    { expect: &'a Observation, private: bool },
    RemoveDir    { expect: &'a Observation },                  // 必須是空的
    Capture      { expect: &'a Observation, into: &'a HomePath }, // 複製成 0600 的新檔，不覆蓋
    Backup       { expect: &'a Observation, into: &'a HomePath }, // hard link，不行時複製；連結只能 hard link
    RestoreLink  { backup: &'a Observation, expect: &'a Observation }, // 把連結的備份改名回目標，取代 expect
    MoveDir      { expect: &'a Observation, to: &'a Observation }, // 不覆蓋的改名（§6.3 初始化）
    RemoveTree   { expect: &'a Observation },                  // 只用於初始化的暫存目錄
}

pub enum Source<'a> {
    Bytes(&'a [u8]),
    Snapshot { copy: &'a Observation }, // 回復時從快照讀取，邊讀邊核對 SHA-256
}

pub enum NewMode {
    Declared(u32),          // 宣告的權限（FR-MAN-015）
    KeepExisting,           // 原檔的權限，Windows 依 §6.4 複製 DACL（FR-MAN-016）
    Default,                // 0666 去掉 umask；Windows 繼承目錄的 ACL
    Restore(SecurityInfo),  // 回復：擷取快照時記下的權限或 DACL
}

pub struct OpOutcome {
    pub result: Observation,               // 操作後的目標；刪除時為 Missing
    pub written: Option<Version>,          // 建立與取代：實際寫入的內容，邊寫邊計算
    pub captured: Option<CaptureInfo>,     // 擷取快照與製作備份
    pub content: Option<Vec<u8>>,          // prepare_read 讀到的內容
}
pub struct CaptureInfo { pub copy: Observation, pub sha256: Sha256, pub security: SecurityInfo, pub linked: bool }
pub struct SecurityInfo { /* Linux 與 macOS 的權限位元，或 Windows 的 DACL */ }
```

`expect` 的父目錄可以是本批建立的目錄：`CreateDir` 的 `OpOutcome::result` 帶有新目錄的身分，`Observation::under(&created, name)` 以它推導子項目的預期觀測。這就是 §6.3 步驟 1「本批建立的目錄，以建立時記下的身分核對」。

**前置條件**：`HomePath` 已依 §6.1 正規化。`temp` 由 `temp_name` 取得；批次中的暫存檔名稱已先寫入 journal（§6.5）。

**後置條件**：

- 走訪、開檔與各操作的系統呼叫依 §6.1、§6.3、§6.4 與 §6.8，只依賴 §7 的假設。
- `inspect` 不開啟非一般檔案（04 §3.1、A18）；一般檔案以 `O_NOFOLLOW | O_NONBLOCK` 開啟後，以 handle 核對類型再讀取（ASM-21）。`Content::Discard` 時只計算 SHA-256。
- `prepare` 的核對不符時回傳 `Stale`，不執行任何系統呼叫。`execute` 失敗時，平台已撤回的部分不留下效果，例如交換後發現版本不符而換回原位；`FsError::applied` 說明操作是否已生效。
- `confirm` 發現路徑不再通往持有的目錄時，經由 handle 復原，回傳 `Stale` 並標明已復原（ASM-06）。發現目標不是預期的結果時，不再動它。
- `written` 由實作在寫入暫存檔時計算，是實際寫入檔案的位元組；core 以它核對期望的版本（§4.14）。
- 錯誤不會被當成「不存在」：只有作業系統回報不存在（`ENOENT`、`ERROR_FILE_NOT_FOUND` 等）才是 `Missing`（03 C4）。

```rust
pub struct FsError {
    pub kind: FsErrorKind,
    pub at: HomePath,
    pub os: Option<OsErrorInfo>, // 作業系統的錯誤代碼與訊息
    pub applied: Applied,        // No、Undone（已經由 handle 復原）、Yes
}

pub enum FsErrorKind {
    LinkInPath { component: HomePath, points_to: Option<String> }, // §6.1
    NotRegular(EntryKind),
    Stale(StaleWhat),       // 觀測失效（§6.3）
    AlreadyExists,          // 不覆蓋的建立或改名遇到既有項目（A15、A22）
    Locked,                 // 另一個程序持有鎖定（§6.7）
    PermissionDenied,
    NoSpace,
    CrossDevice,            // hard link 無法跨檔案系統
    Unsupported,            // 例如檔案系統不支援交換時，連 renameat 也失敗
    Io,
}

pub enum StaleWhat {
    Parent { depth: usize }, // 某一層上層目錄的身分不符，或已不存在
    Kind, Identity, Version, // 目標的類型、身分或版本與觀測不同
    Occupied,                // 預期不存在的目標已存在
    PathMoved,               // 操作後路徑不再通往持有的目錄（ASM-06）
    UnexpectedResult,        // 操作後的目標不是預期的結果
}
```

**同步點**（§8.2）：

```rust
pub trait SyncHook { fn reach(&self, point: SyncPoint, op: OpKind, at: Option<&HomePath>); }
pub struct NoSync; // 正式執行使用，什麼都不做

pub enum SyncPoint {
    AfterPreflight, BeforeCapture, AfterCapture, BeforeOperation, AfterOperation,
    BeforeRecheck, BeforeCommit, BeforeCompletion, BeforeRollback,
}

pub enum OpKind {
    StateLoad, StateCapture, StateCommit, StateRollback,             // 狀態檔
    Capture, Backup, Create, Replace, Delete, SetMode, CreateDir,    // 目標
    RevertDelete, RevertReplace, RevertCreate, RevertSetMode, RevertRemoveDir, // 回復
    ManifestWrite, LocalWrite, SourceCopy, SourceMoveIn,             // 管理與初始化
}
```

**鎖定**（§6.7）：

```rust
pub struct WriteLock { /* LockGuard，持有到程序結束 */ }
impl WriteLock {
    /// 建立狀態目錄（§5.1），再取得 <狀態目錄>/lock 的獨占鎖定，不等待。
    pub fn acquire(fs: &dyn FileSystem, state_dir: &HomePath, diags: &mut Diagnostics) -> Checked<WriteLock>;
}
```

`WriteLock` 沒有其他建構函式，也不實作 `Clone`。執行檔對會寫入的子指令，在讀取任何設定之前呼叫 `acquire`，接著呼叫 `RecoveryManager::clean_stale`（§4.13）。

**錯誤**：`FsError` 不是診斷；呼叫它的類別把它轉成自己的代碼，例如 `config.read-failed`、`template.read-failed`。本節只產生 `lock.busy`（另一個 rosamta 程序正在執行，附鎖定檔的路徑）與 `lock.failed`（建立狀態目錄或開啟鎖定檔失敗）。

**隱藏的設計決定**：走訪與綁定的系統呼叫、各平台的檔案身分、交換不支援時的退回方式、暫存檔的命名、hard link 失敗時改用複製的判斷、DACL 的複製，以及鎖定的實作。這些都在 platform 中，core 只看到上面的型別。

### 4.11 StateStore

**責任**：狀態檔的格式、載入與提交（§5.10）。只處理狀態檔本身；快照與回復由 `RecoveryManager` 負責（§4.13）。

**資料型別**（`state`）：

```rust
pub struct StateFile { records: BTreeMap<HomePath, ManagedOutputRecord> } // 以 target 為鍵
impl StateFile {
    pub fn records(&self) -> impl Iterator<Item = &ManagedOutputRecord>; // 依 target 排序
    pub fn get(&self, target: &HomePath) -> Option<&ManagedOutputRecord>;
    pub fn put(&mut self, record: ManagedOutputRecord);     // 取代同一個 target 的紀錄
    pub fn remove(&mut self, target: &HomePath) -> bool;
    pub fn by_output(&self, name: &OutputName) -> Vec<&ManagedOutputRecord>;
    pub fn to_bytes(&self) -> Vec<u8>;                       // §5.10 的 JSON，依 target 排序
}

pub struct ManagedOutputRecord {
    pub output: OutputName,
    pub target: HomePath,
    pub parent: PersistentId,
    pub observed: ComparisonMetadata,   // O
    pub desired: ComparisonMetadata,    // D
    pub forget_pending: bool,
}

pub struct ComparisonMetadata { pub sha256: Sha256, pub mode: Option<Permissions> } // §5.10

pub enum StateLoadResult {
    Loaded(LoadedState),
    Corrupt { reason: String, line: Option<u32> }, // §5.10 的損毀
    Newer { format: u64 },                         // 由較新的版本寫入
    Unreadable(FsError),                           // 讀取失敗、路徑中有連結、不是一般檔案
}

/// 狀態檔不存在時，file 為空，observation 為 Missing。
pub struct LoadedState { pub file: StateFile, pub observation: Observation }
```

C3 的 `StateFile` 與 03 C4 的 `StateLoadResult` 在此成為具體型別。03 把「不存在」列為一種結果；這裡併入 `Loaded`，以 `observation` 區分：兩者在規劃時的處理相同（REL-003），差別只在提交時用建立還是取代。

**介面**：

```rust
pub struct StateStore<'a> { fs: &'a dyn FileSystem, layout: &'a Layout, sync: &'a dyn SyncHook }

impl<'a> StateStore<'a> {
    pub fn new(fs: &'a dyn FileSystem, layout: &'a Layout, sync: &'a dyn SyncHook) -> Self;
    pub fn load(&self) -> StateLoadResult;
    /// 寫入 new；被取代的必須是 base（A23）。回傳新的基準，供同一個指令的下一次提交使用。
    pub fn commit(&self, lock: &WriteLock, base: &LoadedState, new: &StateFile, temp: &HomePath)
        -> Result<LoadedState, FsError>;
    pub fn path(&self) -> HomePath;
}
```

**前置條件**：`commit` 的 `base` 是本指令載入或提交的結果，`temp` 由呼叫端取得（批次中已寫入 journal）。

**後置條件**：

- `load` 經由 `FileSystem::prepare_read` 讀取，在 `prepare_read` 與 `execute` 之間到達 `BeforeCapture`（`StateLoad`），A20 的第一列以此注入。載入規則依 §5.10：不存在時為空的 `Loaded`；損毀與較新的版本分開回報，都不會被當成空紀錄（REL-004）。
- `commit`：`base` 為 `Missing` 時以 `CreateFile`，否則以 `ReplaceFile` 寫入，`expect` 是 `base.observation`（§6.3）。權限為 `Declared(0o600)`；Windows 繼承狀態目錄的 DACL（§6.4）。在 `BeforeOperation`、`AfterOperation`（`StateCommit`）到達同步點。`written` 與 `new.to_bytes()` 的 SHA-256 不同時，視為寫入失敗。
- 相同的狀態是相同的位元組（§5.10）。`commit` 不比較新舊內容；是否需要提交由呼叫端決定（§4.14）。

**錯誤**：載入的結果由 `PreflightChecker` 回報為 `preflight.state`（§4.6）；`commit` 的錯誤由呼叫端轉成 `manage.concurrent-edit`、`manage.write-failed`（§4.8），或批次的執行錯誤（§4.14）。

**隱藏的設計決定**：JSON 的欄位名稱、十六進位的寫法與排序；損毀的判定；檔案身分轉成 `PersistentId` 的方式。

### 4.12 Planner

**責任**：判定啟用輸出的目標、比對狀態紀錄、渲染期望內容並觀測目標，依 §5.11 產生計畫。預覽、摘要與套用共用同一個 `Planner`（04 §2、FR-PLN-006）。不寫入任何檔案。

**介面**（`plan`）：

```rust
pub struct Planner<'a> { fs: &'a dyn FileSystem, host: &'a HostContext, layout: &'a Layout }

impl<'a> Planner<'a> {
    pub fn new(fs: &'a dyn FileSystem, host: &'a HostContext, layout: &'a Layout) -> Self;
    /// 不需要渲染的目標集合，給 PreflightChecker::check_targets（§4.6）。
    pub fn targets(&self, config: &EffectiveConfig, enabled: &EnabledOutputs) -> TargetSet;
    pub fn plan(&self, input: &PlanInput, diags: &mut Diagnostics) -> Checked<ApplyPlan>;
}

pub struct TargetSet { pub entries: Vec<TargetEntry>, pub reserved: Vec<(Reserved, LocationKey)> }
pub struct TargetEntry {
    pub output: OutputName,
    pub path: HomePath,
    pub declared_at: SourceLocation,          // target 的位置（FR-PLN-009）
    pub key: Result<LocationKey, FsError>,    // 走訪失敗時為錯誤，例如上層目錄是連結
}

pub struct PlanInput<'a> {
    pub config: &'a EffectiveConfig,
    pub enabled: &'a EnabledOutputs,
    pub sources: &'a BTreeMap<OutputName, LoadedSource>,
    pub values: &'a TemplateValues,
    pub state: &'a LoadedState,
    pub targets: &'a TargetSet,
    pub choices: &'a Choices,
    pub keep_current: bool,                   // 預覽要顯示差異時保留目前的內容（FR-PLN-003）
}

pub struct Choices {
    pub strict: bool,
    pub drift: Vec<(Selector, DriftChoice)>,  // FR-DRF-005、FR-DRF-006
    pub delete_modified: Vec<Selector>,       // FR-DRF-014
}
pub enum Selector { All, Path(Located<HomePath>) }
pub enum DriftChoice { Overwrite, Accept }
```

`plan` 不接收 `AnalysisContext`：`AnalysisContext` 定義在 `preflight`，而 `preflight` 依賴 `plan`（§3.2），反過來會形成循環。

**計畫的資料型別**：

```rust
pub struct ApplyPlan { items: Vec<PlanItem>, state: LoadedState, strict: bool }
impl ApplyPlan {
    pub fn items(&self) -> &[PlanItem];                       // 依執行順序，也是預覽列出的順序
    pub fn drift_conflicts(&self) -> impl Iterator<Item = &PlanItem>;
    pub fn strict_blocked(&self) -> bool;                     // 嚴格模式且有 drift 衝突（FR-DRF-009）
    pub fn state(&self) -> &LoadedState;
}

pub struct PlanItem {
    pub output: OutputName,
    pub target: HomePath,
    pub origin: ItemOrigin,
    pub action: Action,
    pub disposition: Disposition,
    pub drift: bool,                          // 有外部修改（FR-DRF-001），包括被接受或覆寫的
    pub observation: Observation,             // 規劃時的觀測，執行時每個操作都依它核對
    pub current: Option<Arc<[u8]>>,           // keep_current 時，目前一般檔案的內容
    pub desired: Option<DesiredContent>,      // 啟用輸出的期望內容 N；清除項目為 None
    pub record_before: Option<ManagedOutputRecord>,
    pub record: RecordChange,
}

pub enum ItemOrigin { Enabled { declared_at: SourceLocation, ownership: Ownership, takeover: bool }, Cleanup }
pub enum Action { Create, Update, Delete, Unchanged }       // C3 的 action

pub enum Disposition {                                      // C3 的 disposition
    Proceed,                                                // 依規則執行，包括只更新紀錄
    Takeover,                                               // 備份未受管理的目標後寫入（FR-MAN-019）
    Resolve(Choice),                                        // 使用者指定的處理
    Retain(RetainReason),
}
pub enum Choice { Overwrite, Accept, Delete }

pub enum RetainReason {
    DriftConflict,                // FR-DRF-004；嚴格模式下另由 strict_blocked 阻擋整批
    Unmanaged(EntryKind),         // 未受管理、沒有接管授權（FR-MAN-020）
    Modified,                     // 待清除的舊輸出已被修改（FR-DRF-013）
    UnsupportedType(EntryKind),   // 待清除的舊目標不是一般檔案
    LocationUnconfirmed,          // 紀錄的位置無法確認（§5.11、A5）
    Reserved,                     // 紀錄指向保留位置（§5.2）
    ForgetPending,                // 停止管理尚未完成（§5.13）
}

pub enum RecordChange {
    Keep,                                                     // record_before 不變
    Remove,                                                   // 刪除 record_before
    Put { observed: ComparisonMetadata, desired: ComparisonMetadata }, // 取代 record_before，或新增
}

/// 期望內容：規劃時渲染或讀入的位元組，建立後不可修改。
pub struct DesiredContent { bytes: Arc<[u8]>, metadata: ComparisonMetadata }
impl DesiredContent {
    pub fn bytes(&self) -> &[u8];
    pub fn metadata(&self) -> &ComparisonMetadata;            // N
}
```

C3 以字串表示的 `action`、`disposition`、`reason` 改成 enum，`reason` 併入 `Retain`：保留的項目一定有原因，其他項目沒有。每個 enum 值有固定的字串，供 IF-004 使用（MNT-003）。

**期望內容與 Executor 的傳遞契約**（03 C3 末段、04 §2）：

- 執行期參照就是 `DesiredContent`：規劃時把渲染結果或原樣複製來源的位元組放進記憶體，同時計算 SHA-256。`DesiredContent` 的建構函式是 `pub(crate)`，core 中只有 `Planner` 呼叫它（§11 核對），所以內容一定來自預檢時的那一次讀取或渲染。
- `Executor` 只寫入 `desired.bytes()`，不讀取來源、不渲染（A24）。寫入後以 `OpOutcome::written` 核對 `metadata().sha256`；不同時是執行錯誤（§4.14）。內容無法取得的情況因此只剩寫入途中損毀，§8.3 以 `FaultFs` 注入。
- 狀態檔只記錄 `metadata()`，不記錄 `bytes()`（SEC-001）。
- `observation` 是執行時每個操作的 `expect`。`Executor` 不重新觀測後再決定動作：動作由計畫決定，執行時只核對觀測是否仍然成立。

**前置條件**：`targets` 在 `check_inputs` 之前或之後呼叫皆可，不依賴值的檢查。`plan` 在 `check_inputs` 與 `check_targets` 都通過之後呼叫：`values` 只能由 `check_inputs` 產生（§4.5），目標沒有碰撞，狀態檔已成功載入。

**後置條件**：

- `targets`：每個啟用、在 manifest 中、有目前平台 `target` 且能展開的輸出一項。無法展開的不列入，由 `check_inputs` 回報。`key` 經由 `FileSystem::location_key` 取得；上層目錄是連結時為 `LinkInPath`，`check_targets` 回報 `preflight.linked-parent`，其他走訪錯誤回報 `preflight.target-path`。保留位置來自 `Layout::reserved`。
- 期望內容：範本以 `render` 渲染，原樣複製使用 `LoadedSource::Verbatim` 的位元組。`metadata().mode` 只在輸出宣告了權限，而且平台是 Linux 或 macOS 時有值（§5.10）。渲染錯誤記在 `diags`，該輸出不進入計畫，`plan` 回傳 `Err`。
- 觀測：每個啟用輸出的目標與每筆紀錄的目標各觀測一次（`inspect`）。`keep_current` 為 `false` 時不保留內容，只計算 SHA-256。
- 紀錄的比對：每筆紀錄的 `target` 轉成位置鍵（§6.2），與啟用輸出的位置鍵比較；`Same` 的紀錄是該輸出的紀錄，**不論名稱是否相同**。輸出改名而目標不變時，紀錄沿用並改成新名稱，不清除後重建。比對後，依 §5.11 的「紀錄位置確認」檢查父目錄的 `PersistentId`。位置鍵相同的紀錄超過一筆時，全部視為位置無法確認。
- 啟用的輸出依 §5.11 決定 `action`、`disposition` 與 `record`：
  - 有紀錄而且位置確認、目標是一般檔案時，依 §5.11 的五列。第 5 列的 drift 衝突依 `choices` 決定 `Resolve(Overwrite)`、`Resolve(Accept)` 或 `Retain(DriftConflict)`。
  - 沒有紀錄、位置無法確認，或目標不是一般檔案時，依 §5.11 的未受管理規則：一般檔案或連結，有接管授權為 `Takeover`，否則 `Retain(Unmanaged(類型))`。目錄與其他類型沒有接管授權時為 `Retain(Unmanaged(類型))`；有接管授權時，`check_plan` 回報 `preflight.unsupported-type`（04 §3.1）。
  - `create-only` 依 §5.11：目標存在時為 `Unchanged`、`Proceed`、`RecordChange::Keep`。
  - 比對到的紀錄帶有 `forget_pending` 時一律 `Retain(ForgetPending)`。
- 沒有被啟用輸出比對到的紀錄，各產生一個清除項目，依 §5.11 的清除表。觀測遇到 `LinkInPath` 時為 `Retain(LocationUnconfirmed)`；位於保留位置時為 `Retain(Reserved)`；已修改而 `choices` 指定刪除時為 `Resolve(Delete)`。
- 嚴格模式：drift 衝突一律為 `Retain(DriftConflict)`，不套用 `choices`（03 §12 第 1 項），`strict_blocked()` 為 `true`。
- `Selector::Path` 以位置鍵與項目比較。沒有比對到任何 drift 衝突（`drift`）或已修改的舊輸出（`delete_modified`）時，回報警告 `apply.choice-unmatched`。同一個項目同時被指定覆寫與接受時，回報錯誤 `apply.choice-conflict`；`All` 與 `Path` 都指定時，`Path` 優先。
- 順序：先列清除項目，再列啟用輸出，各自依 `target` 排序。清除在前，舊輸出刪除後空出的路徑才能給新輸出使用。新目標的某一層上層是本批要刪除的舊輸出時，該項目的觀測改為「從那一層起不存在」；上層是其他非目錄的項目（`blocker`）時，`check_plan` 回報 `preflight.unsupported-type`。
- `Planner` 只使用 `FileSystem` 的唯讀方法。

**錯誤**：`template.render`（§4.5）；`apply.choice-conflict`、`apply.choice-unmatched`（警告）；觀測時的讀取失敗為 `plan.inspect-failed`，附路徑與作業系統的錯誤。

**隱藏的設計決定**：紀錄與目標的比對方式；項目的順序；`Choices` 的優先順序；期望內容保存在記憶體中。

### 4.13 RecoveryManager

**責任**：保存回復快照、記錄本批的每一個操作，在失敗時依相反順序回復（§6.5、§6.6），並清理快照與暫存檔。也負責清理程序中止後殘留的批次目錄。

**介面**（`recovery`）：

```rust
pub struct RecoveryManager<'a> { /* fs、layout、sync、批次目錄、journal、已完成的操作 */ }

impl<'a> RecoveryManager<'a> {
    /// §6.6 步驟 1：選定批次編號、建立 txn/<批次編號>/，狀態檔存在時擷取它的快照。
    pub fn begin(fs: &'a dyn FileSystem, layout: &'a Layout, sync: &'a dyn SyncHook,
                 lock: &'a WriteLock, started_at: SystemTime, state: &LoadedState)
        -> Result<Self, ExecError>;
    pub fn batch(&self) -> &BatchId;
    /// 取得暫存檔名稱，先附加到 journal（§6.5）。
    pub fn temp_name(&mut self, at: &HomePath) -> Result<HomePath, ExecError>;
    /// 擷取快照（§6.5 第 1–3 步）。content 為 false 時不開檔，只記下觀測中的權限（只變更權限的項目）。
    pub fn capture(&mut self, expect: &Observation, content: bool) -> Result<SnapshotId, ExecError>;
    /// 記下一個已生效的操作，以及回復它需要的資料。
    pub fn record(&mut self, done: Done);
    pub fn rollback(self, diags: &mut Diagnostics) -> RecoveryReport;
    /// 批次成功：依 journal 清理暫存檔，刪除 txn/<批次編號>/（C4 的 commit）。
    pub fn finish(self, diags: &mut Diagnostics);

    /// 取得鎖定後、規劃前，清理殘留的批次目錄（§6.5）。由執行檔呼叫。
    pub fn clean_stale(fs: &dyn FileSystem, layout: &Layout, lock: &WriteLock, diags: &mut Diagnostics);
    /// 不取得鎖定的指令：只提示殘留的批次目錄，不刪除。
    pub fn report_stale(fs: &dyn FileSystem, layout: &Layout, diags: &mut Diagnostics);
}

pub enum Done {
    Created     { result: Observation },
    Replaced    { result: Observation, snapshot: SnapshotId, backup: Option<Observation> },
    LinkTakenOver { result: Observation, backup: Observation },
    Deleted     { parent: Observation, snapshot: SnapshotId },
    ModeChanged { result: Observation, snapshot: SnapshotId },
    DirCreated  { result: Observation },
    StateCommitted { result: Observation },
}

pub struct RecoveryReport {
    pub failures: Vec<RecoveryFailure>,     // 空的表示全部已回復（FR-APL-003）
    pub state: StateRecovery,               // Untouched、Restored、NotRestored（附原因）
}
pub struct RecoveryFailure {
    pub at: HomePath,
    pub reason: RecoveryFailureKind,        // Changed、SnapshotMismatch、Occupied、Fs(FsError)
    pub backup: Option<HomePath>,           // 接管過的目標無法回復時，備份的位置（§5.12）
}
```

`BatchId` 選定時，`txn/` 與 `backups/` 底下都不能已有同名的目錄，兩者因此使用同一個編號；`txn/<批次編號>/` 以不覆蓋的方式建立，`backups/<批次編號>/` 在第一次接管時建立（§5.12）。

**前置條件**：已取得鎖定。`begin` 只在計畫有要改動的目標，或需要提交狀態時呼叫（§4.14）；不改動任何東西的批次不建立 `txn/`。

**後置條件**：

- `begin` 依 §6.5 擷取狀態檔的快照，在 `BeforeCapture`、`AfterCapture`（`StateCapture`）到達同步點，核對副本與 `state.observation` 的版本相同（A25）。失敗時刪除剛建立的批次目錄，回傳錯誤，不改動任何目標（03 S5）。
- `capture` 以 `FsOp::Capture` 寫入 `txn/<批次編號>/<序號>`，比較 `CaptureInfo::sha256` 與 `expect` 的版本，再 `confirm`。三步都通過才回傳 `SnapshotId`；版本不符時刪除副本，回傳觀測失效（A16）。
- `rollback` 依 §6.6 的回復表，以與 `record` 相反的順序處理每一個 `Done`，最後處理狀態檔。到達 `BeforeRollback` 一次，每個操作在 `BeforeOperation`、`AfterOperation`（`Revert*`、`StateRollback`）到達同步點。使用快照前，以 `Source::Snapshot` 邊讀邊核對 SHA-256（A16）。一項失敗時記下原因，繼續下一項（04 §3.3）。回復成功的接管項目，刪除它的備份；無法回復的，保留備份並列在 `RecoveryFailure::backup`（§5.12）。
- `rollback` 與 `finish` 都依 §6.5 清理 journal 中的暫存檔與批次目錄，`rollback` 另外移除變空的 `backups/<批次編號>/`。清理失敗只以警告 `apply.cleanup` 回報，不改變結果（04 §3.3）。
- `clean_stale` 依每個殘留批次目錄中的 journal 清理（§6.5），以警告 `apply.stale-batch` 說明該批次沒有正常結束。狀態目錄與設定目錄最上層、名稱含 `.rosamta-tmp.` 的項目，是管理與初始化留下的暫存檔，一併刪除；來源目錄中的不刪除，因為來源目錄由使用者以 git 管理，這類檔案會出現在 `git status` 中。

**錯誤**：`ExecError`（§4.14）；`RecoveryFailureKind` 的每個值有固定的字串：`recovery.changed`（回復前的核對不符）、`recovery.snapshot-mismatch`、`recovery.occupied`（位置已被占用，A22）、`recovery.failed`（附作業系統的錯誤）。警告：`apply.cleanup`、`apply.stale-batch`。

**隱藏的設計決定**：快照的命名與保存位置；journal 的格式；回復的順序；備份在回復後的處理。

### 4.14 Executor

**責任**：依通過預檢與模式檢查的計畫，執行每個項目、製作接管備份、重檢並提交狀態，失敗時交給 `RecoveryManager` 整批回復（§6.6、03 S2、S5）。

**介面**（`exec`）：

```rust
/// 通過 check_plan 與模式檢查的計畫。建構函式是 pub(crate)，只有 ApplyService 建立（§11 核對）。
pub struct ApprovedPlan(ApplyPlan);

pub struct Executor<'a> {
    fs: &'a dyn FileSystem, layout: &'a Layout, sync: &'a dyn SyncHook, state: &'a StateStore<'a>,
}

impl<'a> Executor<'a> {
    pub fn new(fs: &'a dyn FileSystem, layout: &'a Layout, sync: &'a dyn SyncHook,
               state: &'a StateStore<'a>) -> Self;
    pub fn execute(&self, lock: &WriteLock, plan: ApprovedPlan, started_at: SystemTime,
                   diags: &mut Diagnostics) -> ApplyReport;
}

pub struct ApplyReport {
    pub batch: Option<BatchId>,
    pub items: Vec<ItemResult>,                 // 與計畫的項目一一對應，順序相同
    pub failure: Option<ExecError>,             // 原始的執行錯誤（FR-APL-003）
    pub recovery: Option<RecoveryReport>,       // 有回復時（FR-APL-003、FR-APL-004）
}
impl ApplyReport { pub fn outcome(&self) -> ApplyOutcome; }

pub enum ApplyOutcome { Completed, Partial, Failed } // B-5；exit code 的對應見 §9

pub struct ItemResult {
    pub output: OutputName,
    pub target: HomePath,
    pub action: Action,
    pub status: ItemStatus,
    pub backup: Option<HomePath>,               // 接管備份的位置（FR-MAN-019）
}
pub enum ItemStatus {
    Done, Retained(RetainReason),               // 正常結束（C3 的完成、保留）
    Failed,                                     // 發生執行錯誤的項目，原因在 ApplyReport::failure
    Reverted, NotReverted,                      // 已回復；無法回復，原因在 RecoveryReport
    NotRun,                                     // 錯誤發生時尚未處理
}

pub struct ExecError {
    pub item: Option<usize>,                    // 計畫中的項目；狀態檔為 None
    pub at: HomePath,
    pub kind: ExecErrorKind,
}
pub enum ExecErrorKind {
    Fs(FsError),                                // exec.write-failed：建立、取代、刪除、權限、快照或備份失敗
    Stale(StaleWhat),                           // exec.stale：觀測失效（§6.3）
    ContentMismatch,                            // exec.content-mismatch：寫入的內容不是期望內容
    StateChanged,                               // exec.state-changed：狀態檔不是首次載入或本批寫入的版本
}
```

C3 的 `ApplyReport.exitCode` 改為 `outcome()`：core 不決定 exit code，§9 把 `Completed`、`Partial`、`Failed` 對應到 `0`、`2`、`1`。C3 的 `errors` 與 `rollbackFailures` 分別是 `failure` 與 `recovery`；批次在第一個錯誤就停止（§6.6），所以原始錯誤只有一個。

**每個項目的操作**（§6.6 步驟 2）：

| 項目 | 操作的順序 |
| --- | --- |
| `Retain` | 無 |
| `Unchanged`，只更新或不更新紀錄（包括 `Resolve(Accept)`、直接納管、`create-only` 已存在） | 無；由步驟 3、5 核對 |
| `Delete`，舊目標已不存在 | 無；只移除紀錄 |
| `Create` | 不存在的上層目錄依序 `CreateDir`，再 `CreateFile`，權限見 §6.4 |
| `Update`，內容不同（`Proceed`、`Resolve(Overwrite)`） | `capture` → `ReplaceFile` |
| `Update`，只有權限不同 | `capture`（只記權限）→ `SetMode` |
| `Takeover`，一般檔案 | `capture` → `Backup` → `ReplaceFile`；內容相同、只有權限不同時，最後一步改為 `SetMode` |
| `Takeover`，連結 | `Backup`（hard link，§6.5）→ `ReplaceFile`；不另外擷取快照 |
| `Delete`（`Proceed`、`Resolve(Delete)`） | `capture` → `DeleteFile` |

每個操作的 `expect` 是項目的 `observation`；目錄由本批建立時，依 `CreateDir` 的結果推導（§4.10）。`Backup` 寫到 `backups/<批次編號>/<目標的相對路徑>`，需要的目錄以 `CreateDir`（`private`）建立（§5.12）。每個操作依 §4.10 的三次呼叫執行，成功後以 `RecoveryManager::record` 記下。

**前置條件**：已取得鎖定；`plan` 由 `ApplyService` 在 `check_plan` 與模式檢查都通過後建立，`strict_blocked()` 為 `false`。

**後置條件**：

- 依 §6.6 的六個步驟。計畫有要改動的目標或要提交狀態時才呼叫 `RecoveryManager::begin`；否則直接執行步驟 3 與 5。
- 新的狀態從 `plan.state()` 複製，依每個 `Done` 項目的 `RecordChange` 修改：`Remove` 刪除 `record_before`；`Put` 刪除 `record_before`，再加入新紀錄，名稱與目標是本項目的，`parent` 取自操作後（或規劃時，若沒有操作）父目錄的 `PersistentId`。保留的項目不改紀錄（FR-APL-009）。新狀態與載入時的位元組相同時不提交。
- 步驟 3、5 的核對以 `inspect` 進行：寫入的項目核對身分與 `written`；刪除的項目核對仍不存在，父目錄相同；其他項目核對身分、版本與宣告的權限仍與 `observation` 相同。狀態檔核對身分與版本。
- 任何一步失敗，停止處理後續項目，呼叫 `rollback`。失敗的項目為 `Failed`，已處理的項目依回復結果為 `Reverted` 或 `NotReverted`，未處理的為 `NotRun`，保留的項目仍為 `Retained`。drift 衝突與未授權的接管不是執行錯誤，不觸發回復（03 S5）。
- `outcome()`：有 `failure` 為 `Failed`；否則沒有保留項目為 `Completed`（包括空計畫，FR-APL-006）；有完成也有保留為 `Partial`（FR-APL-007）；只有保留為 `Failed`（FR-APL-008）。「不變」算完成。
- `ExecErrorKind` 的每個值有固定的字串，見上方的註解。

**隱藏的設計決定**：每個項目的操作順序；何時開始批次；新狀態的組合方式；完成與回復時各項目狀態的判定。

### 4.15 ApplyService

**責任**：預覽、摘要與套用的流程（03 S1）。依固定的順序呼叫載入、解析、預檢、規劃與執行，套用前做模式檢查。

**介面**（`apply`）：

```rust
pub struct ApplyService<'a> { fs: &'a dyn FileSystem, host: &'a HostContext, sync: &'a dyn SyncHook }

impl<'a> ApplyService<'a> {
    pub fn new(fs: &'a dyn FileSystem, host: &'a HostContext, sync: &'a dyn SyncHook) -> Self;
    /// 預覽與摘要（FR-QRY-005）。不取得鎖定，不寫入任何檔案。
    pub fn preview(&self, req: &ApplyRequest, diags: &mut Diagnostics) -> Checked<ApplyPlan>;
    pub fn apply(&self, lock: &WriteLock, req: &ApplyRequest, diags: &mut Diagnostics) -> Checked<ApplyReport>;
}

pub struct ApplyRequest {
    pub choices: Choices,          // §4.12：嚴格模式與指定的處理方式
    pub diff: bool,                // 預覽是否顯示內容差異；摘要與套用為 false
    pub started_at: SystemTime,    // 批次編號（§5.1、§8.1），由執行檔取得目前時間
}
```

C4 的 `run(request)` 分成 `preview` 與 `apply`：只有套用要求 `&WriteLock`，預覽因此無法寫入（§4.10）。`started_at` 由呼叫端傳入，core 不讀取系統時間，測試可以指定（§8.1）。

**流程**：兩個方法共用第 1–8 步：

1. `Layout::config_dir` → `ConfigLoader::load_local` → `Layout::new` → `ConfigLoader::load`。
2. `StateStore::load`。
3. `ValueResolver::resolve`；`Profile::enabled`。
4. 對 manifest 中每個啟用的輸出呼叫 `SourceLoader::load`。
5. `Planner::targets`。
6. `check_inputs` 與 `check_targets` 都執行（§4.6）。`config.is_complete()` 為 `false`、或任一階段失敗時，到此結束，回傳 `Err`。
7. `Planner::plan`，再 `check_plan`。
8. 模式檢查：`plan.strict_blocked()` 時，對每個 drift 衝突回報錯誤 `apply.strict-conflict`，附輸出名稱與目標路徑（03 §12 第 1 項）。

`preview` 在第 7 步後回傳計畫；第 8 步只在套用時阻擋，預覽照常回傳計畫，並在計畫中標出嚴格模式將不寫入任何輸出（FR-DRF-003）。`apply` 在第 8 步通過後到達 `AfterPreflight`，建立 `ApprovedPlan`，交給 `Executor::execute`，回傳它的回報。

**前置條件**：套用時已取得鎖定，執行檔已呼叫 `RecoveryManager::clean_stale`；預覽時執行檔已呼叫 `report_stale`。

**後置條件**：

- 預覽、摘要與套用使用同一個 `Planner` 與同一組 `Choices`，所以在 FR-PLN-006 的條件下，套用的項目與動作與預覽相同。
- 第 6 步的錯誤在同一次回報，USE-004 的三種錯誤因此一起列出（§4.6）。設定不完整時仍執行第 6 步，檢查讀到的部分。
- 預覽只使用 `FileSystem` 的唯讀方法，不取得鎖定，不建立任何目錄（FR-PLN-004）。
- 預檢或模式檢查失敗時，不建立 `ApprovedPlan`，不呼叫 `Executor`，不改動任何檔案（FR-APL-008、FR-DRF-009）。

**錯誤**：`apply.strict-conflict`；其他代碼由各步驟產生。執行錯誤不是診斷，記在 `ApplyReport`（§4.14）。

**隱藏的設計決定**：步驟的順序，以及哪一步失敗時停止。

### 4.16 InitializationService 與 GitClient

**責任**：準備來源目錄、建立或合併本機設定、記錄來源目錄的位置，並依選項執行第一次套用（03 S6、FR-INI-001–011）。`GitClient` 只負責把遠端 repo clone 到指定的目錄。

**介面**（`init`）：

```rust
pub trait GitClient {
    /// 把 url clone 到 dest。dest 是已存在的空目錄。不讀取標準輸入，不詢問使用者。
    fn clone_into(&self, url: &str, dest: &HomePath) -> Result<(), GitError>;
}
pub enum GitError {
    NotFound,                                       // 找不到 git 執行檔
    Failed { status: Option<i32>, stderr: String }, // git 的 exit status 與錯誤訊息
}

pub struct InitializationService<'a> {
    fs: &'a dyn FileSystem, host: &'a HostContext, sync: &'a dyn SyncHook,
    git: &'a dyn GitClient, apply: &'a ApplyService<'a>,
}

impl<'a> InitializationService<'a> {
    pub fn new(/* 同上的欄位 */) -> Self;
    pub fn initialize(&self, lock: &WriteLock, req: &InitializationRequest, diags: &mut Diagnostics)
        -> Checked<InitializationResult>;
}

pub struct InitializationRequest {
    pub repo: Option<String>,                 // 遠端 repo 的位址（IF-012）
    pub source: Option<PathBuf>,              // 絕對路徑（FR-INI-005）；相對路徑由執行檔轉換
    pub tags: Vec<Tag>,                       // FR-INI-006
    pub values: Vec<(FieldRef, RawValue)>,    // FR-INI-006；文字轉成 RawValue 的規則見 §9
    pub apply: Option<ApplyRequest>,          // None 表示只初始化（FR-INI-003）
}

pub struct InitializationResult {
    pub source: HomePath,
    pub source_created: bool,                 // 這次建立或取得來源目錄
    pub changes: Vec<FileChange>,             // 建立或修改的檔案：manifest、本機設定
    pub apply: Option<ApplyReport>,
}
```

C4 的 `GitClient.cloneToStaging(url)` 改成 `clone_into(url, dest)`：暫存目錄由 `InitializationService` 經由 `FileSystem` 建立，才能依 §6.3 記下身分並在 clone 前後核對；`GitClient` 只執行 git。

platform 的實作以 `std::process` 執行 `git clone -- <url> <dest>`：`--` 避免以 `-` 開頭的位址被當成選項；標準輸入接到空裝置，設定 `GIT_TERMINAL_PROMPT=0`，git 需要帳號密碼時直接失敗，不等待輸入（FR-INI-006）。SSH 使用使用者既有的金鑰與設定（AS-4）。

**流程**：

1. 依 `Layout::config_dir` 與 `load_local` 讀取既有的本機設定。來源目錄依序取 `req.source`、本機設定的 `source`、預設位置（§5.1），以 `Layout::with_source` 建立 `Layout`。
2. 依 §5.2 檢查：三個目錄都不等於 home，彼此的位置鍵沒有 `Same`、`Ancestor` 或 `Descendant` 的關係，路徑中沒有連結。
3. 準備來源目錄：
   - 指定了 `repo`：來源目錄的位置不得存在，包括空目錄，否則失敗，不改動任何檔案（FR-INI-009）；空目錄的提示是先刪除它。刪除該位置旁邊殘留的 `.<名稱>.rosamta-tmp.*` 暫存目錄：鎖定保證沒有其他 rosamta 正在使用它們。建立不存在的上層目錄，在同一個目錄以 `CreateDir` 建立暫存目錄，呼叫 `clone_into`，核對暫存目錄與上層仍是建立時的身分，再以 `MoveDir` 移到正式位置（§6.3），同步點為 `SourceMoveIn`。任一步失敗時以 `RemoveTree` 刪除暫存目錄，並刪除本次建立、仍是空的上層目錄，回報 repo 的位址與原因，不建立本機設定（FR-INI-010）。
   - 沒有指定 `repo`：來源目錄不存在時建立它；`manifest.toml` 不存在時以 `CreateFile` 建立空的 manifest（FR-INI-002）。既有的檔案不改動。
4. 合併本機設定（FR-INI-008）：以 `TomlEditor` 解析既有的 `local.toml`（不存在時從空文字開始），只修改這次指定的值：
   - `req.source` 有值時，寫入 `source`，以 `~/` 開頭、以 `/` 分隔。
   - `req.tags` 加到既有的 `tags` 之後，略過已有的標籤，不移除任何標籤。
   - `req.values` 的每一項寫入 `[values.<模組>]` 的 `<欄位>`，已有的值被取代。
   原本不存在時以 `CreateFile` 寫入，否則以 `ReplaceFile` 取代，被取代的必須是第 1 步讀到的版本（§6.3）。設定目錄不存在時以 `ensure_private_dir` 建立。同步點為 `LocalWrite`。文字與原文相同時不寫入。
5. 第 4 步失敗，而第 3 步取得了遠端來源時，把來源目錄移回暫存名稱並刪除，回到初始化之前的狀態，使用者可以直接重試；不這樣做，再次初始化會因 FR-INI-009 而失敗。
6. 指定了 `repo` 而且 `req.apply` 有值時，以 `ApplyService::apply` 執行第一次套用（FR-INI-001、FR-INI-011），回報放在 `apply`。沒有指定 `repo` 時不套用（04 §2）。

**前置條件**：已取得鎖定，執行檔已呼叫 `RecoveryManager::clean_stale`。

**後置條件**：

- 取得來源失敗時，來源目錄與本機設定都不存在或維持原狀，沒有寫入任何輸出（FR-INI-010）。
- 第一次套用沿用一般套用的預檢、drift 與接管的處理，回報與 exit code 都與一般套用相同（FR-INI-011）。套用失敗不撤銷已完成的初始化（03 S6）。
- `tags` 以加入的方式合併：FR-INI-008 不允許改變這次沒有指定的值，既有的標籤屬於其中。移除標籤要編輯本機設定。
- 指定的欄位值只檢查名稱的格式；型別與欄位是否存在，由第一次套用或之後的預覽檢查（§4.6 對所有層級檢查）。
- 所有寫入都經由 `FileSystem` 的綁定操作；git 以路徑寫入暫存目錄，這是唯一無法綁定的寫入（§6.3）。

**錯誤**：

| 代碼 | 情況 |
| --- | --- |
| `init.source-exists` | 指定遠端時，來源目錄的位置已存在（FR-INI-009） |
| `init.layout` | 三個目錄互相包含、等於 home，或路徑中有連結（§5.2） |
| `init.clone-failed` | git 回報失敗，附 repo 的位址、git 的 exit status 與錯誤訊息（FR-INI-010） |
| `init.git-missing` | 找不到 `git` 執行檔 |
| `init.stale` | clone 後暫存目錄或上層不是建立時的身分，或移入時正式位置已被占用（A7） |
| `init.write-failed` | 建立來源目錄、manifest 或本機設定失敗，附作業系統的錯誤 |
| `init.concurrent-edit` | 本機設定在初始化期間被修改 |
| `docedit.mixed-line-endings`、`config.syntax` | 既有的本機設定無法編輯（§4.7） |

**隱藏的設計決定**：暫存目錄的命名與清理；來源目錄移入的方式；本機設定的合併規則；git 的呼叫方式與環境變數。

## 5. 資料與檔案格式

本節定義 rosamta 讀寫的檔案：程式使用的位置、四種使用者設定檔、範本的中繼資料、狀態檔與比較基準、接管備份，以及停止管理的表示。檔案格式依 §2.2 採用 TOML，狀態檔例外（§5.10）。

### 5.1 程式使用的位置

| 位置 | 預設路徑 | 可改變位置的設定 | 內容 |
| --- | --- | --- | --- |
| 來源目錄 | `~/.local/share/rosamta` | 本機設定的 `source`；未設定時依 `XDG_DATA_HOME` | manifest、Profile、使用者模組、範本與原樣複製的來源檔（§5.4） |
| 設定目錄 | `~/.config/rosamta` | `XDG_CONFIG_HOME` | 本機設定 `local.toml`（§5.8） |
| 狀態目錄 | `~/.local/state/rosamta` | `XDG_STATE_HOME` | 狀態檔 `state.json`（§5.10）、鎖定檔 `lock`（§6）、接管備份 `backups/`（§5.12）、回復快照 `txn/`（§6） |

- 三個平台使用同一套規則，`~` 是使用者的 home（取得方式見 §6）。如此 IF-005 的說明只需一份。Windows 不採用 `AppData\Roaming`：在漫遊設定檔的環境中，它會同步到其他主機，而本機設定與狀態檔只屬於一台主機（SRS §3.4.3）。
- 設定了 `XDG_DATA_HOME`、`XDG_CONFIG_HOME` 或 `XDG_STATE_HOME` 時，位置改為「該變數的值／`rosamta`」。值必須是 home 內的絕對路徑，否則除了顯示說明以外的子指令都回報該變數與值，回傳 `1`（DC-002）。
- 本機設定的 `source` 記錄來源目錄的位置，後續指令從這裡取得（FR-INI-004）。初始化時指定了來源目錄（FR-INI-005），就寫入這個值；沒有指定時不寫，使用預設位置。
- 設定目錄、狀態目錄以及 `backups/`、`txn/` 底下的目錄，在第一次需要時建立。Linux 與 macOS 的權限為 `0700`；Windows 的存取控制見 §6。

暫存檔放在被替換檔案的同一個目錄（ASM-01、ASM-02），名稱為 `.<原檔名>.rosamta-tmp.<程序編號>.<序號>`，與 pilot 相同。狀態檔的暫存檔也依此命名，放在狀態目錄。

每次套用以批次開始時的 UTC 時間產生批次編號，格式為 `YYYYMMDDTHHMMSSZ`。接管備份與回復快照的目錄都以批次編號命名。建立目錄時要求該名稱尚不存在；已存在時依序加上 `-2`、`-3`，直到建立成功。

### 5.2 保留位置

04 §3.1 要求預檢保護程式自己使用的位置。保留位置如下：

| 保留位置 | 範圍 |
| --- | --- |
| 來源目錄 | 整個目錄樹 |
| 設定目錄 | 整個目錄樹，包含本機設定 |
| 狀態目錄 | 整個目錄樹，包含狀態檔、鎖定檔、接管備份與回復快照 |
| 暫存檔 | 任何目錄中，檔名含 `.rosamta-tmp.` 的檔案 |

啟用的輸出符合下列任一情況時，預檢失敗，指出輸出名稱、它在 manifest 的位置與衝突的保留位置，回傳 `1`。動作為「不變」或目標尚不存在時同樣適用（A11）：

- 目標路徑等於某個保留目錄，或位於保留目錄之內
- 目標路徑是某個保留目錄的上層，也就是同一路徑同時被當成檔案與目錄（04 §3.1）
- 目標的檔名含 `.rosamta-tmp.`

清除計畫不得刪除保留位置內的檔案；狀態檔的舊紀錄指向保留位置時，該項目保留並回報原因。初始化時，來源目錄、設定目錄與狀態目錄也不得互相包含。路徑是否相同或互相包含，依 §6 的路徑比較判定，包含大小寫與別名。

### 5.3 設定檔的共通規則

manifest、Profile、使用者模組與本機設定都是 TOML 檔，遵守以下規則。

讀取：

- 檔案須是 UTF-8。開頭的 UTF-8 BOM 會被略過；其他無法解碼的位元組，回報檔案路徑與所在的行。
- 解析前把 CRLF 轉成 LF。來源目錄在 Windows 上可能被 git 轉成 CRLF；轉換後，多行字串的值在三個平台都相同（§2.2 探查結果的最後一列）。換行數目不變，診斷的行號仍對應原檔。
- 不認得的鍵一律是錯誤，回報檔案、行號與鍵名，並列出該位置可用的鍵（FR-MAN-029）。四種檔案都適用，打錯的鍵才不會被默默忽略。
- 名稱的格式：輸出名稱為 `[A-Za-z0-9][A-Za-z0-9_-]*`；模組與欄位名稱為 `[A-Za-z_][A-Za-z0-9_]*`，因為範本以 `theme.foreground` 這種屬性語法引用它們；標籤為 `[A-Za-z0-9_-]+`。名稱區分大小寫。

指令修改這四種檔案時（FR-MAN-030），沿用 §2.2 的規則。§2.2 留下的兩項，在下列第 3、4 點決定：

- 修改值時沿用原值的 decor（前後空白與行尾註解）。修改 inline table 中的鍵時，維持 inline table 的寫法。
- 整份使用 CRLF 的檔案，先轉成 LF 編輯，完成後轉回 CRLF。檔案開頭有 BOM 時保留。
- **混用 LF 與 CRLF 的檔案拒絕編輯**，回報檔案路徑與第一個換行方式不同的行，回傳 `1`。無論統一成哪一種，都會改到與這次修改無關的行，違反 FR-MAN-030；讓使用者先統一換行，結果才可預期。讀取不受影響。
- **在既有表格中新增鍵時，沿用同一表格最後一個鍵的縮排**（decor 的前綴）；表格是空的時不縮排。toml_edit 本身不會沿用（§2.2）。
- 新增表格（新的輸出或內建輸出的覆寫）加在檔案結尾，與前文隔一個空行，以 `key = value` 的標準寫法輸出。
- 移除表格時，緊接在上方的註解視為該宣告的一部分，一起移除（§2.2）。

### 5.4 來源目錄與 manifest

來源目錄的結構：

```
<來源目錄>/
├── manifest.toml   輸出的宣告
├── profile.toml    Profile（§5.7）
├── modules/        使用者模組（§5.5），底下任意層的 *.toml
├── templates/      慣例位置，不是必要的
└── files/          納入管理的檔案（FR-MAN-023）的預設位置
```

只有 `manifest.toml`、`profile.toml` 與 `modules/` 的名稱有固定意義。範本與原樣複製來源的位置由 manifest 指定，檔名不影響任何屬性（FR-MAN-006）。`manifest.toml` 不存在時，回報來源目錄的路徑，回傳 `1`。`profile.toml` 不存在時視為沒有條件，所有輸出都不啟用。`modules/` 不存在時視為沒有使用者模組。

納入管理（FR-MAN-023）預設把檔案複製到 `files/` 底下，沿用目標在 home 內的相對路徑，例如 `~/.config/nvim/init.lua` 複製到 `files/.config/nvim/init.lua`。預定位置已被占用時，在副檔名前依序加上 `-2`、`-3`，例如 `init-2.lua`、`.gitconfig-2`，加到 `-99` 仍被占用就回報失敗（04 §2）。

manifest 有兩種表格，檔案的頂層不能有其他鍵：

- `[outputs.<名稱>]`：使用者宣告的輸出
- `[builtin.<名稱>]`：內建輸出的覆寫（FR-MAN-011）

覆寫與新輸出分成兩種表格，拼錯內建輸出的名稱時，回報的是「沒有這個內建輸出」並列出相近的名稱，而不是被當成一個缺少屬性的新輸出。`[outputs.*]` 的名稱與內建輸出相同時，預檢失敗，指出應改用 `[builtin.*]`。

| 屬性 | TOML 型別 | 未寫時 | 說明 | 需求 |
| --- | --- | --- | --- | --- |
| `template` | 字串 | — | 範本的路徑，相對於來源目錄，以 `/` 分隔。以 `builtin:` 開頭時指內建範本 | FR-MAN-001、FR-TPL-003 |
| `copy` | 字串 | — | 原樣複製的來源檔路徑，規則同 `template` | FR-TPL-006 |
| `target` | 字串或表格 | — | 字串表示三個平台相同；表格的鍵為 `linux`、`macos`、`windows`，只寫需要的平台 | FR-MAN-007–010 |
| `ownership` | 字串 | `"full"` | `"full"`（完整管理）或 `"create-only"`（只在不存在時建立） | FR-MAN-012、FR-MAN-013 |
| `permissions` | 字串 | 不宣告 | 八進位的權限位元，`"000"`–`"777"`，可加前導 `0`，例如 `"0600"`。不接受 setuid 等特殊位元。Windows 上不套用 | FR-MAN-015、FR-MAN-016 |
| `takeover` | 布林 | `false` | 是否接管目標路徑上未受管理的檔案 | FR-MAN-018–020 |
| `removed` | 布林 | `false` | 只用於 `[builtin.*]`，見 §5.13 | FR-MAN-025 |

`[outputs.*]` 必須有 `target`，以及 `template`、`copy` 兩者之一。延後的 FR-MAN-014、FR-MAN-017 沒有對應的屬性，寫了會被當成不認得的鍵。`permissions` 以字串表示，因為 TOML 的整數不允許前導零，使用者寫 `0600` 會是語法錯誤，寫 `0o600` 又不是一般的寫法。

`target` 的寫法（展開、正規化與 home 邊界的檢查見 §6）：

- 以 `~` 開頭表示 home；`${NAME}` 引用環境變數 `NAME`（FR-MAN-009）；`$$` 表示字面的 `$`。
- 路徑以 `/` 分隔，manifest 中的文字不得含 `\`；環境變數的值可以含 `\`，例如 Windows 的 `APPDATA`。
- 展開後不是絕對路徑、引用了未定義的環境變數，或含有無法以 UTF-8 表示的字元時，預檢失敗。

`[builtin.*]` 只寫要改的屬性，其餘沿用內建宣告（FR-MAN-011）。合併以屬性為單位，`target` 例外：表格形式依平台合併，只寫 `target.linux` 時，其他平台沿用內建路徑；字串形式則取代三個平台。覆寫 `template` 或 `copy` 時，取代內建輸出的來源。

```toml
# 原樣複製，三個平台相同的目標路徑
[outputs.gitconfig]
copy = "files/.gitconfig"
target = "~/.gitconfig"

[outputs.nvim]
copy = "files/.config/nvim/init.lua"
target.linux = "~/.config/nvim/init.lua"
target.macos = "~/.config/nvim/init.lua"
target.windows = "${LOCALAPPDATA}/nvim/init.lua"
takeover = true             # 第一次套用時接管既有的設定檔

[outputs.ssh-config]
template = "templates/ssh_config.j2"
target = "~/.ssh/config"
permissions = "0600"

# 只改內建輸出在 Linux 的目標路徑與範本（FR-TPL-003）
[builtin.ghostty]
target.linux = "~/.config/ghostty/rosamta"
template = "templates/ghostty.j2"
```

### 5.5 使用者模組與欄位宣告

`modules/` 底下的檔案有兩種表格，可以寫在同一個檔案，也可以分開；檔名與檔案的讀取順序不影響結果（FR-PRF-011）：

- `[modules.<模組>]`：宣告使用者模組與它的欄位（FR-MOD-001、FR-MOD-004）
- `[values.<模組>]`：設定欄位值，屬於覆寫層級 2「使用者模組」。內建模組與使用者模組的值都寫在這裡

```toml
# modules/colors.toml
[values.theme]
foreground = "#c0caf5"
background = "#1a1b26"

[modules.prompt]
version = 1

[modules.prompt.fields.width]
type = "integer"
required = true
min = 1
max = 72
description = "提示列的最大寬度"

[modules.prompt.fields.style]
type = "enum"
options = ["plain", "powerline"]

[values.prompt]
width = 40
```

`[modules.<模組>]` 的 `version` 是模組格式版本，供範本宣告相容版本（FR-TPL-004、§5.9）；未寫時沒有版本，宣告了相容版本的範本引用這個模組時，預檢失敗。模組名稱不得與內建模組相同，也不得與範本的全域名稱相同（`host` 與 §4.5 註冊的函式）。

欄位宣告的鍵：

| 鍵 | 適用的型別 | 說明 |
| --- | --- | --- |
| `type` | 全部 | `"string"`、`"integer"`、`"decimal"`、`"boolean"`、`"color"`、`"enum"`、`"list"`、`"map"`，對應 SRS §3.4.1 的八種型別。必填 |
| `required` | 全部 | 是否必填，預設 `false` |
| `description` | 全部 | 說明文字，供 IF-006 顯示 |
| `min`、`max` | `integer`、`decimal` | 允許的最小值、最大值（含） |
| `options` | `enum` | 選項清單，不得為空、不得重複。必填 |
| `items` | `list` | 元素的宣告，寫成 inline table，可用的鍵與欄位宣告相同，但不含 `required`、`description`。必填 |
| `min_items`、`max_items` | `list` | 元素個數的下限、上限（含） |
| `keys` | `map` | 名稱的格式：`"identifier"`（`[A-Za-z_][A-Za-z0-9_]*`）或 `"word"`（不含空白與控制字元的非空字串），預設 `"word"` |
| `values` | `map` | 值的宣告，寫法同 `items`。必填 |
| `default` | 全部 | 只用於內建模組的宣告（§5.6）；使用者模組的值寫在 `[values.*]` |

名稱的格式只提供這兩種，不開放正規表示式：格式本身也是設定檔相容性的一部分（MNT-002），固定的格式較容易在同一個 major 版本內維持不變。

欄位值的 TOML 表示：

| 型別 | TOML 表示 | 檢查與正規化 |
| --- | --- | --- |
| `string` | 字串 | 不得為空 |
| `integer` | 整數 | 64 位元有號整數 |
| `decimal` | 浮點數或整數 | 整數視為同值的小數；不接受 `nan`、`inf` |
| `boolean` | 布林 | — |
| `color` | 字串 | `#` 加 6 位十六進位數字，不分大小寫（DATA-001）。檢查後轉成小寫再交給範本，`#FF8000` 與 `#ff8000` 因此產生相同的輸出 |
| `enum` | 字串 | 必須是 `options` 之一 |
| `list` | 陣列 | 每個元素依 `items` 檢查 |
| `map` | 表格或 inline table | 名稱依 `keys` 檢查，值依 `values` 檢查 |

同一層級的重複：同一個模組只能在一個檔案中宣告；同一個欄位值只能由一個檔案設定。違反時預檢失敗，列出每一處的檔案與行號（FR-PRF-012）。不同檔案設定同一個模組的不同欄位不算重複。`[values.*]` 引用未宣告的模組或欄位時，預檢失敗並列出名稱相近的欄位。

選填欄位在四個層級都沒有值時，以 `none` 傳入範本（§2.3）。

### 5.6 內建模組、輸出與範本

內建模組以與 §5.5 相同的格式宣告，加上 `default`，編進執行檔（DC-004）。四個模組的格式版本都是 `1`。

| 模組 | 欄位 | 型別 | 必填 | 範圍或格式 | 預設值 | SRS §3.4.2 的設定 |
| --- | --- | --- | --- | --- | --- | --- |
| `theme` | `foreground` | color | 是 | — | `#e5e5e5` | 前景色 |
| `theme` | `background` | color | 是 | — | `#000000` | 背景色 |
| `theme` | `cursor` | color | 否 | — | — | 游標顏色 |
| `theme` | `ansi` | list，元素為 color | 是 | 剛好 16 個 | xterm 的預設 16 色（下方） | ANSI 16 色 |
| `theme` | `opacity` | decimal | 否 | 0–1 | — | 背景透明度 |
| `font` | `family` | string | 否 | — | — | 字型名稱 |
| `font` | `size` | decimal | 否 | 1–200 | — | 字級 |
| `shellrc` | `env` | map，名稱為 `identifier`，值為 string | 否 | — | — | 環境變數 |
| `shellrc` | `aliases` | map，名稱為 `word`，值為 string | 否 | — | — | alias |
| `shellrc` | `path` | list，元素為 string | 否 | — | — | 加入 `PATH` 的目錄 |
| `terminal` | `cursor_shape` | enum | 否 | `block`、`beam`、`underline` | — | 游標形狀 |
| `terminal` | `scrollback` | integer | 否 | 0–100000 | — | 捲動行數 |
| `terminal` | `keybindings` | map，名稱為 `word`，值為 enum | 否 | 名稱是按鍵組合，例如 `ctrl+shift+c`；值為 `copy`、`paste`、`new_tab`、`close_tab`、`next_tab`、`previous_tab`、`increase_font_size`、`decrease_font_size`、`reset_font_size` | — | 快捷鍵 |

`ansi` 的預設值依序為 `#000000`、`#cd0000`、`#00cd00`、`#cdcd00`、`#0000ee`、`#cd00cd`、`#00cdcd`、`#e5e5e5`、`#7f7f7f`、`#ff0000`、`#00ff00`、`#ffff00`、`#5c5cff`、`#ff00ff`、`#00ffff`、`#ffffff`；`foreground` 與 `background` 取其中第 7 與第 0 色。

必填欄位都有預設值（DATA-003）。沒有預設值的欄位都是選填，未設定時內建範本不輸出對應的設定，讓程式使用自己的預設值（DATA-004）。字型名稱與字級沒有跨平台都適用的預設值，因此是選填。快捷鍵的值是不分程式的動作名稱，由各內建範本轉成目標程式的寫法，同一份設定才能套用到不同的終端機。

**內建輸出與內建範本**：每個內建範本對應一個同名的內建輸出，以 `builtin:<名稱>` 引用（§5.4）。內建輸出不會自動啟用，要在 Profile 的 `enable` 列出（FR-PRF-006）。

| 內建輸出與範本 | 程式 | `program` | `format_version` | 目標路徑 | 引用的模組 |
| --- | --- | --- | --- | --- | --- |
| `ghostty` | Ghostty | `ghostty` | `1.2` | Linux、macOS：`~/.config/ghostty/config` | `theme`、`font`、`terminal` |
| `alacritty` | Alacritty | `alacritty` | `0.15` | Linux、macOS：`~/.config/alacritty/alacritty.toml`；Windows：`${APPDATA}/alacritty/alacritty.toml` | `theme`、`font`、`terminal` |
| `posix-shell` | POSIX shell（bash、zsh 等） | `sh` | `POSIX.1-2017` | Linux、macOS：`~/.config/shell/rosamta.sh` | `shellrc` |
| `powershell` | PowerShell | `powershell` | `5.1` | Windows：`~/Documents/PowerShell/rosamta.ps1` | `shellrc` |

- 兩個終端機涵蓋三個平台：Ghostty 沒有 Windows 版，Alacritty 三個平台都有。shell 環境依平台分成 POSIX shell 與 PowerShell。
- 終端機的設定檔直接寫到程式讀取的位置，與 SRS 3.2.8 的例子相同；已有設定檔的使用者可宣告 `takeover`，或以 `[builtin.*]` 改變目標路徑。shell 的輸出則寫到獨立的檔案，由使用者在自己的啟動檔載入，例如在 `~/.bashrc` 加上 `. ~/.config/shell/rosamta.sh`：`~/.bashrc` 與 PowerShell 的 `$PROFILE` 通常還有使用者自己的內容，完整管理會覆寫它們。
- `format_version` 是撰寫範本時依據的版本。`powershell` 只使用 Windows PowerShell 5.1 也支援的語法，PowerShell 7 同樣能載入。實作開始時若各程式已有新的穩定版，依新版撰寫範本並更新本表。
- 每個欄位至少由一個內建範本使用（DATA-002）。`terminal.scrollback` 只有 Alacritty 的範本使用：Ghostty 的捲動上限以位元組計，無法由行數換算。上限 100000 是 Alacritty 接受的最大值。單元測試逐一修改每個欄位的值，確認至少一個內建輸出的內容改變，並逐一確認每個內建範本都有 `program` 與 `format_version`（FR-TPL-002）。

### 5.7 Profile

`profile.toml` 以 `[[rules]]` 列出條件：

| 鍵 | TOML 型別 | 說明 | 需求 |
| --- | --- | --- | --- |
| `os` | 字串 | `"linux"`、`"macos"` 或 `"windows"` | FR-PRF-001 |
| `hostname` | 字串 | 主機名稱，比較時不分 ASCII 大小寫。Windows 的主機名稱常以大寫顯示 | FR-PRF-002 |
| `tags` | 字串陣列 | 主機必須具備的標籤，全部都要有 | FR-PRF-003 |
| `enable` | 字串陣列 | 條件成立時啟用的輸出名稱 | FR-PRF-005 |
| `values` | 表格 | `[rules.values.<模組>]`，條件成立時的欄位值，屬於覆寫層級 3 | FR-PRF-008 |

```toml
[[rules]]                   # 所有主機
enable = ["gitconfig", "nvim"]

[[rules]]
os = "linux"
tags = ["laptop"]
enable = ["ghostty"]

[[rules]]
tags = ["work"]
enable = ["ssh-config"]
[rules.values.font]
size = 13
```

`os`、`hostname`、`tags` 都沒寫的條件對所有主機成立。同一條件內的項目全部符合才成立；一個輸出被多個條件啟用時，任一條件成立就啟用（FR-PRF-007）。條件的列出順序不影響結果：兩個成立的條件設定同一個欄位時，預檢失敗並列出兩處（FR-PRF-012），不由順序決定勝負。不成立的條件中的 `values` 不參與比較。

`enable` 列出不存在的輸出名稱時，只在標準錯誤顯示警告，不阻擋套用。停止管理（FR-MAN-025）只修改 manifest，Profile 中留下的名稱因此不會讓下次套用失敗。

### 5.8 本機設定

`local.toml` 放在設定目錄（§5.1），不在來源目錄（DATA-005）：

| 鍵 | TOML 型別 | 說明 | 需求 |
| --- | --- | --- | --- |
| `source` | 字串 | 來源目錄的位置，可用 `~`，必須在 home 內；未寫時使用預設位置 | FR-INI-004、FR-INI-005 |
| `tags` | 字串陣列 | 這台主機的標籤 | FR-PRF-004 |
| `values` | 表格 | `[values.<模組>]`，覆寫層級 4 的欄位值 | FR-PRF-009 |

```toml
source = "~/dotfiles"
tags = ["work", "laptop"]

[values.font]
size = 14
```

本機設定不存在時，視為沒有標籤、沒有覆寫，來源目錄使用預設位置。初始化時合併本機設定的方式見 §4（FR-INI-008），編輯規則依 §5.3。

### 5.9 範本的中繼資料

範本可以在檔案開頭以 MiniJinja 的註解寫中繼資料，內容是 TOML：

```
{#- rosamta
program = "ghostty"
format_version = "1.1"
[requires]
theme = 1
font = [1, 2]
-#}
background = {{ theme.background }}
```

| 鍵 | 說明 | 需求 |
| --- | --- | --- |
| `program` | 對應的程式名稱 | FR-TPL-002 |
| `format_version` | 該程式設定格式的版本，字串 | FR-TPL-002 |
| `requires` | 相容的模組格式版本，每個模組寫一個整數或整數陣列 | FR-TPL-004 |

只有檔案的開頭恰好是 `{#- rosamta` 加換行時，這段註解才被當成中繼資料。內建範本必須有 `program` 與 `format_version`，以單元測試逐一檢查；使用者範本可以省略整段。預檢比較 `requires` 與模組的格式版本：模組的版本不在宣告之列，或模組沒有宣告版本，預檢失敗，列出範本路徑、模組名稱與雙方的版本（FR-TPL-005）。

中繼資料寫在範本裡，而不是 manifest：相容的版本取決於範本引用了哪些欄位，是範本內容的性質；寫在範本裡，範本被複製到其他來源目錄或覆寫內建範本時，宣告會一起帶過去。

這個寫法不需要另外剝除：MiniJinja 渲染時把註解當成空白，`-#}` 去掉註解後的換行，錯誤訊息的行號仍是檔案的實際行號，中繼資料也不會被列為欄位引用。pilot P5 以 MiniJinja 3.1.0 驗證了這幾點。`-#}` 也會去掉中繼資料之後的空行，範本的內容若須以空行開頭，改用 `{{ "\n" }}` 輸出。

### 5.10 狀態檔

狀態檔是狀態目錄中的 `state.json`。狀態檔只由 rosamta 讀寫，不需要保留註解與格式，所以不用 TOML。JSON 的讀寫與 IF-004 的結構化輸出共用 serde_json，不需要另一個套件；500 筆紀錄約 150 KB，解析時間遠小於 PERF-001 的 1 秒。

```json
{
  "format": 1,
  "outputs": [
    {
      "name": "ssh-config",
      "target": ".ssh/config",
      "parent": { "file_id": "00000000000002f1" },
      "observed": { "sha256": "9f86d081…", "mode": "0600" },
      "desired": { "sha256": "9f86d081…", "mode": "0600" }
    }
  ]
}
```

| 欄位 | 說明 |
| --- | --- |
| `format` | 狀態檔的格式版本，第一版為 `1` |
| `outputs[].name` | 輸出名稱（DATA-006） |
| `outputs[].target` | 目標路徑，以 home 內的相對路徑表示，以 `/` 分隔（DATA-006）。不含 home 本身，使用者的 home 改名後紀錄仍有效 |
| `outputs[].parent` | 寫入或確認紀錄時，目標父目錄的檔案身分：`file_id` 為十六進位的檔案編號，Windows 另有十六進位的 `volume`。用來確認父目錄在兩次套用之間沒有被替換（04 §3.1、§5.11） |
| `outputs[].observed` | 實際基準：上次記錄時目標的比較資料 |
| `outputs[].desired` | 期望基準：上次記錄時期望內容的比較資料 |
| `outputs[].forget_pending` | 只在停止管理尚未完成時出現，值為 `true`（§5.13） |

比較資料（03 的 `ComparisonMetadata`）有兩個欄位：

- `sha256`：內容的 SHA-256，十六進位小寫。原樣複製的期望內容是來源檔的位元組；範本的期望內容是渲染結果的 UTF-8 位元組。
- `mode`：權限位元，寫法同 manifest 的 `permissions`。只在輸出宣告了權限，而且平台是 Linux 或 macOS 時記錄；否則為 `null`。未宣告的權限因此不會參與 drift 判定（FR-MAN-016）。

紀錄以 `target` 識別，同一個目標路徑只有一筆紀錄（依 §6 的路徑比較）。同一個輸出名稱可以有兩筆紀錄，例如輸出改了目標路徑，而舊路徑上被修改過的檔案仍保留（FR-DRF-013）。寫入時依 `target` 排序，內容相同的狀態就是相同的位元組。

`parent` 不記錄 Linux 與 macOS 的 `st_dev`：裝置編號由核心在掛載時配給，例如 btrfs 的 subvolume 使用匿名裝置編號，重新開機後可能改變（ASM-15）。

SEC-001：狀態檔只有名稱、路徑、檔案編號、權限位元與 SHA-256，沒有輸出的內容。SHA-256 無法還原內容，但很短而且可猜測的內容，例如只有四位數字的檔案，可以用窮舉比對出來。輸出中的機密由使用者保護（AS-7），狀態目錄的權限為 `0700`。

載入時（REL-003、REL-004）：

- 檔案不存在：視為空紀錄。
- JSON 語法錯誤、缺少欄位、有不認得的欄位、`target` 重複、`sha256` 或 `file_id` 不是合法的十六進位，都視為損毀：預檢失敗，指出狀態檔路徑，不覆寫它。
- `format` 大於 `1`：同樣阻擋，但訊息說明狀態檔由較新版本的 rosamta 寫入，而不是說檔案損毀。

### 5.11 比較基準與判定

本節是 03 C3 兩份基準的演算法，以 03 §12 第 3 項的三組案例驗證。以下 C 表示目標現況的比較資料，N 表示本次的期望，O、D 分別是紀錄中的實際基準與期望基準。C 是否含權限，依本次輸出的宣告決定（§5.10）。

- **C 符合 N**：`sha256` 相同；N 有 `mode` 時，目前的權限也與它相同。
- **外部修改**：C 與 O 的 `sha256` 不同，或 O 有 `mode` 而目前的權限與它不同。O 沒有 `mode` 時不比較權限，宣告權限之前的紀錄因此不會被當成 drift。
- **來源改變**：N 與 D 的 `sha256` 或 `mode` 不同。

**紀錄位置確認**：紀錄的 `target` 與本次目標路徑相同，而且目前父目錄的檔案身分等於 `parent`。父目錄不存在時，目標也不存在。位置無法確認時，啟用的輸出依「沒有紀錄」處理，清除則保留。只有一般檔案才比較內容；目標是符號連結或其他類型時，依 04 §3.1 的表格與 §6 處理。

完整管理、啟用、有紀錄且位置確認的輸出，依序判斷：

| 列 | 條件 | 動作 | 處理 | 套用後的紀錄 |
| --- | --- | --- | --- | --- |
| 1 | 目標不存在 | 建立 | — | O = D = N |
| 2 | C 符合 N | 不變 | O ≠ C 或 D ≠ N 時只更新紀錄，項目為完成（FR-DRF-002） | O = C，D = N |
| 3 | 沒有外部修改，來源沒有改變 | 不變 | —（曾接受 drift 時，目標維持接受的內容） | 不變 |
| 4 | 沒有外部修改，來源改變 | 更新 | 內容不同才重寫；權限不同才變更（FR-PLN-002） | O = D = N |
| 5 | 有外部修改 | drift 衝突 | 略過、覆寫或接受（FR-DRF-004–006）；嚴格模式下整批不寫入（FR-DRF-009） | 略過：不變；覆寫：O = D = N；接受：O = C，D = N |

沒有紀錄、或紀錄位置無法確認的一般檔案：C 符合 N 時直接納管，O = C，D = N（FR-MAN-021）；不符合時，有接管授權就備份後寫入，O = D = N，否則保留（FR-MAN-019、FR-MAN-020）。C 的內容相同但權限不同時不算符合，要變更權限也須接管授權，才不會在未授權的情況下改動未受管理的檔案。

只在不存在時建立（FR-MAN-013）：目標不存在時建立，O = D = N；目標存在時為「不變」，不比較內容，不更新紀錄，不列為 drift。沒有紀錄而目標已存在時，也不建立紀錄。

清除（輸出被移除、停用或改了目標路徑）：

| 舊目標的狀況 | 處理 |
| --- | --- |
| 不存在 | 只移除紀錄，項目為完成（FR-DRF-015） |
| 不是一般檔案、位置無法確認，或位於保留位置 | 保留檔案與紀錄，回報原因 |
| 沒有外部修改 | 刪除，並移除紀錄（FR-DRF-012） |
| 有外部修改 | 保留檔案與紀錄（FR-DRF-013）；指定刪除時才刪除（FR-DRF-014） |

紀錄帶有 `forget_pending` 時，無論輸出是否仍啟用，項目一律保留，原因是停止管理尚未完成（§5.13）。

三組案例。目標原為 A，紀錄 O = D = A，接著被外部改成 B：

| 案例 | 步驟 | C | N | 判定前的 O、D | 判定 | 結果 |
| --- | --- | --- | --- | --- | --- | --- |
| 接受後重跑不變 | 指定接受並套用 | B | A | A、A | 第 5 列，接受 | 目標仍為 B；O = B，D = A |
| | 不做任何修改再套用 | B | A | B、A | 第 3 列 | 不變，不列為衝突（FR-DRF-006） |
| 來源改後一般更新 | 接受後，模組改變，期望變成 E | B | E | B、A | 第 4 列 | 更新為 E，預覽的差異是 B 到 E；O = D = E（FR-DRF-008） |
| | 同上，但新期望恰好是 B | B | B | B、A | 第 2 列 | 不重寫，只把 D 更新為 B（04 §3.2） |
| 目標再改後偵測 drift | 接受後，目標又被改成 F | F | A | B、A | 第 5 列 | drift 衝突 |
| | 同上，但改成的內容恰好是 A | A | A | B、A | 第 2 列 | 只更新紀錄，項目為完成（FR-DRF-002） |

04 A4 的三個輸出共用前景色的情境，是第一組案例與第二組案例的組合：曾接受 drift 的輸出走第 4 列或第 2 列，另外兩個輸出走第 4 列。

權限的兩個例子。宣告 `"0600"` 的輸出被外部改成 `0644`、內容不變：C 不符合 N，O 有 `mode` 而權限不同，屬第 5 列的 drift 衝突。原本沒有宣告權限的輸出，在 manifest 加上 `"0600"`：O、D 沒有 `mode`，所以沒有外部修改；N 有 `mode`，所以來源改變，屬第 4 列，只變更權限、不重寫內容（FR-MAN-015）。

### 5.12 接管備份

03 §12 第 2 項的定案：

- 位置：`<狀態目錄>/backups/<批次編號>/<目標在 home 內的相對路徑>`，例如 `~/.local/state/rosamta/backups/20261109T061530Z/.config/ghostty/config`。沿用目標的相對路徑，使用者不必查表就能找到原檔。批次編號已唯一（§5.1），同一批次內兩個目標不會解析為同一路徑（FR-PLN-009），所以備份的名稱唯一。
- 只在批次中有接管時才建立批次目錄。目錄權限為 `0700`。
- 備份就是原本的目錄項目：一般檔案保留原內容與權限（SEC-003）；符號連結只保存連結本身（§7.4、ASM-11）。建立方式與跨檔案系統時的處理見 §6；備份目錄與目標不在同一個檔案系統時，建立方式不同，但位置與命名規則相同。
- 回報列出備份的絕對路徑（FR-MAN-019）。
- 保留：批次成功後，rosamta 不刪除也不覆寫備份，由使用者自行刪除。備份可能是原檔唯一的副本，SRS 也沒有規定保留期限。一個目標被接管後就成為受管理的輸出，之後不會再接管，所以備份的數量不超過被接管的目標數。
- 批次失敗：整批回復把原檔還原到目標路徑後，移除該項的備份，因為原檔已經回到原處；本批的備份目錄變空時一併移除。某個目標無法回復時，它的備份保留，回報列出備份位置，讓使用者自行取回（FR-APL-004）。

接管備份與回復快照是兩個不同的東西（03 S2）：快照放在 `txn/<批次編號>/`，供程式回復用，批次結束時清理；接管備份供使用者取回原檔，依上述規則保留。

### 5.13 停止管理的表示

**內建輸出**（03 §12 第 6 項）：內建輸出不在 manifest 中，無法「移除宣告」。停止管理時，在 manifest 寫入只含 `removed = true` 的覆寫：

```toml
[builtin.alacritty]
removed = true
```

- 合併內建宣告時，`removed = true` 的內建輸出不進入有效 manifest，所以不會在下次載入時重新出現，也不會被啟用、套用或清除。
- 表格中不得有其他屬性，否則預檢失敗。停止管理時，這個表格原有的其他屬性一併移除。
- 列出輸出（FR-MAN-026）仍顯示這個內建輸出，標示為已移除，使用者刪除這個表格就能恢復。
- 這與移除使用者輸出的宣告效果相同：手動寫入 `removed = true` 而不刪除狀態紀錄時，舊目標依 §5.11 清除，就像手動從 manifest 刪掉一個使用者輸出。寫在 manifest 而不是本機設定，因為從 manifest 移除使用者輸出也會影響所有主機，兩者一致。

**未完成的停止管理**：停止管理要寫兩個檔案，manifest 在來源目錄，狀態檔在狀態目錄，無法一次替換。04 §2 要求能辨識只完成一筆寫入的情況，在兩份資料一致前不清除、接管或改動該項目。寫入順序如下：

1. 狀態檔：在該輸出的紀錄加上 `forget_pending: true`。
2. manifest：移除 `[outputs.<名稱>]`，或寫入 `[builtin.<名稱>] removed = true`。
3. 狀態檔：刪除該紀錄。

在第 1、2 步之後中止，紀錄都帶有 `forget_pending`，套用時該目標一律保留（§5.11），不會因為 manifest 已移除而被清除，也不會因為紀錄將被刪除而被接管。回報的原因提示使用者再執行一次停止管理；指令對已完成的步驟不重做，可以直接再執行。輸出從未套用過、沒有紀錄時，只做第 2 步。停止管理與套用之間的互斥見 §6。

## 6. 關鍵演算法與平台處理

本節定義路徑的表示與比較、檔案操作如何與重檢綁定、寫入與回復，以及多程序互斥，回答 03 §12 第 4、5 項與 §5 留給本節的決定。演算法只依賴 §7 的平台假設，其中本節新增的列在 §7.7。各演算法由哪個類別負責見 §4；同步點與故障注入見 §8。

### 6.1 home 與路徑

**home**：執行檔啟動時以 Rust 的 `std::env::home_dir` 取得 home，放進 `HostContext`，整次執行使用同一個值。Linux 與 macOS 取環境變數 `HOME`，未設定或為空時查詢使用者資料庫（`getpwuid_r`）；Windows 取 `USERPROFILE`，未設定或為空時呼叫 `GetUserProfileDirectoryW`。測試因此可以用環境變數指定 home（§8.1）。

home 必須是存在的目錄，而且是絕對路徑；否則除了顯示說明以外的子指令都回報 home 的值與原因，回傳 `1`。home 本身與它上層的路徑可以含連結，例如 `/home` 連到 `/var/home` 的系統，這是系統的配置，不在 rosamta 檢查的範圍內（ASM-05）。啟動時開啟 home 並持有它的 handle，記下檔案身分與實際路徑（Linux 與 macOS 用 `realpath`，Windows 用 `GetFinalPathNameByHandleW`）。之後所有的走訪都從這個 handle 開始。Windows 上這個 handle 不含 `FILE_SHARE_DELETE`，執行期間 home 不能被改名（ASM-08）。

**展開與正規化**：manifest 的 `target`、本機設定的 `source` 與 `XDG_*` 變數，依下列步驟轉成 home 內的相對路徑：

1. 展開 `~`、`${NAME}` 與 `$$`（§5.4），`~` 換成 home 的原值。
2. 展開後必須是絕對路徑。Linux 與 macOS 以 `/` 開頭；Windows 以磁碟代號加分隔符號開頭，例如 `C:\`，不接受 UNC 路徑（`\\server\share`）與裝置路徑（`\\?\`、`\\.\`）。
3. 以 `/` 分割，Windows 也以 `\` 分割。去掉空的元件與 `.`；遇到 `..` 時移除前一個元件。這是純文字的處理：rosamta 只依正規化後的路徑操作，不把原本的寫法交給作業系統解析，所以 `~/a/../b.conf` 一律指 `~/b.conf`，即使 `a` 是連結也一樣（FR-PLN-010）。
4. 去掉 home 的前綴。與 home 的原值或實際路徑相符都可以，例如 home 為 `/home/u`、實際路徑為 `/var/home/u` 時，兩種寫法都在 home 內。Linux 與 macOS 逐位元組比較；Windows 不分大小寫，因為磁碟代號與使用者目錄都由系統建立在不分大小寫的目錄中。不在 home 內時預檢失敗，回報原本的寫法與展開後的路徑（FR-MAN-010）。
5. 每個元件須符合名稱限制：不得為空；Windows 上不得含 `<>:"|?*` 與控制字元，不得以 `.` 或空格結尾，不得是保留的裝置名稱（`CON`、`PRN`、`AUX`、`NUL`、`COM1`–`COM9`、`LPT1`–`LPT9`，加上副檔名也一樣）。Win32 路徑會默默去掉結尾的 `.` 與空格，並把裝置名稱解讀成裝置，兩個不同的寫法因此可能指向同一個位置，或指向檔案系統以外，所以直接拒絕。拒絕 `:` 也排除了 NTFS 的替代資料流。

結果是 home 內的相對路徑，§5.10 的 `target` 就是它以 `/` 連接的寫法。目標、來源目錄、設定目錄與狀態目錄都不得等於 home 本身。Windows 的實作以 `\\?\` 加絕對路徑呼叫 API，不受 `MAX_PATH` 的長度限制；`\\?\` 不處理 `.` 與 `..`，所以正規化必須在此之前完成。

**走訪**：所有檔案操作都從 home 的 handle 逐層開啟目錄，不跟隨連結。Linux 與 macOS 用 `openat(O_NOFOLLOW | O_DIRECTORY)`（ASM-05）；Windows 以 `FILE_FLAG_OPEN_REPARSE_POINT` 開啟，拒絕 reparse point 並核對最終路徑（ASM-09），handle 帶 `FILE_LIST_DIRECTORY`、不含 `FILE_SHARE_DELETE`（ASM-08）。來源目錄中的範本與原樣複製來源，以同樣的方式從來源目錄的 handle 走訪（§3.1）。

規則只有一條：**路徑中的目錄都不得是連結**。目標本身可以是連結，依 04 §3.1 的表格處理。

| 情況 | 處理 |
| --- | --- |
| 啟用的輸出，目標的某一層上層目錄是連結 | 預檢失敗，指出是哪一層、連結指向哪裡，提示把 `target` 改成實際的路徑，回傳 `1`（04 §3.1「必須操作的目標類型不受支援」） |
| 待清除的舊目標，某一層上層目錄是連結 | 位置無法確認，保留檔案與紀錄（§5.11、A5） |
| 來源目錄、設定目錄或狀態目錄的路徑中有連結 | 除了顯示說明以外的子指令都失敗，回傳 `1` |
| 範本或原樣複製的來源檔是連結，或它的路徑中有連結 | 預檢失敗，回報路徑與類型（04 §3.1、A9） |

不跟隨目錄的連結，是因為預檢無法分辨使用者自己建立的連結和 A10 那種在兩次檢查之間換入的連結，兩者在檔案系統上完全相同。若要跟隨，每次重檢都要重新解析連結，並確認結果仍在 home 或來源目錄內，pilot 驗證的逐層 `O_NOFOLLOW` 也不再適用。代價是 `~/.config` 本身是連結的使用者，必須把 `target` 寫成連結指向的實際位置，預檢的訊息會指出這一點。來源檔不接受連結，理由相同；多個輸出可以直接在 manifest 引用同一個來源檔，不需要以連結共用。git 在不支援連結的環境（例如 Windows 未開啟 `core.symlinks`）會把連結存成一般檔案，這時讀到的是一般檔案。

尚不存在的上層目錄在執行時建立（§6.3）。

### 6.2 位置的比較

FR-PLN-009 與 FR-PLN-010 的目標碰撞、§5.2 的保留位置、§5.10 的紀錄比對，以及初始化時三個目錄互不包含，都使用同一種比較。路徑先依 §6.1 正規化，再從 home 走訪，轉成**位置鍵**：

- 已存在的元件，以檔案身分表示（ASM-04）。目標本身存在時也一樣。
- 從第一個不存在的元件起，以名稱表示。最深一個存在的目錄不分大小寫時，名稱逐字元轉成小寫（Unicode 的小寫對應）後再比較；新建的目錄沿用上層的設定，所以其後的元件都依同一規則。目錄是否區分大小寫由 `FileSystem` 回答（§3.1、§6.8）。

兩個位置鍵相同，表示同一個位置；一個是另一個的前段，表示一個路徑被當成另一個路徑的上層目錄，也就是同一路徑同時被當成檔案與目錄（04 §3.1）。

已存在的部分以身分比較，不必猜測檔案系統的命名規則：只差大小寫的名稱、Windows 的 8.3 短檔名，以及在 home 內兩處掛載同一個目錄（bind mount），都會得到相同的身分。指向同一個檔案的兩個 hard link 也算碰撞。

尚不存在的部分只能比較名稱，可能與檔案系統的實際規則不同，例如 macOS 把 Unicode 標準等價的名稱視為同一個檔案，而上述規則只處理大小寫。這類漏判不會造成覆寫：建立檔案一律不覆蓋既有項目（§6.3），後建立的一項會因目標已存在而失敗，依執行失敗整批回復（§6.6）。

FR-PLN-010 的兩個驗收例：`~/a/../b.conf` 在正規化時就變成 `~/b.conf`。Windows 上的 `~/.Config/app.conf` 與 `~/.config/app.conf`，若 `.config` 已存在，兩者走訪到同一個目錄身分；若不存在，home 不分大小寫，比較轉成小寫的名稱。兩種情況都判為碰撞，回報列出每個輸出的名稱、宣告位置與目標路徑（FR-PLN-009）。

位置鍵只在一次執行內有效（ASM-04）。狀態檔以相對路徑與父目錄身分記錄位置（§5.10）；載入後把紀錄的路徑轉成位置鍵，再與本次的目標比對。因此在 Windows 上把 `target` 的 `.Config` 改成 `.config`，新舊路徑是同一個位置，紀錄沿用並改寫成新的寫法，不會清除後重建。

### 6.3 觀測與操作綁定

預檢通過，不代表執行時目標仍然相同（03 §12 第 4 項）。本節的做法：規劃時記錄觀測；每次操作前依觀測核對，並把核對和操作綁在同一組 handle 上；平台無法綁定的空窗，以操作後的核對找出並復原。

**觀測**：規劃時，`Planner` 為每個目標記錄下列資料。狀態檔、本機設定與 manifest 的讀取也記錄同樣的觀測。

- 位置：從 home 到父目錄，每一層的檔案身分。
- 存在狀態與類型：一般檔案、目錄、連結或其他類型，不跟隨連結判斷。Windows 只有 `IO_REPARSE_TAG_SYMLINK` 算連結，其他 reparse point（junction、OneDrive 的雲端檔案等）屬於其他類型（ASM-09）。
- 身分：目標存在時的檔案身分。
- 版本：一般檔案內容的 SHA-256，以及 Linux 與 macOS 的權限位元。開檔時使用 `O_NOFOLLOW | O_NONBLOCK`（ASM-21），開啟後以 handle 核對身分與類型，再讀取內容。不是一般檔案的目標只看類型，不開檔讀取（04 §3.1、A18）。

**綁定**：每一個會改變檔案、或讀取快照與備份的操作，都依下列步驟執行：

1. 從 home 走訪到父目錄，核對每一層的身分與觀測相同。本批建立的目錄，以建立時記下的身分核對。
2. 核對目標的存在狀態、類型與身分。操作依賴內容時（取代、刪除、擷取快照、製作備份），經由開啟的 handle 計算 SHA-256，與觀測的版本比較。
3. 透過步驟 1 持有的父目錄 handle 執行操作（下表）。
4. 再從 home 走訪一次，確認路徑仍通往持有的父目錄，且目標是操作預期的結果：本批寫入的檔案，或已不存在。
5. 步驟 1、2 不符時，不執行操作。步驟 4 發現路徑已不通往持有的目錄時，經由同一組 handle 復原這次操作（ASM-06）；發現目標不是預期的結果時，不再動它。三種情況都是觀測失效，依執行失敗處理（§6.6）。

Windows 的目錄 handle 擋下了改名（ASM-08），步驟 4 是多一層保險；Linux 與 macOS 擋不住（ASM-06），步驟 4 是唯一的防線。

各操作的實作如下，P 是持有的父目錄 handle，N 是目標的名稱，暫存檔見 §6.4：

| 操作 | Linux、macOS | Windows |
| --- | --- | --- |
| 建立 | 暫存檔寫好後 `linkat(P, 暫存檔, P, N)`，再以 `unlinkat` 移除暫存檔的名稱。N 已存在時 `linkat` 以 `EEXIST` 失敗，不覆蓋（ASM-16） | `MoveFileExW(暫存檔, N, MOVEFILE_WRITE_THROUGH)`，不指定 `MOVEFILE_REPLACE_EXISTING`，N 已存在時失敗（ASM-16） |
| 取代 | 交換式改名：以 `renameat2(RENAME_EXCHANGE)`（Linux）或 `renameatx_np(RENAME_SWAP)`（macOS）交換暫存檔與 N（ASM-17）。交換後，暫存檔的名稱指向被取代的檔案；核對它的身分與版本仍是觀測所見，不符時再交換一次換回來 | 以 `FILE_READ_DATA` 開啟目標，共用模式只有 `FILE_SHARE_READ`，核對身分與版本；持有期間其他程序無法改名、刪除或寫入目標（ASM-18）。核對後關閉，立即 `MoveFileExW(MOVEFILE_REPLACE_EXISTING \| MOVEFILE_WRITE_THROUGH)`（ASM-02） |
| 刪除 | 以 `renameat` 把 N 改成暫存檔名稱，核對它的身分與版本，相符才 `unlinkat`；不符時以不覆蓋的改名移回 N（ASM-16）。被換入的可能是目錄，`linkat` 無法為目錄建立名稱，所以不用「建立」的方式 | 以 `DELETE \| FILE_READ_DATA` 開啟，共用模式只有 `FILE_SHARE_READ`，核對身分與版本後設定 `FileDispositionInfo`（ASM-10、ASM-18） |
| 變更權限 | `openat(O_NOFOLLOW \| O_NONBLOCK)` 開啟，核對身分與版本後 `fchmod` | 不適用（§5.4） |
| 建立目錄 | `mkdirat(P, N)`，再以 `O_NOFOLLOW` 開啟並記下身分 | `CreateDirectoryW`，再依 §6.1 開啟並記下身分 |

取代與刪除都先把舊檔移到另一個名稱、確認無誤後才放手：被換入的檔案、特殊檔案，或在最後一次核對後對原檔的就地修改，都會在交換或移開後被發現，並移回原位保留（A18、A19）。建立不覆蓋，所以最後一次確認位置為空之後才出現的檔案也會保留（A15）。

**剩下的空窗**：Windows 的取代，在關閉 handle 到 `MoveFileExW` 之間仍有空窗；Linux 與 macOS 的檔案系統不支援交換時（`EINVAL` 或 `ENOTSUP`），改用 `renameat` 直接取代，在最後一次核對與 `renameat` 之間也有空窗（ASM-07）。空窗內被換入的檔案或對原檔的修改會被取代掉，事後也無法發現。這段空窗只有一次系統呼叫的長度，位於同一個 `FileSystem` 呼叫內部，§8 的同步點都在它之外。

**管理與初始化的寫入**（04 §3.3 末段、A7、A12）沿用同一套操作：

- manifest 與本機設定的修改用「取代」，被取代的必須是讀取時的版本。使用者在指令執行期間以編輯器修改了檔案時，指令回報失敗，保留使用者的版本。原本不存在的本機設定用「建立」。
- 納入管理的來源副本用「建立」。預定位置在最後一次確認後才被占用時，依 §5.4 改用下一個名稱。寫入順序是先建立副本、再修改 manifest；manifest 修改失敗時刪除本次建立的副本。
- 初始化取得遠端來源時，先 clone 到正式位置所在目錄中的 `.<名稱>.rosamta-tmp.<程序編號>.<序號>/`，確保與正式位置在同一個檔案系統，再以不覆蓋的方式把整個目錄改名到正式位置：Linux 用 `renameat2(RENAME_NOREPLACE)`，macOS 用 `renameatx_np(RENAME_EXCL)`，Windows 用不指定取代的 `MoveFileExW`（ASM-16）。git 以路徑寫入暫存目錄，rosamta 無法綁定它的操作，只能在 clone 前後核對暫存目錄與它的上層仍是建立時的身分。

### 6.4 檔案的寫入與權限

**暫存檔**：寫入一律先寫暫存檔，放在目標所在的目錄，命名依 §5.1。Linux 與 macOS 以 `openat(P, O_CREAT | O_EXCL | O_NOFOLLOW, 0600)` 建立（ASM-03），Windows 以 `CREATE_NEW` 建立。寫入完整內容後 `fsync`（Windows 為 `FlushFileBuffers`），設定最終的權限，記下它的身分，再依 §6.3 建立或取代。與 pilot P2 相同（ASM-01、ASM-02）。

**權限**（Linux、macOS）：暫存檔以 `0600` 建立，內容寫完後、建立或取代之前，以 `fchmod` 設成最終的權限：

| 情況 | 最終的權限 |
| --- | --- |
| 輸出宣告了 `permissions` | 宣告的值（FR-MAN-015） |
| 未宣告，目標已存在 | 原檔的權限位元（FR-MAN-016） |
| 未宣告，目標不存在 | `0666` 去掉 umask，與一般程式建立的檔案相同 |

內容在 `0600` 下寫入，寫完才放寬，不完整的內容不會被其他帳號讀到；最終的權限與輸出寫好後本來就會有的權限相同。新建的輸出不預設 `0600`，因為設定檔通常不需要保密，預設值與使用者以編輯器建立的檔案一致；需要保密的輸出以 `permissions` 宣告，例如 §5.4 的 `ssh-config`。umask 只能透過設定它來讀取，由執行檔在啟動時、還沒有其他執行緒時讀取一次並立即還原，交給 platform 的 `FileSystem` 實作。

**Windows 的存取控制**：

- 暫存檔沿用目錄繼承的 ACL（ASM-02）。取代既有檔案時，若原檔的 DACL 含有明確設定的項目，或設為不繼承上層，在寫入內容之前以 `GetSecurityInfo`、`SetSecurityInfo` 把原檔的 DACL 複製到暫存檔（ASM-19），取代後保有原本的存取設定（FR-MAN-016）。原檔的 DACL 全是繼承來的項目時不必複製，暫存檔繼承到的是同一組。
- rosamta 建立的狀態目錄、設定目錄，以及 `backups/`、`txn/` 底下的批次目錄，建立時就指定不繼承上層的 DACL：目前的使用者、`SYSTEM` 與 `Administrators` 完全控制，並由子項目繼承（ASM-19）。這與 Windows 為使用者 home 預設的設定相同，其他一般帳號無法存取（04 §3.3）。Linux 與 macOS 上這些目錄是 `0700`（§5.1）。目錄已存在時，不修改它的權限或 ACL。
- Windows 沒有權限位元，`permissions` 不套用，狀態檔的 `mode` 為 `null`（§5.4、§5.10）。SEC-003 只適用於 Linux 與 macOS。

### 6.5 回復快照、接管備份與殘留檔案

**回復快照**：批次中第一次要改動檔案時，`RecoveryManager` 建立 `txn/<批次編號>/`（§5.1）。每個要被取代或刪除的既有一般檔案，在操作前擷取快照：

1. 依 §6.3 步驟 1、2 開啟目標，經由同一個 handle 讀取，邊讀邊計算 SHA-256，寫入 `txn/<批次編號>/` 中的副本（Linux 與 macOS 為 `0600`），另記下原檔的權限位元，Windows 則記下 DACL。
2. 讀完後，以計算出的 SHA-256 與觀測的版本比較；不符時捨棄副本，視為觀測失效（A16）。比較的是實際讀到的位元組，讀取期間的修改會使結果不符，即使讀完後又改回也一樣。
3. 依 §6.3 步驟 4 確認路徑仍通往原本的目錄，確保讀到的不是替代位置的檔案（A16）。

只變更權限的項目不複製內容，只記下原權限。副本以序號命名，與目標的對應關係保存在記憶體中：批次只在同一個程序內回復，03 S5 沒有程序中止後自動復原的需求。

快照用複製，不用 hard link。hard link 與原檔是同一個檔案，擷取後原檔被就地修改時，快照也跟著改變，保存不了規劃時的版本。複製的代價是空間（§6.6）。

**接管備份**的建立方式（§5.12）：

- 一般檔案：先嘗試在備份目錄為原檔建立 hard link。Linux 與 macOS 用 `linkat`，建立後核對備份的身分與原檔相同；Windows 以原檔的 handle 用 `FileLinkInformation` 建立（ASM-11）。hard link 就是原檔本身，權限自然相同（SEC-003），也不占額外空間。備份目錄與目標不在同一個檔案系統，或無法建立 hard link 時（例如 `EXDEV`、`ERROR_NOT_SAME_DEVICE`，或 Linux 的 `fs.protected_hardlinks` 拒絕），改以快照的方式複製，Linux 與 macOS 再把副本的權限設成原檔的權限（SEC-003）。兩種方式都在建立後核對備份的版本。
- 連結：只能以 hard link 備份（§7.4），所以備份目錄必須與目標在同一個檔案系統。規劃時比較兩者的檔案系統（Linux 與 macOS 的 `st_dev`、Windows 的 volume 序號，在同一次執行內有效，ASM-04），不同時預檢失敗，回報目標與狀態目錄，回傳 `1`（04 §3.1「無法安全備份」）。不改用 `readlinkat` 加 `symlinkat` 複製連結：那樣回復時就要在目標路徑建立新的連結，正是 §7.4 為了 Windows 標準使用者與 DC-003 而避開的做法。連結的回復以這份備份為來源，不另外擷取快照。

**殘留檔案的清理**（§5.1 的暫存檔與 `txn/`）：

- 批次中每個暫存檔的名稱，包括 §6.3 交換或移開後指向舊檔的名稱，在建立前先把它在 home 內的相對路徑附加到 `txn/<批次編號>/journal`（每行一筆）。
- 批次成功或回復結束後，依 journal 刪除仍存在的暫存檔，再刪除整個 `txn/<批次編號>/`。刪除經過 §6.3 的走訪，只刪除名稱含 `.rosamta-tmp.` 的項目，不跟隨連結。§6.3 中移不回原位而保留下來的外部檔案不刪除，回報它的位置。
- 程序中止後，下一個取得鎖定（§6.7）的指令在規劃之前，依 journal 清理每一個殘留的批次目錄，並在標準錯誤說明該批次沒有正常結束。中止的批次可能已改動部分輸出，但每個輸出都是完整的舊版或新版（REL-001），狀態檔也是（REL-002）。已寫入、狀態卻未提交的輸出，下次規劃時內容符合期望，依 §5.11 只更新紀錄或直接納管；已刪除而未提交的，依「舊檔不存在」只移除紀錄。所以不需要另外的復原步驟。
- 預覽與查詢不取得鎖定，也不寫入，所以不清理；發現殘留的批次目錄時，只在標準錯誤提示。
- 清理失敗只在標準錯誤警告，不改變已提交的結果與 exit code（04 §3.3）。

### 6.6 批次的執行與回復

本節細化 03 S5。預檢通過後，`Executor` 依下列步驟執行：

1. **開始**：狀態檔存在時，依 §6.5 擷取它的快照，核對與首次載入的版本相同（A25）；不存在時記下「不存在」。失敗時不改動任何目標。
2. **逐項處理**：依計畫的順序處理每個項目，順序與預覽列出的相同。保留的項目不動。要改動目標的項目，先擷取快照、需要時製作接管備份，再依 §6.3 執行。只更新紀錄的項目不動目標。任何一步失敗或觀測失效，停止處理後續項目，進入回復。
3. **提交前重檢**：依 04 §3.3，核對所有預定回報完成的項目，包括「不變」與只更新紀錄的項目。建立或更新的目標仍是本批寫入的檔案，內容與適用的權限都相同；刪除的目標仍不存在；其餘項目的身分、版本與宣告的權限仍與觀測相同。狀態檔仍是首次載入的版本。任何一項不符就是觀測失效。
4. **提交狀態**：狀態有變更時，把新狀態寫成暫存檔，依 §6.3 取代狀態檔，被取代的必須是首次載入的版本（A23）；原本沒有狀態檔時用「建立」。
5. **完成確認**：再做一次步驟 3 的核對；狀態檔改為核對它是本批寫入的檔案（有提交時），或仍是首次載入的版本（無須提交時）。通過才判定批次完成（A17、A26）。
6. **結束**：依 §6.5 清理快照、journal 與暫存檔，回報各項結果與接管備份的位置。

步驟 3 讓已經失效的批次不必提交狀態；步驟 5 涵蓋提交期間發生的改動。04 §3.3 要求重檢條件「維持到批次完成」，以步驟 5 為界，之後的改動不屬於本批。

**回復**：步驟 2 到 5 失敗時，依操作的相反順序處理本批改動過的每個目標，最後處理狀態檔。每一項都依 §6.3 綁定，先核對目標仍是本批操作後的結果：

| 本批的操作 | 回復前的核對 | 回復方式 |
| --- | --- | --- |
| 建立 | 目標仍是本批寫入的檔案，內容與權限相同 | 依 §6.3 刪除 |
| 更新、接管一般檔案 | 同上 | 以快照的內容與原權限（Windows 為原 DACL）寫成暫存檔，依 §6.3 取代。接管的備份在取代成功後刪除（§5.12） |
| 接管連結 | 同上 | 把備份改名回目標路徑，取代本批寫入的檔案（§7.4） |
| 刪除 | 父目錄仍在原位，目標仍不存在 | 以快照的內容與原權限依 §6.3 建立；位置已被占用時不覆蓋（A22） |
| 變更權限 | 目標的身分與內容未變，權限仍是本批設定的值 | 以 `fchmod` 改回原權限 |
| 建立目錄 | 仍是本批建立的目錄，而且是空的 | 刪除；不符時保留，不算回復失敗 |
| 狀態檔 | 目前是本批寫入的版本 | 核對快照仍是首次載入的版本後（A25），依 §6.3 取代；原本沒有狀態檔時刪除 |

使用快照之前，先核對副本的 SHA-256 仍與擷取時相同；不同時不使用它（A16）。核對不符或無法確認的項目保留現況，記下路徑與原因，繼續處理下一項（04 §3.3）。回報說明是否全部已回復，否則逐項列出未能回復的路徑與原因（FR-APL-003、FR-APL-004）。

**空間**（04 §3.3）：執行期間需要的空間有四部分：

- 所有啟用輸出的期望內容，在規劃時讀入記憶體，執行時不重讀來源（04 §2、A24）。
- 同一時間最多一個暫存檔，位於目標所在的檔案系統，大小等於該輸出。
- 回復快照，位於狀態目錄的檔案系統，總和等於本批要取代或刪除的既有檔案大小。
- 跨檔案系統的接管備份，大小等於被接管的檔案；同一個檔案系統時是 hard link，不占空間。

空間不足時，寫入暫存檔或快照會失敗，依執行失敗整批回復。回復只寫入暫存檔，需要的空間不超過被回復的單一檔案；它在本批取代時已經寫入過同樣大小的暫存檔。

### 6.7 多程序互斥

SRS AS-5 假設同一組目標同時只有一個 rosamta 程序在套用；03 §12 第 4 項要求說明多個 rosamta 同時執行時怎麼處理。

- 會寫入的子指令（套用，包括指定接受 drift；納入管理、修改屬性、停止管理；初始化）在讀取任何設定之前，取得 `<狀態目錄>/lock` 的獨占鎖定，持有到程序結束。取不到時立即回報「另一個 rosamta 程序正在執行」與鎖定檔的路徑，回傳 `1`，不等待。
- 預覽、摘要、查詢與說明不取得鎖定：鎖定檔可能需要建立，違反預覽與查詢的唯讀要求（FR-PLN-004、FR-QRY-006）。另一個程序正在套用時，預覽看到的是當下的檔案。
- Linux 與 macOS：經 §6.1 的走訪，以 `openat(O_RDWR | O_CREAT | O_NOFOLLOW, 0600)` 開啟鎖定檔，再呼叫 `flock(LOCK_EX | LOCK_NB)`。不用 `fcntl` 的記錄鎖：同一個程序關閉該檔案的任何一個 fd，`fcntl` 鎖就會釋放。Windows：以共用模式 `0` 和 `OPEN_ALWAYS` 開啟，已被其他程序開啟時以 `ERROR_SHARING_VIOLATION` 失敗（ASM-20）。
- 程序結束或被終止時，作業系統釋放鎖定，不會留下需要手動刪除的鎖。鎖定檔建立後不刪除：刪除後再建立，兩個程序可能各自鎖住不同的檔案。
- 初始化時狀態目錄可能還不存在，先依 §5.1 建立狀態目錄，再取得鎖定。

**停止管理與套用**（§5.13、FR-MAN-025、04 §2）：兩者取得同一個鎖定，不會交錯執行。套用在取得鎖定之後才規劃，不沿用先前預覽的計畫，所以停止管理完成後的套用，一定依新的 manifest 與狀態檔規劃；04 §2 提到的計畫版本檢查不需要另外實作。A5 最後一個情境中，若在套用持有鎖定時執行停止管理，停止管理會因取不到鎖定而失敗，套用依原計畫完成，屬於 04 §5「平台阻止外部操作」的情形。

鎖定只協調 rosamta 程序之間的執行。其他程式的改動，由 §6.3 的核對處理。兩個 rosamta 程序使用不同的狀態目錄時（例如 `XDG_STATE_HOME` 不同），鎖定無效，這已超出 AS-5 的範圍，但 §6.3 的核對仍然有效。

### 6.8 三平台差異

| 項目 | Linux | macOS | Windows |
| --- | --- | --- | --- |
| home | `HOME`，未設定時查 `getpwuid_r` | 同 Linux | `USERPROFILE`，未設定時 `GetUserProfileDirectoryW` |
| 檔案身分 | `st_dev` 加 `st_ino`；狀態檔只記 `st_ino`（ASM-15） | 同 Linux | volume 序號加 128 位元 file ID |
| 走訪目錄 | `openat(O_NOFOLLOW \| O_DIRECTORY)` | 同 Linux | `FILE_FLAG_OPEN_REPARSE_POINT`，拒絕 reparse point，核對最終路徑 |
| 操作期間目錄被改名 | 擋不住；操作後重檢並復原（ASM-06） | 同 Linux | handle 不含 `FILE_SHARE_DELETE`，擋下（ASM-08） |
| 是否區分大小寫 | 預設區分；目錄有 casefold 屬性（`FS_CASEFOLD_FL`）時不分 | 依 `fpathconf(_PC_CASE_SENSITIVE)`；APFS 預設不分 | 預設不分；目錄設定了區分大小寫（`FILE_CS_FLAG_CASE_SENSITIVE_DIR`）時區分 |
| 建立 | `linkat`，不覆蓋 | 同 Linux | `MoveFileExW`，不指定取代 |
| 取代 | `renameat2(RENAME_EXCHANGE)` 後核對；不支援時 `renameat` | `renameatx_np(RENAME_SWAP)` 後核對；不支援時 `renameat` | 持有 handle 核對後，`MoveFileExW` 取代 |
| 刪除 | 移開、核對後 `unlinkat`；不符時 `renameat2(RENAME_NOREPLACE)` 移回 | 移開、核對後 `unlinkat`；不符時 `renameatx_np(RENAME_EXCL)` 移回 | handle 的 `FileDispositionInfo` |
| 不覆蓋的改名（初始化、刪除時移回） | `renameat2(RENAME_NOREPLACE)` | `renameatx_np(RENAME_EXCL)` | `MoveFileExW`，不指定取代 |
| 權限 | 權限位元；新檔為 `0666` 去掉 umask | 同 Linux | 不套用 `permissions`；有明確項目的 DACL 複製到新檔 |
| 程式目錄的存取限制 | `0700` | 同 Linux | 不繼承的 DACL：本人、`SYSTEM`、`Administrators` |
| 視為連結的目標 | 符號連結 | 同 Linux | 只有 `IO_REPARSE_TAG_SYMLINK`；其他 reparse point 屬於其他類型 |
| 不開檔讀取的類型 | 目錄、FIFO、裝置檔、通訊端 | 同 Linux | 目錄、`IO_REPARSE_TAG_SYMLINK` 以外的 reparse point |
| 建立符號連結 | 不需要（ASM-13） | 不需要 | 不需要 |
| 鎖定 | `flock` | `flock` | 共用模式 `0` |
| 名稱限制 | 不含 `/` 與 NUL | 同 Linux | §6.1 第 5 步 |
| 路徑長度 | — | — | 內部使用 `\\?\` |

是否區分大小寫的查詢只影響尚不存在的路徑的碰撞判定。查詢失敗時（例如舊版 Windows 不支援 `FileCaseSensitiveInfo`），採用該平台的預設值；判定有誤時，最壞的結果是碰撞在執行時才被不覆蓋的建立擋下（§6.2）。

## 7. 平台假設清單

本節列出設計依賴、而 Rust 標準函式庫不保證的作業系統行為。每項假設以 `ASM-NN` 編號，列出官方文件依據、文件保證的範圍，以及 pilot 結果（`prototypes/pilot/`）。§6 的演算法只依賴本節列出的行為；06 為每個 `ASM-NN` 安排測試情境（§13）。

pilot 結果分四種：

- 通過：官方文件有保證，pilot 在三平台的結果也相符。
- 有條件：pilot 在三平台都相符，但相關行為官方文件沒有寫明，或須由設計另外確保某個前提。條件寫在「保證範圍與條件」欄。
- 未驗證：pilot 無法重現這個行為，例如需要重新開機。須寫明假設不成立時的後果，以及由 pilot 或 06 如何驗證。
- 不可行：這個做法無法滿足需求，須改變設計或修訂 SRS（04 §3.3 末段）。

本版沒有「不可行」的項目。「未驗證」的只有 ASM-15。P4、P5 驗證的是選定套件的行為，不屬於平台假設，結果見 §2.2、§2.3；兩者在三平台都通過。

### 7.1 驗證環境

證據來自 pilot workflow 的 CI run 42634818704，P1–P5 與發佈檢查全部通過。§7.7 的 P6 結果來自 CI run 42785134928，這個 run 也重跑了 P1–P5 與發佈檢查，全部通過；環境與下表相同，只有 ubuntu runner 的 kernel 為 7.0.0-1014-aws。runner 映像檔固定版本；正式支援的平台版本由 06 決定（PORT-001）。

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
| ASM-07 | Linux 與 macOS 沒有以 handle 刪除或改名檔案的操作，核對檔案身分到 `unlinkat`、`renameat` 之間仍有空窗 | POSIX `unlinkat`、`renameat` 只接受名稱 | 這個空窗內的替換無法事先阻止。§6.3 的取代以交換式改名（ASM-17）、刪除以先移開再核對，把被換入的檔案移到另一個名稱後再核對，不會刪除它；只有檔案系統不支援交換時，取代仍有這個空窗。§8.2 的 `BeforeOperation` 同步點就在這個空窗；不支援交換時退回 `renameat` 的情形由 06 驗證 | 有條件。P1 的同步點不在這個空窗內；持續替換的 300 輪沒有違反檢查條件。P6 在這個空窗換入一般檔案、連結或目錄，或就地修改，交換式改名的取代與先移開的刪除都發現並放回原位（ASM-17） |
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

### 7.6 跨次執行的檔案身分

ASM-04 只在同一批次內比較檔案身分。狀態檔另外記錄目標父目錄的身分（§5.10），供下一次套用確認父目錄沒有被替換（04 §3.1），因此依賴下列假設。

| 編號 | 假設 | 依據 | 保證範圍與條件 | pilot 結果 |
| --- | --- | --- | --- | --- |
| ASM-15 | 目錄存在期間，它的檔案編號跨程序、跨重新開機都不變：Linux 與 macOS 的 `st_ino`，Windows 的 volume 序號與 128 位元 file ID。狀態檔不記錄 Linux 與 macOS 的 `st_dev` | `BY_HANDLE_FILE_INFORMATION` 文件：NTFS 上的檔案在刪除前保持同一個 file ID。POSIX `<sys/stat.h>` 只規定 `st_dev` 加 `st_ino` 在系統內唯一，沒有規定重新掛載後不變 | `st_dev` 由核心在掛載時配給，例如 btrfs subvolume 使用匿名裝置編號，重新開機後可能不同，所以不記錄。不比較 `st_dev` 時，不同檔案系統上的目錄可能編號相同；目錄刪除後，編號也可能被新目錄重複使用（ASM-04）。這兩種誤判都會把父目錄當成沒有被替換，但 §5.11 仍要求內容符合紀錄的實際基準才會覆寫或刪除，所以被改動的內容與 rosamta 上次寫入的相同。反過來，編號若意外改變，紀錄判為位置無法確認：啟用的輸出改按未受管理處理，清除時保留檔案，不會誤寫或誤刪 | 未驗證。CI 無法重新開機。本機 btrfs 的 home 裝置編號為 0:47，major 為 0，是匿名裝置。06 在實機上於重新開機前後各套用一次，確認沒有紀錄被判為位置無法確認 |

### 7.7 §6 新增的假設

§6 的演算法另外依賴下列行為，由 pilot P6 驗證（CI run 42785134928，§7.1）。假設涉及其他程序的（ASM-18、ASM-20），由另一個 pilot 程序執行外部操作；其餘在 `BeforeOperation` 同步點由測試程序自己改動目標。假設不成立時的後果寫在「保證範圍與條件」欄。

| 編號 | 假設 | 依據 | 保證範圍與條件 | pilot 結果 |
| --- | --- | --- | --- | --- |
| ASM-16 | 有不覆蓋既有項目的建立與改名：Linux 與 macOS 的 `linkat` 在新名稱已存在時以 `EEXIST` 失敗；不覆蓋的改名用 `renameat2(RENAME_NOREPLACE)`（Linux）與 `renameatx_np(RENAME_EXCL)`（macOS），目錄與檔案都適用；Windows 的 `MoveFileExW` 不指定 `MOVEFILE_REPLACE_EXISTING` 時，目的地已存在就失敗 | POSIX `link`：path2 已存在時以 `EEXIST` 失敗，連結也算存在。Linux `rename(2)`：`RENAME_NOREPLACE` 在 newpath 已存在時失敗，須檔案系統支援。macOS `rename(2)`：`RENAME_EXCL` 須檔案系統支援（`VOL_CAP_INT_RENAME_EXCL`）。`MoveFileExW` 文件：未指定該旗標時不取代既有檔案 | `linkat` 需要檔案系統支援 hard link；不支援時（例如 FAT 系列）建立會失敗，依執行失敗整批回復，不會覆寫。不覆蓋的改名用於初始化與 §6.3 刪除時的移回。不支援時初始化回報失敗；移回失敗時，被移開的項目留在暫存名稱下，回報它的位置，不刪除（§6.5）。pilot 沒有測試不支援的檔案系統 | 通過。三平台上，名稱已被一般檔案、目錄或連結占用時（Windows 測一般檔案與目錄），建立都失敗（Linux 與 macOS 為 `EEXIST`，Windows 為 `ERROR_ALREADY_EXISTS`），占用的項目與暫存檔都不變。目錄改名到空目錄、非空目錄或一般檔案也都失敗。對照組：Linux 與 macOS 的一般 `renameat` 會取代空目錄 |
| ASM-17 | Linux 的 `renameat2(RENAME_EXCHANGE)` 與 macOS 的 `renameatx_np(RENAME_SWAP)` 以一次操作交換兩個名稱，任何時間點兩個名稱都指向完整的檔案 | Linux `rename(2)`：`RENAME_EXCHANGE` 以原子方式交換 oldpath 與 newpath，兩者都須存在，Linux 3.15 起提供，須檔案系統支援。macOS `rename(2)`：`RENAME_SWAP` 交換來源與目的地，須檔案系統支援（`VOL_CAP_INT_RENAME_SWAP`） | 不支援時回傳 `EINVAL` 或 `ENOTSUP`，§6.3 改用 `renameat`，最後一次核對與取代之間留有空窗（ASM-07）。程序在交換後被終止時，目標是完整的新檔，舊檔留在暫存檔的名稱下，由 §6.5 清理。pilot 沒有測試不支援交換的檔案系統，退回 `renameat` 的路徑由 06 驗證 | 通過。Linux 與 macOS 交換後，兩個名稱的身分互換；目標是連結時，搬動的是連結本身，所指檔案不變；另一個名稱不存在時以 `ENOENT` 失敗。另一個程序反覆交換時讀取 10,000 次、終止 50 次，沒有一次找不到檔案或讀到不完整的內容。依 §6.3 實作的取代與刪除，在 `BeforeOperation` 把目標換成一般檔案、連結或目錄，或就地修改，都判定不符並放回原位，沒有留下暫存檔 |
| ASM-18 | Windows 上，以 `FILE_READ_DATA` 開啟、共用模式只有 `FILE_SHARE_READ` 的檔案 handle 存在期間，其他程序無法改名、刪除或寫入該檔案 | `CreateFileW` 文件：沒有 `FILE_SHARE_DELETE` 時，其他程序以 delete 存取權開啟會失敗；沒有 `FILE_SHARE_WRITE` 時，以寫入存取權開啟會失敗 | ASM-08 的檔案版本。ASM-08 發現只有屬性存取權的 handle 不參與共用檢查，所以須帶 `FILE_READ_DATA`。關閉 handle 到 `MoveFileExW` 之間的空窗不受保護（§6.3）。其他程序以 `MoveFileExW` 取代目標時，失敗的錯誤是 `ERROR_ACCESS_DENIED`，不是 sharing violation | 有條件。兩個 Windows job 中，取代與刪除持有目標期間，另一個程序的改名、刪除與寫入都以 `ERROR_SHARING_VIOLATION` 失敗，取代目標以 `ERROR_ACCESS_DENIED` 失敗，讀取成功；之後操作照計畫完成。對照組：只有屬性存取權的 handle 擋不住改名、刪除與寫入 |
| ASM-19 | Windows 上，`CreateDirectoryW` 可在建立時指定 DACL，之後在目錄中建立的檔案繼承它；`GetSecurityInfo`、`SetSecurityInfo` 可經由 handle 讀取並設定檔案的 DACL，包含是否繼承上層 | `CreateDirectoryW` 文件：`lpSecurityAttributes` 指定新目錄的安全描述元。`SetSecurityInfo` 文件：`PROTECTED_DACL_SECURITY_INFORMATION` 使 DACL 不繼承上層 | 讀取原檔的 DACL 須有 `READ_CONTROL`，設定暫存檔的 DACL 須有 `WRITE_DAC`，檔案擁有者兩者都有。不成立時，取代後的輸出改用目錄繼承的 ACL（同 ASM-02），程式目錄則沿用上層的 ACL | 通過。兩個 Windows job 中，以 `D:P` 加三個 `OICI` 項目建立的目錄，DACL 與指定的相同；其中新建的檔案只有這三個繼承來的項目。原檔的 DACL 不繼承上層，或含明確項目時，複製到暫存檔再以 `MoveFileExW` 取代，取代後的 SDDL 與原本完全相同。對照組：不複製時，明確項目與不繼承的設定都不見了，只剩目錄繼承的項目，與 ASM-02 相符 |
| ASM-20 | Linux 與 macOS 的 `flock(LOCK_EX \| LOCK_NB)`，在其他程序持有鎖定時以 `EWOULDBLOCK` 失敗；Windows 以共用模式 `0` 開啟的檔案，其他程序再開啟時以 `ERROR_SHARING_VIOLATION` 失敗。兩者都在程序結束時釋放 | Linux `flock(2)`：鎖定屬於 open file description，所有相關的 fd 關閉時釋放。macOS `flock(2)`：`LOCK_NB` 時以 `EWOULDBLOCK` 失敗。`CreateFileW` 文件：共用模式為 `0` 時，其他程序無法開啟該檔案 | `flock` 不屬於 POSIX，但 Linux 與 macOS 都提供。網路檔案系統不在範圍內。不成立時，兩個 rosamta 程序可能同時套用，這超出 AS-5 的範圍，§6.3 的核對仍然有效 | 通過。三平台上，持有鎖定時另一個程序都失敗（Linux 與 macOS 為 `EWOULDBLOCK`，Windows 為 `ERROR_SHARING_VIOLATION`）。Linux 與 macOS 在同一個程序內開啟再關閉同一個檔案的另一個 fd，鎖定仍在。持有者被終止後，第一次嘗試就取得鎖定；鎖定檔沒有被刪除 |
| ASM-21 | Linux 與 macOS 上，以 `O_NONBLOCK` 開啟 FIFO 或裝置檔不會阻塞 | POSIX `open`：以 `O_RDONLY \| O_NONBLOCK` 開啟 FIFO 立即返回；支援非阻塞開啟的裝置檔也立即返回 | 只在判定類型與開檔之間被換成特殊檔案時才會遇到；開啟後以 handle 判定類型，不是一般檔案就不讀取（§6.3）。pilot 沒有測試裝置檔 | 通過。Linux 與 macOS 以 `O_NOFOLLOW \| O_NONBLOCK` 開啟 FIFO 都立即返回，以 handle 判定為其他類型。對照組：不加 `O_NONBLOCK` 時開啟會阻塞 |

### 7.8 文件出處

- POSIX.1-2024：[`rename`](https://pubs.opengroup.org/onlinepubs/9799919799/functions/rename.html)、[`link`／`linkat`](https://pubs.opengroup.org/onlinepubs/9799919799/functions/link.html)、[`open`／`openat`](https://pubs.opengroup.org/onlinepubs/9799919799/functions/open.html)、[`unlink`／`unlinkat`](https://pubs.opengroup.org/onlinepubs/9799919799/functions/unlink.html)、[`<sys/stat.h>`](https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_stat.h.html)
- Linux：[`rename(2)`](https://man7.org/linux/man-pages/man2/rename.2.html)、[`flock(2)`](https://man7.org/linux/man-pages/man2/flock.2.html)
- macOS：[`fsync(2)`](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fsync.2.html)、[`flock(2)`](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/flock.2.html)、[`rename(2)`（xnu 原始碼）](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/rename.2)、[QA1118 Statically linked binaries on Mac OS X](https://developer.apple.com/library/archive/qa/qa1118/_index.html)
- Windows：[`MoveFileExW`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw)、[`CreateFileW`](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew)、[`GetFinalPathNameByHandleW`](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfinalpathnamebyhandlew)、[`FILE_ID_INFO`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_id_info)、[`BY_HANDLE_FILE_INFORMATION`](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/ns-fileapi-by_handle_file_information)、[`FILE_DISPOSITION_INFO`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_disposition_info)、[`FILE_LINK_INFORMATION`](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_link_information)、[`FILE_RENAME_INFORMATION`](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_rename_information)、[`CreateSymbolicLinkW`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createsymboliclinkw)、[Create symbolic links 使用者權限](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/security-policy-settings/create-symbolic-links)、[`CreateDirectoryW`](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createdirectoryw)、[`GetSecurityInfo`](https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-getsecurityinfo)、[`SetSecurityInfo`](https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-setsecurityinfo)
- Rust：[`std::env::home_dir`](https://doc.rust-lang.org/std/env/fn.home_dir.html)

## 8. 可測試性設計

本節說明測試如何替換 rosamta 的外部依賴、在指定的時間點改動檔案，以及注入失敗。04 §5 的情境與 06 的測試，都以這些機制撰寫。

### 8.1 可替換的依賴

core 的行為只取決於下列依賴，每一項都由呼叫端傳入（§3.1），測試可以換成自己的版本：

| 依賴 | 正式執行 | 測試 |
| --- | --- | --- |
| `HostContext`：作業系統、主機名稱、home、環境變數 | 執行檔啟動時收集（§3.3、§6.1） | 直接建構，home 指向測試建立的暫存目錄 |
| 目前時間，用於批次編號（§5.1） | 系統時間 | 固定的時間，用來重現批次編號重複時加上 `-2` 的情況 |
| `FileSystem` | platform 的實作 | platform 的實作；需要注入失敗時，以 testkit 的 `FaultFs` 包裝（§8.3） |
| `SyncHook` | 什麼都不做的實作 | testkit 的 `SyncScript`，在指定的同步點執行注入（§8.2） |
| `GitClient` | 呼叫 `git`（§3.3） | 從本機的測試 repo clone，或回傳指定錯誤的假實作 |

執行檔的測試以子程序執行 rosamta，透過 `HOME`（Windows 為 `USERPROFILE`）與 `XDG_*` 環境變數把 home 與三個程式目錄指向暫存目錄（§6.1）。標準輸出與標準錯誤接到 pipe，所以不是終端機（IF-009）。

不提供存在記憶體中的假 `FileSystem`。要驗證的正是平台的行為：連結、改名、共用模式與權限，假實作只會驗證假實作本身。純邏輯不需要檔案系統：§5.11 的判定以觀測與紀錄為輸入，單元測試直接建構觀測值；路徑正規化、設定檔與狀態檔的解析也一樣。

### 8.2 同步點

同步點是 core 呼叫 `SyncHook` 的位置，用來在指定的時間點讓「外部程式」改動檔案。每次呼叫傳入同步點、操作種類，以及目標在 home 內的相對路徑：

| 同步點 | 位置 |
| --- | --- |
| `AfterPreflight` | 預檢通過，執行任何操作之前；每個批次一次 |
| `BeforeCapture` | §6.3 步驟 1、2 通過，開檔擷取快照、製作備份，或讀取狀態檔之前 |
| `AfterCapture` | 副本已寫好，與觀測的版本比較之前 |
| `BeforeOperation` | 最後一次核對通過，執行建立、取代、刪除、變更權限或建立目錄的系統呼叫之前；Windows 此時仍持有目錄與目標的 handle |
| `AfterOperation` | 系統呼叫完成，§6.3 步驟 4 的重檢之前 |
| `BeforeRecheck` | 所有項目都處理完，§6.6 步驟 3 之前 |
| `BeforeCommit` | §6.6 步驟 3 通過，提交狀態之前 |
| `BeforeCompletion` | 狀態已提交或無須提交，§6.6 步驟 5 之前 |
| `BeforeRollback` | 決定回復後，處理第一個項目之前 |

操作種類分成四組：狀態檔的載入、擷取、提交與回復；目標的擷取快照、製作備份、建立、取代、刪除、變更權限、建立目錄；回復時的刪除、取代、建立、變更權限與刪除目錄；管理與初始化的 manifest 與本機設定修改、來源副本的建立、來源目錄的移入。

`BeforeOperation` 必須落在最後一次核對與系統呼叫之間，所以 `FileSystem` 的每個操作分成兩次呼叫：第一次完成走訪與核對，回傳持有 handle 的物件；第二次執行系統呼叫，以及交換或移開後的核對（§6.3）。介面在 §4 定義。§6.3 所說的剩下的空窗，位於第二次呼叫的內部，同步點到不了。

同步點只放在 core，platform 不知道測試的存在。正式執行傳入什麼都不做的 `SyncHook`，每個同步點多一次函式呼叫，成本可以忽略。不用編譯期的 feature 開關：那樣測試的程式就不是發佈的程式。

testkit 的 `SyncScript` 讓測試以「同步點、操作種類、路徑、第幾次」指定時間點，並登記要執行的注入，例如以 `std::fs` 替換父目錄、就地修改目標或換入 FIFO。`SyncScript` 記錄每一個到達的同步點，以及每次注入是否成功：

- 登記的同步點沒有到達時，測試失敗。04 §5 說「未到達同步點時重測」；同步點是確定的，沒有到達表示測試寫錯，而不是運氣不好。
- 注入被平台擋下時（例如 Windows 的 `ERROR_SHARING_VIOLATION`），測試確認擋下的是這次注入，並核對原操作依計畫完成（04 §5）。

注入在同一個程序內執行。Windows 的共用模式作用在 handle 之間，與是否同一個程序無關；Linux 與 macOS 的改名與刪除也不區分程序，所以結果與真正的外部程式相同。需要另一個程序持續改動的壓力測試（如 pilot P1 的 300 輪），由 testkit 另外啟動子程序。

### 8.3 故障注入

testkit 的 `FaultFs` 包裝任一 `FileSystem` 實作，把呼叫轉給被包裝的實作；符合條件（操作種類、路徑、第幾次）的呼叫改為回傳指定的錯誤，例如權限不足、空間不足或 I/O 錯誤。用途：

| 用途 | 注入方式 | 04 §5 情境 |
| --- | --- | --- |
| 中途寫入失敗 | 第 N 個目標的建立或取代回傳權限不足。另外以移除目錄寫入權限的方式實測一次，驗證平台的實際錯誤（FR-APL-002 的驗收方式） | A6 |
| 狀態提交失敗、提交後的寫入確認失敗 | 狀態檔的取代，或 §6.6 步驟 5 讀取狀態檔時回傳錯誤 | A6、A20、A23、A25 |
| 部分回復失敗 | 回復時某一項的操作回傳錯誤 | A6 |
| 空間不足 | 暫存檔或快照的寫入回傳 `ENOSPC`、`ERROR_DISK_FULL`；06 另以容量有限的檔案系統實測 | A8 |
| 停止管理的兩次寫入之間失敗 | manifest 的取代回傳錯誤 | A5 |

`FaultFs` 只模擬呼叫失敗。外部程式的改動用同步點（§8.2），因為它必須真的改變檔案系統，§6.3 的核對才有東西可以發現。程序中止（REL-001、REL-002、A6、A13）也不用 `FaultFs`：如同 pilot P2，執行檔的測試啟動 rosamta，在隨機的時間點終止它。

### 8.4 測試的位置

| 位置 | 內容 | 使用 testkit |
| --- | --- | --- |
| core 的單元測試（`#[cfg(test)]`） | 純邏輯：路徑正規化、位置鍵的比較、§5.11 的判定、設定檔與狀態檔的解析、範本的引用分析 | 否 |
| core 的 integration test（`rosamta-core/tests/`） | 服務搭配真實的檔案系統：觀測與綁定、執行與回復，以及以同步點與故障注入撰寫的情境 | 是 |
| platform 的測試 | 每個 `ASM-NN` 的平台行為，延續 pilot 的測試 | 否 |
| 執行檔的測試（`rosamta/tests/`） | 子指令、輸出格式、exit code、程序中止 | 是，用來建立測試用的 home |

core 的單元測試不能使用 testkit。testkit 依賴 core；core 的單元測試若把 testkit 列為 dev-dependency，Cargo 會編出兩份 core，testkit 回傳的型別與受測的 core 型別不同，trait 的實作對不上。integration test 把 core 當成一般的函式庫連結，只有一份 core，所以注入故障的測試都放在 integration test 或執行檔的測試。

三平台的 CI 沿用 pilot 的設定：Linux 與 macOS 以一般使用者執行，Windows 另建標準使用者，並在開發人員模式開啟與關閉時各跑一次（§7.1）。

### 8.5 04 §5 情境與同步點

下表列出 04 §5 中指定了時間點的情境，以及對應的同步點（括號內為操作種類）。「預期」欄是依 §6 的處理方式，06 依此撰寫驗收條件；各情境共同的部分（回傳 `1`、回復其他可安全處理的項目、逐項回報）不重複寫出。

| 情境 | 04 §5 的時間點 | 同步點 | 預期 |
| --- | --- | --- | --- |
| A10 | 預檢剛通過 | `AfterPreflight` | 走訪時父目錄的身分不符，不執行操作 |
| A10 | 最後一次路徑檢查通過，尚未寫入或刪除 | `BeforeOperation`（建立、取代、刪除） | Windows 擋下替換；Linux 與 macOS 的操作落在原目錄，步驟 4 發現後經由 handle 復原（ASM-06） |
| A10 | 回復的最後一次檢查通過，實際操作前 | `BeforeOperation`（回復的各操作） | 同上；該項保留並回報 |
| A14 | 重檢前 | `BeforeRecheck` | 提交前重檢不符，不提交狀態 |
| A14 | 最後一次目標重檢後，狀態替換前 | `BeforeCommit` | 狀態照常提交，完成確認發現不符，回復原狀態 |
| A15 | 最後一次確認位置為空後，實際建立前 | `BeforeOperation`（建立） | 不覆蓋的建立失敗，外部檔案保留 |
| A15 | 狀態提交前替換本批建立的檔案 | `BeforeCommit` | 完成確認發現身分不符，該檔案不動，回復原狀態 |
| A16 | 擷取快照或製作備份的最後一次路徑檢查後，開檔前替換父目錄 | `BeforeCapture`（擷取快照、製作備份） | Windows 擋下替換；Linux 與 macOS 經由持有的目錄讀到原檔，擷取後的步驟 4 發現路徑改變，不使用副本 |
| A16 | 擷取時改動原檔，到擷取或核對結束才改回 | `BeforeCapture` 改動，操作結束後改回 | 讀到的內容與觀測的版本不符 |
| A16 | 擷取時改動，擷取完成後、核對副本前改回 | `BeforeCapture` 改動，`AfterCapture` 改回 | 比較的是實際讀到的內容，仍然不符 |
| A16 | 回復前替換或破壞某項快照 | `BeforeRollback` | 快照的 SHA-256 不符，不使用；該項保留並回報 |
| A17 | 寫入完成後，第一次依路徑查驗身分前 | `AfterOperation`（建立、取代） | 步驟 4 發現身分不符，替換進來的檔案不動 |
| A17 | 最後一次目標重檢後，狀態替換前 | `BeforeCommit` | 完成確認發現不符 |
| A17 | 最後一次重檢後，判定完成前（不提交狀態） | `BeforeCompletion` | 完成確認發現不符 |
| A17 | 最後一次狀態重檢後，判定完成前移除紀錄 | `BeforeCompletion` | 完成確認發現狀態檔不是首次載入的版本，保留外部的改動 |
| A18 | 最後一次檢查後換入特殊檔案 | `BeforeOperation`（取代、刪除） | Windows 擋下替換；Linux 與 macOS 交換或移開後發現類型不符，移回原位保留 |
| A19 | 預檢後、重檢前就地修改 | `AfterPreflight` | 步驟 2 版本不符 |
| A19 | 最後一次內容核對後，實際清除或覆寫前 | `BeforeOperation`（取代、刪除） | Windows 擋下寫入（ASM-18）；Linux 與 macOS 交換或移開後發現版本不符，換回原位，保留修改 |
| A20 | 首次載入、擷取原狀態的最後一次路徑檢查後 | `BeforeCapture`（狀態檔的載入、擷取） | 同 A16 的第一列，替代位置的狀態檔不會被讀取 |
| A20 | 提交的最後一次路徑檢查後 | `BeforeOperation`（狀態檔的提交） | 同 A10 的第二列 |
| A20 | 提交後寫入確認失敗，回復的最後一次路徑檢查後 | `FaultFs` 使完成確認失敗，再於 `BeforeOperation`（狀態檔的回復）替換 | 同 A10 的第三列；回報原狀態是否已還原 |
| A21 | 回復重檢前就地修改 | `BeforeRollback` | 回復前的核對不符，保留修改 |
| A21 | 最後一次內容核對後，實際回復前 | `BeforeOperation`（回復的刪除、取代） | Windows 擋下；Linux 與 macOS 移開或交換後發現，換回原位，保留修改 |
| A22 | 完成清除後、重檢前 | `BeforeRecheck` | 提交前重檢發現目標存在 |
| A22 | 最後一次目標重檢後，狀態替換前 | `BeforeCommit` | 完成確認發現目標存在，外部檔案保留 |
| A22 | 回復最後一次確認路徑為空後，還原舊檔前 | `BeforeOperation`（回復的建立） | 不覆蓋的建立失敗，外部檔案保留，回報該路徑無法回復 |
| A23 | 提交的最後一次內容核對後，替換前 | `BeforeOperation`（狀態檔的提交） | Windows 擋下；Linux 與 macOS 交換後發現版本不符，換回原狀態檔 |
| A23 | 回復的最後一次內容核對後，回寫前 | `FaultFs` 使完成確認失敗，再於 `BeforeOperation`（狀態檔的回復）修改 | 同上，保留外部的修改，回報狀態無法回復 |
| A25 | 擷取原狀態時改寫，核對前改回 | `BeforeCapture`（狀態檔的擷取）改寫，`AfterCapture` 改回 | 同 A16 的第三列，不改動任何目標 |
| A25 | 提交後寫入確認失敗，以遭改動的副本回復 | `FaultFs` 使完成確認失敗，`BeforeRollback` 改動副本 | 副本的 SHA-256 不符，保留現況，回報狀態無法回復 |
| A26 | 最後一次目標重檢後，批次完成前（須寫入新狀態） | `BeforeCommit` 或 `BeforeCompletion` | 完成確認發現不符，回復原狀態 |
| A26 | 同上（全批免寫狀態） | `BeforeCompletion` | 完成確認發現不符 |

不依同步點的情境：A5 停止管理的兩次寫入之間以 `FaultFs` 注入失敗，程序中止則以執行檔的測試終止；A5 最後一個情境在 `AfterPreflight` 執行停止管理，依 §6.7 會因取不到鎖定而失敗。A7、A12 用 `BeforeOperation`（來源目錄的移入、來源副本的建立、manifest 與本機設定的修改）；A7 的「下載途中才被占用」由假的 `GitClient` 在 clone 期間建立檔案。A24 在 `AfterPreflight` 與第一個項目的 `AfterOperation` 替換來源；執行期參照無法取得內容的情境，在 §4 定義參照後補上。A6、A13 的程序中止與 A8 見 §8.3。

A16 第三列的預期依 §6.5 的快照方式而定：快照是邊讀邊計算的複製，核對的是讀到的位元組，所以「改回」不會讓錯誤的副本通過。若改用 hard link 當快照，副本就是原檔本身，改回之後副本是正確的版本，這一列的預期就不成立；這是 §6.5 不用 hard link 當快照的另一個理由。
