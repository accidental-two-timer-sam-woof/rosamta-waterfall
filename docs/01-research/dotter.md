# 前期研究：Dotter

| 項目 | 內容 |
| --- | --- |
| 專案 | [SuperCuber/dotter][repo] |
| 定位 | 以 package、共用變數與範本管理 dotfile，支援符號連結與檔案產生 |
| 技術 | Rust、TOML、Handlebars |
| 授權 | Unlicense |
| 程式版本 | v0.13.5；tag commit `7dec24fea4f6516a014f1a3ea51491e23d9005d4` |
| Wiki 版本 | `7f111da392bd60385be9e79d08c0178d5693afc4` |
| 實測環境 | Windows，無符號連結建立權限；官方 `dotter-windows-x64-msvc.exe` v0.13.5；目標皆為臨時目錄 |
| 日期 | 2026-11-04 |

## 1. Dotter 解決的問題

把 dotfile 放進 Git，再以符號連結接到各程式的設定路徑，可以保留版本紀錄，但檔案一多，就得另外記錄來源與目標的對應。換一台機器時，連結要重新建立；遇到不同作業系統、不同螢幕或不同軟體組合，同一份設定也未必適用。Dotter 將檔案對應、套用範圍與變數放進 TOML，讓這些差異可以跟著設定檔一起維護。[專案說明][repo]

它有兩種輸出方式：內容不需改變時建立符號連結；內容需依變數調整時，將來源當作 Handlebars 範本，產生一般檔案。多個範本可引用同一組變數，因此配色、字型等資料不必在每個程式的設定中各存一份。[連結與範本][templates]

這與 rosamta 的模組概念有直接重疊。Dotter 已能用 package 共用資料、用範本轉換各程式的格式；rosamta 的差異需要落在模組欄位的定義、驗證與內建範本，而不能只以「一份資料產生多個設定檔」區分兩者。

## 2. 設定的組成與邊界

### Package 是部署單位

| 欄位 | 用途 | 邊界 |
| --- | --- | --- |
| `files` | 將 repo 中的來源路徑對應到目標路徑 | 對應表以來源為 key，不是以目標為 key |
| `variables` | 提供範本資料 | 啟用後進入共用的變數空間，不自動加上 package 名稱 |
| `depends` | 宣告需要一併啟用的 package | 相依會遞迴展開；不是部署工作的先後順序 |

這三個欄位均可省略。例如 `[theme.variables]` 定義 `foreground` 後，其他 package 的範本使用 `{{foreground}}`，不是 `{{theme.foreground}}`。需要 `{{palette.foreground}}` 這種命名時，必須自行在變數中建立 `palette` 表格。[Package 文件][packages]、[設定結構與合併實作][src-config]

相依展開採集合累加，直到不再出現新的 package。指定不存在的 package 或相依項目會在取得 package 資料時報錯。循環相依不會形成無限遞迴，但也不會產生依賴排序：`depends` 用來選取 package，不能保證 A 的檔案先於 B 寫入。[設定實作][src-config]

### 共用設定與本機設定

| 檔案 | 內容 |
| --- | --- |
| `.dotter/global.toml` | Package、共用變數、檔案對應、helpers 與全域設定 |
| `.dotter/local.toml` | 本機選擇的 package、includes、檔案與變數覆寫 |
| Includes 指向的 TOML | 依 package 名稱補充或覆寫設定，例如 Windows 的輸出路徑 |
| `.dotter/<hostname>.toml` | 預設 local 設定不存在時的替代檔案 |
| `.dotter/cache.toml` | 已部署的來源與目標對應，區分符號連結與範本 |
| `.dotter/cache/` | 渲染後的範本內容等快取資料 |

Hostname 選擇是 fallback：只有指定的 local 設定檔不存在時，才改找同目錄下的 `<hostname>.toml`。它不會把多份符合條件的主機規則自動疊加，也不能直接等同 rosamta 規劃的 OS、hostname、標籤比對。[其他功能][misc]、[設定實作][src-config]

範本另可讀取 `dotter.windows`、`dotter.linux`、`dotter.macos`、`dotter.unix` 等平台布林值，以及 `dotter.packages`、`dotter.current_dir`。取得主機名稱成功時也會提供 `dotter.hostname`；這些值是範本資料，不會自行產生一套 Profile 優先權規則。[內建變數實作][src-helpers]

來源路徑與 includes 路徑依執行時的工作目錄解讀，不能假設一律相對於 TOML 所在目錄。一般做法是在 dotfiles repo 根目錄執行 Dotter。目標路徑另外支援 `~` 與環境變數展開。[入門文件][getting-started]、[設定實作][src-config]

### 多份 repo 可以分別部署，但不會自動互相切換

多份 dotfiles repo 可以放在不同目錄，分別進入各自的根目錄執行 `dotter deploy`。使用預設路徑時，每份 repo 讀取自己的 `.dotter/global.toml`、`.dotter/local.toml` 與 cache；切換工作目錄即可選擇這次部署採用哪份設定。[參數預設值][src-args]、[設定載入][src-config]

部署 repo B 不會自動撤除 repo A 的輸出。若兩者管理不同目標，兩組輸出可以並存；若指向同一目標，B 可能因目標已存在或內容不符合自己的 cache 而略過。兩份獨立的 cache 不會共同協調該目標的歸屬，強制覆寫也不會同步更新另一份 repo 的追蹤狀態。[部署與撤除][src-deploy]、[建立與更新檢查][src-actions]

若要用 B 完整取代 A，可先在 A 執行 `dotter undeploy`，確認撤除成功且沒有因衝突留下的輸出，再進入 B 部署。撤除依 A 的 cache 處理全部已追蹤輸出，不只處理與 B 重疊的檔案；兩次命令也不是同一筆交易，B 失敗時不會自動恢復 A。只想更換部分設定時，使用同一 repo 的 package 選取與 local 覆寫，通常比讓多份 repo 反覆接管相同目標更容易追蹤。[部署與撤除][src-deploy]、[Package 設定][packages]

## 3. 一份配色如何供兩個程式使用

```text
dotfiles/
├── .dotter/
│   ├── global.toml
│   └── local.toml
└── templates/
    ├── alacritty.toml
    └── tmux.conf
```

`.dotter/global.toml`：

```toml
[theme.variables]
palette = { foreground = "#d8dee9", background = "#2e3440" }

[terminal]
depends = ["theme"]

[terminal.files]
"templates/alacritty.toml" = { target = "~/.config/alacritty/alacritty.toml", type = "template" }

[tmux]
depends = ["theme"]

[tmux.files]
"templates/tmux.conf" = { target = "~/.tmux.conf", type = "template" }
```

`.dotter/local.toml`：

```toml
packages = ["terminal", "tmux"]
```

`templates/alacritty.toml`：

```handlebars
[colors.primary]
foreground = "{{palette.foreground}}"
background = "{{palette.background}}"
```

`templates/tmux.conf`：

```handlebars
set -g status-style "fg={{palette.foreground}},bg={{palette.background}}"
```

選取 `terminal` 與 `tmux` 會一併啟用 `theme`。兩個範本讀取同一份 `palette`，但輸出各自需要的語法。這些片段只涵蓋配色，並不是完整的應用程式設定。[Package 文件][packages]

若只想讓某台主機使用白色前景，可在 local 設定加入：

```toml
[variables.palette]
foreground = "#ffffff"
```

`palette` 以表格方式遞迴合併，因此只覆寫 `foreground`，`background` 仍保留 `#2e3440`。實測將兩個目標改到臨時目錄後，首次部署均使用 `#d8dee9`；加入 local 覆寫後，兩份輸出均改為 `#ffffff`，背景值不變。

相同資料能產生兩種格式，靠的是使用者維護兩個範本。Dotter 對 package、local 與 complex target 的設定結構會拒絕未知欄位，但自訂變數沒有「合法色碼」「應用程式支援哪些欄位」或「透明度範圍」等領域規格。設定結構檢查與應用程式資料驗證是不同的層次。[設定型別與反序列化][src-config]

## 4. 合併規則不能只看成「後面的蓋掉前面的」

### 4.1 設定層級

| 階段 | 行為 |
| --- | --- |
| Global | 載入全部 package |
| Includes | 按清單順序修補 package；較後的 include 可覆寫較前的值 |
| Package 選取 | 保留 local 指定的 package 與相依項目 |
| Package 間合併 | 將啟用的檔案與變數合併，適用獨立的衝突規則 |
| Local | 覆寫合併後的檔案與變數 |
| Manual patch | `--patch` 從標準輸入讀取額外設定，優先於 local |
| 停用檔案 | 移除目標值為空字串 `""` 的對應 |

Includes 與 local 的變數覆寫會遞迴處理表格；遇到非表格值時，以新值取代舊值。陣列不採逐項合併。`--patch` 也遵循變數的遞迴覆寫方式。[合併實作：`recursive_extend_map`、`merge_configuration_files`][src-config]

### 4.2 不同 package 的衝突

| 情況 | v0.13.5 的處理 |
| --- | --- |
| 兩個 package 宣告相同來源路徑 | 報錯，即使目標值相同也不直接接受 |
| 兩個 package 宣告相同頂層變數，且不都是表格 | 報錯，例如兩邊都宣告純量 `font_size` |
| 兩個 package 宣告相同頂層變數，且都是表格 | 遞迴合併 |
| 上述表格內部又有同名欄位 | 後合併的值覆寫先前的值，不再以同樣規則報錯 |
| 不同來源路徑指向同一目標 | 不由來源 key 的重複檢查攔下，可能到部署時才發生衝突 |

Package 存在 `BTreeMap` 中，合併依 package 名稱排序，不依 `local.toml` 中 `packages` 的列出順序。實測設定 `packages = ["b", "a"]`，兩者均定義 `palette.x`，結果仍由 `b` 的值覆寫 `a`；`a` 的其他表格欄位則保留。[設定實作][src-config]

因此，「不同 package 的變數衝突一定報錯」不夠精確。頂層純量衝突與表格內部欄位衝突，適用不同規則。若以 package 名稱控制勝出的配色，改名就可能改變結果，也容易讓相依關係與優先權混在一起。

### 4.3 Includes 對相依的限制

Includes 修補既有 package 時，程式只合併 `files` 與 `variables`，不把 include 內的 `depends` 合入既有 package；include 新增的整個 package 則會保留其相依欄位。[設定實作][src-config]

例如 global 已有 `app`，include 再寫 `[app] depends = ["theme"]`，不能據此認定 `theme` 會被啟用。實測只選取 `app` 時，引用 `theme` 變數的範本仍因缺少變數而失敗。相依宣告應放在原始 package 定義，避免將 includes 視為所有欄位都能通用覆寫的機制。

## 5. 輸出型態與範本的責任

### 符號連結與範本不是相同的編輯方式

| 項目 | 符號連結 | 範本輸出 |
| --- | --- | --- |
| 來源與目標內容 | 指向同一份內容 | 來源含範本語法，目標為渲染結果 |
| 修改來源 | 目標隨之改變 | 需重新部署或由 watch 觸發更新 |
| 修改目標 | 可能直接修改 repo 中的來源 | 修改的是產生後的檔案，可能被視為部署衝突 |
| 主機差異 | 通常透過選取不同來源或路徑處理 | 可由變數、條件與 helpers 處理 |

自動模式以來源是否含 `{{` 判斷範本；無法讀成 UTF-8 的來源會判為符號連結。檔案若只是含有字面上的 `{{`，也可能被誤判，因此可用 complex target 明確指定 `type = "symbolic"` 或 `type = "template"`。全域另有 `settings.default_target_type`。[輸出判定實作][src-fs]、[Complex Target][targets]

目錄預設會遞迴展開為個別檔案對應；symbolic target 可用 `recurse = false` 保留整個目錄的連結方式。整個目錄連結與逐檔部署的管理範圍不同，不能視為純粹的效能選項。[目錄展開實作][src-config]

### 範本不會自動保證輸出格式有效

Handlebars 使用 strict mode，缺少變數會報錯；HTML escaping 被停用，避免一般設定值被轉成 HTML entity。不過，字串若包含引號、換行或反斜線，仍需由範本正確處理目標格式，不能因為渲染成功就認定 TOML、shell 或其他設定語法有效。[範本初始化][src-helpers]

`append` 與 `prepend` 加在來源內容的前後，然後才一起渲染。它們不是把文字附加到目標原有內容，也不是對既有設定檔進行欄位合併。實測來源為 `middle`，前後各加入含 `{{x}}` 的片段，`x = "value"` 時得到 `value/middle/value`。[範本動作][src-config]、[渲染與寫入][src-actions]

檔案的 `if` 條件使用範本的判斷語意，可依變數或 helper 決定是否納入部署。條件轉為 false 後，檔案不只是不再更新：若 cache 記錄它曾經部署，目標也可能進入清除流程。實測從 true 改成 false，未被手動修改的目標隨下一次部署移除。[條件過濾][src-helpers]、[部署差集計算][src-deploy]

### Windows 的降級行為

Windows 無符號連結權限時，Dotter 會印出警告，說明缺少連結權限並將改用複製，再改走範本部署路徑，連明確指定的 symbolic target 也會轉換。使用者會收到警告，但沒有在該次降級前選擇是否接受。輸出不再是持續同步的連結；來源改動後，目標仍需重新部署。[部署實作][src-deploy]

降級路徑使用文字讀取與 Handlebars 渲染，不保證能原樣複製任意二進位檔，或保留含字面 `{{` 的內容。實測環境確實觸發無連結權限警告，但案例均明確選用 template，沒有驗證 symbolic 來源降級後的相容性。

## 6. Cache、衝突與失敗後的狀態

### Cache 保存的是內容與對應，不只是 hash

`.dotter/cache.toml` 保存來源與目標路徑，`.dotter/cache/` 則保存渲染後的內容。範本目標與快取內容不同時，代表目標可能在上次部署後被修改。Cache 不只是可有可無的加速資料，也影響更新、刪除與衝突判斷。[Cache 結構][src-config]、[內容比較][src-fs]

由於快取包含輸出內容，若範本輸出機密，快取也可能包含同樣的機密。官方建議將 local 設定與 cache 排除於版本控制之外；忽略 Git 並不等於檔案已加密。[入門文件][getting-started]、[連結與範本][templates]

### 手動修改不一定一律拒絕

| 狀態 | 更新範本時的行為 |
| --- | --- |
| 目標與 cache 相同 | 可依新來源重新渲染 |
| 目標不存在、cache 仍在 | 重新建立目標 |
| 目標與 cache 不同，且與新渲染結果也不同 | 預設略過；`--force` 可覆寫 |
| 目標與 cache 不同，但已等於新渲染結果 | 可接受現況並更新 cache |

最後一種情況可避免把已經手動調整到正確內容的目標一律當成衝突。這個判斷不等於三方合併；來源與手動修改各自有不同變更時，Dotter 不會因此自動合併兩者。[`update_template`][src-actions]

Package 停用、檔案對應移除、來源與目標組合改變時，Dotter 會依 cache 與新設定的差集處理舊輸出。刪除範本目標仍會檢查目前內容；被手動修改的目標可能因保護機制而留下，不能保證每次都會無條件清除。[部署與撤除][src-deploy]、[刪除動作][src-actions]

### 一次部署不是整批交易

部署逐項執行，個別動作失敗後可記錄錯誤並繼續其他動作；已成功寫入的輸出不會因後面的範本失敗而自動回復。實測 `a` 範本有效、`z` 範本缺少變數時，程序回傳 1，但 `a` 的輸出已存在。[動作分派][src-deploy]

兩個不同來源指向同一目標時，也不一定在寫入前就被攔下。實測 `a` 與 `z` 指向相同目標，首次部署先寫入 `a`，再拒絕 `z`，最後回傳 1。這是發生部分寫入後的衝突，不是完整的設定預檢。

單檔寫入也沒有使用「暫存檔寫完後再 rename」作為統一保證。Windows 範本由快取複製到目標；檔案寫入與複製直接使用檔案系統 API。原始碼不足以支持斷電或強制中止時仍具 atomic replacement 的承諾。[檔案系統實作][src-fs]、[範本部署][src-actions]

部署與撤除的一般成功回傳 0，錯誤或因衝突略過的情況回傳 1；部分成功沒有獨立的 exit code。自動化程式只看到 1 時，不能據此判定「完全沒有寫入」。[命令入口][src-main]

## 7. 預覽、命令執行與 hooks

### Dry-run 的保護範圍

| 項目 | v0.13.5 行為 |
| --- | --- |
| 受管理的檔案寫入、刪除與連結操作 | 使用模擬檔案系統，不直接套用到磁碟 |
| Cache 狀態檔儲存 | 略過 |
| 部署與撤除 hooks | 略過 |
| 範本渲染與 `if` 條件求值 | 仍會執行 |
| 渲染中的命令 helpers | 仍可能啟動外部命令並產生副作用 |
| Windows 連結權限探測 | 仍走獨立的實際探測程式，不受模擬檔案系統統一攔截 |

README 與 CLI help 把 `--force` 描述為優先於 `--dry-run`，但 v0.13.5 的部署程式仍以 `dry_run` 決定是否使用模擬檔案系統。實測兩者併用時，受管理檔案與 cache 的內容 hash 均未改變。`force` 讓模擬動作採用強制處理分支，並未在這個案例中切換成實際寫入。[參數說明][src-args]、[部署實作][src-deploy]

Dry-run 的模擬只涵蓋受管理的檔案操作。`command_success`、`command_output` 會直接呼叫系統 shell，沒有接收 dry-run 的隔離控制；實測 dry-run 中的 helper 仍建立了標記檔，而受管理的目標與 pre-deploy hook 的標記檔都未建立。[命令 helpers][src-helpers]

Windows 的權限探測使用固定名稱 `DOTTER_SYMLINK_TEST`，探測前會嘗試移除該路徑，再建立與移除測試連結。即使不使用命令 helper，仍不宜把 dry-run 定義成整個程序絕對不碰磁碟。[`symlinks_enabled`][src-fs]

### Helpers 與 hooks 的能力不同

| 機制 | 用途 | 注意事項 |
| --- | --- | --- |
| Handlebars 條件與迴圈 | 依資料選擇內容 | 缺少變數與錯誤範本會影響產生結果 |
| `include_template` | 讀取並渲染其他範本片段 | 路徑與被引用檔案也是輸入的一部分 |
| `math` | 計算表達式 | v0.13.5 使用 `evalexpr`；Wiki 仍提到 `meval-rs` |
| `command_success` | 依命令成功與否回傳判斷結果 | 命令可能改變外部狀態 |
| `command_output` | 將命令標準輸出插入範本 | 實作不以非零 exit status 自動判定渲染失敗 |
| Rhai 自訂 helper | 擴充轉換邏輯 | 受編譯時 `scripting` feature 控制 |
| Hooks | 在部署、撤除前後執行腳本 | 先渲染再執行；不能等同交易回復機制 |

Helpers 的輸出可能受外部程式版本、環境變數與命令結果影響，相同 TOML 不一定足以重現相同輸出。`watch` 會在檔案變更後重新部署，但不會把產生後的檔案自動反向轉回範本。[範本 helpers][src-helpers]、[監看實作][src-watch]

Hooks 包含 pre-deploy、post-deploy、pre-undeploy、post-undeploy。Windows 會優先尋找對應 `.bat`；Unix 可直接執行具有執行權限的腳本，否則交給 `sh`。Post-deploy hook 在部署動作與 cache 儲存之後執行，因此它失敗時，先前的部署仍可能已經完成。[Hook 實作][src-hooks]、[部署實作][src-deploy]

## 8. rosamta 可以沿用什麼，還需要補什麼

rosamta 的 [README](../../README.md) 將 Module、Template、Profile 列為核心概念，目前仍是設計方向。

| rosamta 概念 | Dotter 已提供的能力 | rosamta 需要具體定義的部分 |
| --- | --- | --- |
| Module | Package 共用變數、表格資料與遞迴相依 | 配色、字型等欄位的型別、範圍、必填條件及跨欄位限制 |
| Template | Handlebars、共用片段、自訂 helper、明確輸出型態 | 內建範本的涵蓋範圍、個別覆寫、資料格式版本與輸出語法驗證 |
| Profile | Local 選取、includes、hostname fallback、平台變數 | OS、hostname、標籤多條件符合時的優先權，以及選取結果的說明 |

### 共用資料需要資料契約

Dotter 能把 `palette.foreground` 提供給多個範本，卻不會因變數名稱是 foreground 就知道它應該是色碼。rosamta 若提供 `theme.yaml`，應明訂色彩格式、透明度範圍、缺值處理與錯誤位置。這樣才能在寫入前指出設定問題，而不是等應用程式啟動失敗。

### 範本維護需要穩定邊界

Package 範例能展示共用資料，但每個程式的格式轉換仍由範本負責。rosamta 規劃附帶常見程式的範本，可降低初次設定成本；相對地，也必須處理應用程式版本變更、模組欄位變更，以及使用者只覆寫單一範本時的相容性。只提供更多範本，不能取代版本與覆寫規則。

### 衝突應在寫入前說清楚

Dotter 的設定層衝突與磁碟層衝突是兩件事：前者包括同名變數、重複來源，後者包括同一目標被多個來源使用、目標被手動修改。rosamta 若要降低部分寫入的情況，需要在產生計畫時就檢查目標唯一性、完成純資料範本渲染，再決定是否寫入。含外部命令的步驟則需要另外標示，避免把預覽當成完全隔離的執行環境。

### 預覽與失敗回報應有明確承諾

「顯示差異」不等於「沒有副作用」，「回傳失敗」也不等於「沒有變更」。rosamta 應分別定義預覽是否允許外部命令、單檔寫入是否 atomic、整批失敗是否回復，以及部分成功如何回報。不能只以一個「安全部署」名稱概括這些不同保證。

## 9. 候選需求與驗收條件

| 編號 | 候選需求 | 驗收條件 | 依據 |
| --- | --- | --- | --- |
| DOT-01 | 一份模組資料可供多個範本引用 | 修改 foreground 後，所有引用輸出使用新值；未修改的 background 保留 | 共用配色與 local 覆寫 |
| DOT-02 | 設定優先權明確，衝突可追溯 | 覆寫結果列出值的來源；package 或模組改名不應暗中改變優先權 | 表格合併與 package 排序 |
| DOT-03 | 模組資料在寫入前完成驗證 | 不合法型別、色碼或缺少必填欄位時，指出欄位與來源，不寫入輸出 | 變數儲存與領域驗證的差異 |
| DOT-04 | 預先檢查目標碰撞 | 兩個輸出解析為相同目標時，在任何目標寫入前回報兩個來源；路徑比較符合平台規則 | 不同來源指向同一目標 |
| DOT-05 | 偵測目標的外部修改 | 手動修改與新產生結果不同時，預設保留目標並列為未完成；兩者已相同時可更新狀態，不列為衝突；只有明確選擇覆寫才更動衝突目標 | Cache 與 drift 判斷 |
| DOT-06 | 尊重明確指定的輸出型態 | 指定 symbolic 卻無連結權限時，預檢失敗且不寫入任何輸出；只有使用者事先允許降級才改用一般檔案，並列出型態變更。含字面 `{{` 的原樣輸出不得經範本渲染 | 自動判定與 Windows 降級 |
| DOT-07 | 預覽的副作用範圍可控制 | 預覽不寫入受管理目標與狀態；預設不執行任意外部命令，需執行的步驟明確列出 | Dry-run、helpers 與 hooks |
| DOT-08 | 清楚回報失敗後的狀態 | 逐項列出完成、保留、未執行與失敗的輸出；全部完成回傳 0，有完成項目也有未完成項目回傳 2，預檢失敗或沒有任何項目完成而仍有未完成項目時回傳 1 | 非交易式部署 |
| DOT-09 | 內建範本可個別覆寫並管理版本 | 覆寫一個範本不需複製整個模組；模組格式與範本不相容時在寫入前報錯 | rosamta 內建與自訂範本方向 |
| DOT-10 | 移除或停用輸出時保護手動修改 | 舊輸出與上次產生內容相同時可清除；內容不同時預設保留，列為未完成並保留追蹤資料；只有明確同意刪除才移除檔案與追蹤資料 | Cache 差集與條件檔案 |

這些項目是 rosamta 的候選規格，不代表正式 SRS 已採用。Atomic replacement、整批回復與狀態檔機密保護也需要各自的驗收條件，不能由上述功能推定已成立。

### 衝突處理與回傳碼

DOT-05 與 DOT-10 的預設策略在互動與非互動執行中均為保留，不自動等待輸入。只有使用者明確啟用互動，且標準輸入連接終端機時，才提供逐檔選擇：更新衝突可選「保留」「覆寫」，刪除衝突可選「保留」「刪除」。未選擇、輸入結束或沒有終端機時，一律保留並繼續回報，不反覆提示。

DOT-08 的「完成」包含成功寫入、成功刪除及原本已符合期望的項目；「未完成」包含因衝突保留、執行失敗及因錯誤而未執行的項目。空的部署計畫回傳 0。預檢失敗時不執行部署並回傳 1；通過預檢後，完成與未完成項目並存才回傳 2。例如 2 個項目中 1 個完成、1 個因衝突保留，回傳 2；2 個均因衝突保留，回傳 1。

DOT-06 與 Dotter 的差異是降級需要事先允許，而非增加一條警告。明確指定的輸出型態應納入預檢，不能在寫入途中因權限不足才臨時改變。

## 10. v0.13.5 的實測結果

環境為 Windows，使用官方 release 執行檔。範本目標使用臨時目錄，未套用到實際 Alacritty、tmux 或個人 home 設定；輸出內容驗證不等於應用程式整合驗證。

| 案例 | 條件 | 觀察結果 |
| --- | --- | --- |
| T01 | Terminal 與 tmux 相依同一個 theme | 兩份輸出使用相同前景與背景，exit code 0 |
| T02 | Local 只覆寫 `palette.foreground` | 兩份輸出的前景更新，背景保留，exit code 0 |
| T03 | 手動修改範本目標，再部署 | 修改保留，回傳 1 |
| T04 | 對衝突目標使用 `--dry-run --force` | 回傳 0，案例目錄內所有檔案的內容 hash 不變 |
| T05 | 對同一衝突使用 `--force` | 目標回復為渲染內容，回傳 0 |
| T06 | 將啟用 package 清單改為空 | 先前未被修改的兩個輸出移除，回傳 0 |
| T07 | 兩個 package 宣告同名頂層純量 | 報 `variable "x" already encountered`，回傳 1 |
| T08 | 兩個 package 宣告同一表格的同名欄位，選取順序為 b、a | b 的值勝出，a 的其他欄位保留，回傳 0 |
| T09 | Includes、local、manual patch 覆寫同一欄位 | Manual patch 勝出，未覆寫欄位保留，回傳 0 |
| T10 | 兩個 includes 覆寫同一欄位，無 local 覆寫 | 後列出的 include 勝出，回傳 0 |
| T11 | a 範本有效、z 範本缺少變數 | a 已寫入，z 未產生，回傳 1；錯誤含變數名稱與範本行列 |
| T12 | Dry-run 範本呼叫會建立檔案的命令 helper，同時存在 pre-deploy hook | Helper 標記檔建立，hook 標記檔與受管理目標均未建立，回傳 0 |
| T13 | 檔案 `if` 由 true 改成 false | 目標先建立，再於下一次部署移除，兩次均回傳 0 |
| T14 | a、z 兩個來源指向同一目標 | a 先寫入，z 因目標已存在而略過，回傳 1 |
| T15 | Include 為既有 app 增加 theme 相依，範本引用 theme 變數 | Theme 未因該相依而啟用，缺少變數，回傳 1 |
| T16 | Prepend 與 append 都含範本變數 | 前綴、來源、後綴一起渲染為 `value/middle/value`，回傳 0 |

Linux、macOS、符號連結建立、symbolic 來源降級後的相容性、強制中止、磁碟故障、watch 的長時間運作，以及應用程式版本相容性均無實測結果。這些情境的可靠性不能由上述案例推定。

### 關鍵案例的最小重現

適用環境：Windows、Python 3、官方 Dotter v0.13.5。程式只在新建的臨時目錄寫入檔案，結束後自動清除；不讀取個人 dotfile。T12 的命令語法使用 Windows `cmd`。

將程式存為 `reproduce.py`，以 `python reproduce.py C:\tools\dotter-windows-x64-msvc.exe` 執行；執行檔路徑依存放位置調整。

```python
import subprocess
import sys
import tempfile
from pathlib import Path

binary = Path(sys.argv[1]).resolve()
version = subprocess.check_output([str(binary), "--version"], text=True).strip()
assert version == "dotter 0.13.5", version

cases = {
    "T11": {
        "global": '''[app.variables]
x = "ok"
[app.files]
"a" = { target = "out/a", type = "template" }
"z" = { target = "out/z", type = "template" }
''',
        "files": {"a": "{{x}}", "z": "{{missing_variable}}"},
        "args": [],
        "code": 1,
        "expected": {"out/a": "ok", "out/z": None},
    },
    "T12": {
        "global": '''[app.files]
"source" = { target = "out/result", type = "template" }
''',
        "files": {
            "source": '{{command_output "echo helper-ran>helper-marker.txt"}}',
            ".dotter/pre_deploy.bat": "@echo hook-ran>hook-marker.txt\n",
        },
        "args": ["--dry-run"],
        "code": 0,
        "expected": {
            "helper-marker.txt": "helper-ran\n",
            "hook-marker.txt": None,
            "out/result": None,
        },
    },
    "T14": {
        "global": '''[app.files]
"a" = { target = "out/result", type = "template" }
"z" = { target = "out/result", type = "template" }
''',
        "files": {"a": "from-a", "z": "from-z"},
        "args": [],
        "code": 1,
        "expected": {"out/result": "from-a"},
    },
    "T15": {
        "global": '''[app.files]
"source" = { target = "out/result", type = "template" }
[theme.variables]
color = "red"
''',
        "local": 'includes = [".dotter/extra.toml"]\npackages = ["app"]\n',
        "files": {
            "source": "{{color}}",
            ".dotter/extra.toml": '[app]\ndepends = ["theme"]\n',
        },
        "args": [],
        "code": 1,
        "expected": {"out/result": None},
    },
}

with tempfile.TemporaryDirectory(prefix="dotter-reproduce-") as temporary:
    for name, case in cases.items():
        root = Path(temporary) / name
        inputs = {
            ".dotter/global.toml": case["global"],
            ".dotter/local.toml": case.get("local", 'packages = ["app"]\n'),
            **case["files"],
        }
        for relative, content in inputs.items():
            path = root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
        command = [str(binary), "--noconfirm", *case["args"]]
        result = subprocess.run(command, cwd=root, stdin=subprocess.DEVNULL,
                                capture_output=True, text=True, encoding="utf-8",
                                errors="replace", timeout=30)
        observed = {
            relative: ((root / relative).read_text(encoding="utf-8")
                       if (root / relative).exists() else None)
            for relative in case["expected"]
        }
        print(name, "exit:", result.returncode, "outputs:", observed)
        print(result.stdout, result.stderr)
        assert result.returncode == case["code"], name
        assert observed == case["expected"], name
```

| 案例 | 可直接核對的結果 |
| --- | --- |
| T11 | 回傳 1，`out/a` 為 `ok`、`out/z` 不存在；已成功寫入的檔案沒有回復 |
| T12 | 回傳 0，只有 helper 建立的標記檔存在；dry-run 略過 hook 與受管理輸出，但不隔離 helper 命令 |
| T14 | 回傳 1，`out/result` 為 `from-a`，錯誤顯示 z 因目標已存在而略過；目標碰撞未在首次寫入前攔截 |
| T15 | 回傳 1，錯誤指出缺少 `color`；同一設定的 theme 具有該變數，但既有 app 的 include 相依未使其啟用 |

## 11. 參考資料

### 官方文件

- [Dotter v0.13.5 README][repo]
- [Getting Started][getting-started]
- [Symbolic links and Templates][templates]
- [Packages and local.toml][packages]
- [Complex Target][targets]
- [Built-ins, Helpers, and Settings][builtins]
- [Miscellaneous features][misc]
- [v0.13.5 release][release]

### 固定版本原始碼

- [`Cargo.toml`：版本、授權、依賴與 features][src-cargo]
- [`config.rs`：設定結構、合併、相依與目錄展開][src-config]
- [`deploy.rs`：部署順序、cache 更新、dry-run 與錯誤收集][src-deploy]
- [`actions.rs`：建立、更新、刪除與範本寫入][src-actions]
- [`filesystem.rs`：檔案比較、實際與模擬操作、符號連結探測][src-fs]
- [`handlebars_helpers.rs`：strict mode、escaping、條件與命令 helpers][src-helpers]
- [`hooks.rs`：腳本渲染與跨平台執行][src-hooks]
- [`args.rs`：命令列參數][src-args]
- [`main.rs`：exit code][src-main]
- [`watch.rs`：變更監看][src-watch]

[repo]: https://github.com/SuperCuber/dotter/blob/7dec24fea4f6516a014f1a3ea51491e23d9005d4/README.md
[getting-started]: https://github.com/SuperCuber/dotter/wiki/1.-Getting-Started/7f111da392bd60385be9e79d08c0178d5693afc4
[templates]: https://github.com/SuperCuber/dotter/wiki/2.-Symbolic-links-and-Templates/7f111da392bd60385be9e79d08c0178d5693afc4
[packages]: https://github.com/SuperCuber/dotter/wiki/3.-Packages-and-%60local.toml%60/7f111da392bd60385be9e79d08c0178d5693afc4
[targets]: https://github.com/SuperCuber/dotter/wiki/4.-Complex-Target/7f111da392bd60385be9e79d08c0178d5693afc4
[builtins]: https://github.com/SuperCuber/dotter/wiki/5.-Built%E2%80%90ins,-Helpers,-and-Settings/7f111da392bd60385be9e79d08c0178d5693afc4
[misc]: https://github.com/SuperCuber/dotter/wiki/6.-Miscellaneous-features/7f111da392bd60385be9e79d08c0178d5693afc4
[release]: https://github.com/SuperCuber/dotter/releases/tag/v0.13.5
[src-cargo]: https://github.com/SuperCuber/dotter/blob/7dec24fea4f6516a014f1a3ea51491e23d9005d4/Cargo.toml
[src-config]: https://github.com/SuperCuber/dotter/blob/7dec24fea4f6516a014f1a3ea51491e23d9005d4/src/config.rs
[src-deploy]: https://github.com/SuperCuber/dotter/blob/7dec24fea4f6516a014f1a3ea51491e23d9005d4/src/deploy.rs
[src-actions]: https://github.com/SuperCuber/dotter/blob/7dec24fea4f6516a014f1a3ea51491e23d9005d4/src/actions.rs
[src-fs]: https://github.com/SuperCuber/dotter/blob/7dec24fea4f6516a014f1a3ea51491e23d9005d4/src/filesystem.rs
[src-helpers]: https://github.com/SuperCuber/dotter/blob/7dec24fea4f6516a014f1a3ea51491e23d9005d4/src/handlebars_helpers.rs
[src-hooks]: https://github.com/SuperCuber/dotter/blob/7dec24fea4f6516a014f1a3ea51491e23d9005d4/src/hooks.rs
[src-args]: https://github.com/SuperCuber/dotter/blob/7dec24fea4f6516a014f1a3ea51491e23d9005d4/src/args.rs
[src-main]: https://github.com/SuperCuber/dotter/blob/7dec24fea4f6516a014f1a3ea51491e23d9005d4/src/main.rs
[src-watch]: https://github.com/SuperCuber/dotter/blob/7dec24fea4f6516a014f1a3ea51491e23d9005d4/src/watch.rs
