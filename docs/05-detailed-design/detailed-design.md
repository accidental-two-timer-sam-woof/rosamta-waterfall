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

## 3. 套件結構

本節把 03 C1–C4 的類別分配到 crate 與 module，並規定依賴方向。圖見 [uml.md](uml.md) 的 P1、P2；各類別的公開介面見 §4。

### 3.1 crate 劃分

程式以 Cargo workspace 分成四個 crate：

| crate | 種類 | 內容 | 依賴的 workspace crate | 外部 crate |
| --- | --- | --- | --- | --- |
| `rosamta` | 執行檔 | 命令列介面（§9）。啟動時收集主機資訊，建立 platform 的實作，交給 core 的服務 | core、platform | 參數解析與 diff 的套件在 §9 選定；serde_json |
| `rosamta-core` | 函式庫 | 03 的全部服務與資料型別，以及 `FileSystem`、`GitClient` 兩個 trait | 無 | toml_edit、minijinja、serde、serde_json、sha2 |
| `rosamta-platform` | 函式庫 | `FileSystem` 的 Unix 與 Windows 實作、`GitClient` 的實作（呼叫 `git` 執行檔），以及主機資訊的收集 | core | rustix（Unix）、windows-sys（Windows） |
| `rosamta-testkit` | 函式庫，只作 dev-dependency | 可注入故障與同步點的 `FileSystem`、建立測試用 home 的工具（§8） | core、platform | — |

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
| 0 | `fs` | `FileSystem`、`FileInspection` | trait，以及它使用的檔案身分、類型、內容指紋與目錄 handle | `diag`、`path` |
| 0 | `layout` | — | 程式使用的位置（§5.1）與保留位置（§5.2） | `diag`、`host`、`path` |
| 1 資料 | `config` | `ConfigLoader`、`EffectiveConfig`、`Manifest`、`OutputDefinition`、`Module`、`Field`、`Profile`、`ProfileRule`、`LocalSettings`、`Template` | 讀取四種設定檔與內建資源，合併內建輸出與覆寫（§5.3–§5.9） | 第 0 層 |
| 1 | `docedit` | — | 保留格式的 TOML 編輯（§5.3） | `diag` |
| 1 | `resolve` | `ValueResolver`、`ResolvedValues`、`ValueSource` | 四層覆寫與值的來源 | 第 0 層、`config` |
| 1 | `template` | — | MiniJinja 環境、範本 loader、靜態引用、原樣複製來源的讀取 | 第 0 層、`config` |
| 1 | `state` | `StateStore`、`StateFile`、`ManagedOutputRecord`、`ComparisonMetadata`、`StateLoadResult` | 狀態檔的格式、載入與原子提交（§5.10） | 第 0 層 |
| 2 流程 | `plan` | `Planner`、`ApplyPlan`、`PlanItem` | 計畫與比較基準的判定（§5.11） | 第 0 層、`config`、`resolve`、`template`、`state` |
| 2 | `preflight` | `PreflightChecker`、`AnalysisContext` | 整批預檢 | 第 0 層、`config`、`resolve`、`template`、`state`、`plan` |
| 2 | `recovery` | `RecoveryManager`、`RecoveryReport` | 回復快照的保存與還原 | 第 0 層、`state` |
| 2 | `exec` | `Executor`、`ApplyReport`、`ItemResult` | 執行計畫、製作接管備份（§5.12） | 第 0 層、`state`、`plan`、`recovery` |
| 3 子指令 | `apply` | `ApplyService`、`ApplyRequest` | 預覽、摘要與套用的流程 | 第 0 層、`config`、`resolve`、`template`、`state`、第 2 層 |
| 3 | `manage` | `ManifestService`、`OutputPropertyPatch`、`OperationResult` | 納入、修改屬性、停止管理（§5.13） | 第 0 層、`config`、`docedit`、`state` |
| 3 | `query` | `QueryService`、`OutputSummary`、`OutputDescription`、`ResolvedValueDescription` | 引用、輸出與欄位值的查詢 | 第 0 層、`config`、`resolve`、`template` |
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

`[modules.<模組>]` 的 `version` 是模組格式版本，供範本宣告相容版本（FR-TPL-004、§5.9）；未寫時沒有版本，宣告了相容版本的範本引用這個模組時，預檢失敗。模組名稱不得與內建模組相同，也不得與範本的全域名稱相同（`host` 與 §4 註冊的函式）。

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

### 5.6 內建模組的欄位

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
| `terminal` | `scrollback` | integer | 否 | 0–1000000 | — | 捲動行數 |
| `terminal` | `keybindings` | map，名稱為 `word`，值為 enum | 否 | 名稱是按鍵組合，例如 `ctrl+shift+c`；值為 `copy`、`paste`、`new_tab`、`close_tab`、`next_tab`、`previous_tab`、`increase_font_size`、`decrease_font_size`、`reset_font_size` | — | 快捷鍵 |

`ansi` 的預設值依序為 `#000000`、`#cd0000`、`#00cd00`、`#cdcd00`、`#0000ee`、`#cd00cd`、`#00cdcd`、`#e5e5e5`、`#7f7f7f`、`#ff0000`、`#00ff00`、`#ffff00`、`#5c5cff`、`#ff00ff`、`#00ffff`、`#ffffff`；`foreground` 與 `background` 取其中第 7 與第 0 色。

必填欄位都有預設值（DATA-003）。沒有預設值的欄位都是選填，未設定時內建範本不輸出對應的設定，讓程式使用自己的預設值（DATA-004）。字型名稱與字級沒有跨平台都適用的預設值，因此是選填。快捷鍵的值是不分程式的動作名稱，由各內建範本轉成目標程式的寫法，同一份設定才能套用到不同的終端機。

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

## 7. 平台假設清單

本節列出設計依賴、而 Rust 標準函式庫不保證的作業系統行為。每項假設以 `ASM-NN` 編號，列出官方文件依據、文件保證的範圍，以及 pilot 結果（`prototypes/pilot/`）。§6 的演算法只依賴本節列出的行為；06 為每個 `ASM-NN` 安排測試情境（§13）。

pilot 結果分四種：

- 通過：官方文件有保證，pilot 在三平台的結果也相符。
- 有條件：pilot 在三平台都相符，但相關行為官方文件沒有寫明，或須由設計另外確保某個前提。條件寫在「保證範圍與條件」欄。
- 未驗證：pilot 無法重現這個行為，例如需要重新開機。設計須在假設不成立時仍不會誤寫或誤刪，並寫明由 06 如何驗證。
- 不可行：這個做法無法滿足需求，須改變設計或修訂 SRS（04 §3.3 末段）。

本版沒有「不可行」的項目，「未驗證」只有 ASM-15。P4、P5 驗證的是選定套件的行為，不屬於平台假設，結果見 §2.2、§2.3；兩者在三平台都通過。

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

### 7.6 跨次執行的檔案身分

ASM-04 只在同一批次內比較檔案身分。狀態檔另外記錄目標父目錄的身分（§5.10），供下一次套用確認父目錄沒有被替換（04 §3.1），因此依賴下列假設。

| 編號 | 假設 | 依據 | 保證範圍與條件 | pilot 結果 |
| --- | --- | --- | --- | --- |
| ASM-15 | 目錄存在期間，它的檔案編號跨程序、跨重新開機都不變：Linux 與 macOS 的 `st_ino`，Windows 的 volume 序號與 128 位元 file ID。狀態檔不記錄 Linux 與 macOS 的 `st_dev` | `BY_HANDLE_FILE_INFORMATION` 文件：NTFS 上的檔案在刪除前保持同一個 file ID。POSIX `<sys/stat.h>` 只規定 `st_dev` 加 `st_ino` 在系統內唯一，沒有規定重新掛載後不變 | `st_dev` 由核心在掛載時配給，例如 btrfs subvolume 使用匿名裝置編號，重新開機後可能不同，所以不記錄。不比較 `st_dev` 時，不同檔案系統上的目錄可能編號相同；目錄刪除後，編號也可能被新目錄重複使用（ASM-04）。這兩種誤判都會把父目錄當成沒有被替換，但 §5.11 仍要求內容符合紀錄的實際基準才會覆寫或刪除，所以被改動的內容與 rosamta 上次寫入的相同。反過來，編號若意外改變，紀錄判為位置無法確認：啟用的輸出改按未受管理處理，清除時保留檔案，不會誤寫或誤刪 | 未驗證。CI 無法重新開機。本機 btrfs 的 home 裝置編號為 0:47，major 為 0，是匿名裝置。06 在實機上於重新開機前後各套用一次，確認沒有紀錄被判為位置無法確認 |

### 7.7 文件出處

- POSIX.1-2024：[`rename`](https://pubs.opengroup.org/onlinepubs/9799919799/functions/rename.html)、[`link`／`linkat`](https://pubs.opengroup.org/onlinepubs/9799919799/functions/link.html)、[`open`／`openat`](https://pubs.opengroup.org/onlinepubs/9799919799/functions/open.html)、[`unlink`／`unlinkat`](https://pubs.opengroup.org/onlinepubs/9799919799/functions/unlink.html)、[`<sys/stat.h>`](https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_stat.h.html)
- macOS：[`fsync(2)`](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fsync.2.html)、[QA1118 Statically linked binaries on Mac OS X](https://developer.apple.com/library/archive/qa/qa1118/_index.html)
- Windows：[`MoveFileExW`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw)、[`CreateFileW`](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew)、[`GetFinalPathNameByHandleW`](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfinalpathnamebyhandlew)、[`FILE_ID_INFO`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_id_info)、[`BY_HANDLE_FILE_INFORMATION`](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/ns-fileapi-by_handle_file_information)、[`FILE_DISPOSITION_INFO`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_disposition_info)、[`FILE_LINK_INFORMATION`](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_link_information)、[`FILE_RENAME_INFORMATION`](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_rename_information)、[`CreateSymbolicLinkW`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createsymboliclinkw)、[Create symbolic links 使用者權限](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/security-policy-settings/create-symbolic-links)
