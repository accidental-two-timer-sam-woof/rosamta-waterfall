| 專案名稱 | Dotbot |
| --- | --- |
| 連結 | https://github.com/anishathalye/dotbot |
| 作者與授權 | Anish Athalye；MIT License |
| 主要技術 | Python；YAML 或 JSON 部署設定 <br> 命令列介面 |
| 相依環境 | pyproject.toml 宣告 Python >=3.7、PyYAML<br>Windows 依 README 要求 Python 3.8 以上及建立符號連結的權限 |
| 選擇原因 | 核心流程明確，可觀察設定檔解析、任務分派、外掛與檔案操作的分工 |
| 調查日期 | 2026/11/4 |

本次記錄以 master 內容為準



Dotbot 是用來自動部署個人設定檔的工具，主要理念為輕量級和獨立性。

因為它不負責設定檔的版本管理，所以可搭配 Git 使用。



Dotbot 支援許多第三方外掛功能（plugins）：

| 金鑰管理 | dotbot-age, dotbot-gitcrypt, |
| --- | --- |
| 軟體套件管理 | dotbot-brew, dotbot-apt, dotbot-yum, |
| 作業系統和應用程式設定 | crontab-dotbot, dotbot-firefox, |

### 外掛的使用時機與範例

當部署需求除了建立設定檔連結，還包含安裝軟體等工作時，可以使用第三方外掛擴充功能。

例如，在 macOS 或 Linux 上，可透過 `dotbot-brew` 使用 Homebrew 安裝套件。先取得外掛，再於部署清單載入：

```yaml
- plugins:
    - dotbot-brew/

- brew:
    - git
```

其中，`dotbot-brew/` 是已下載的外掛目錄；`brew` 是外掛新增的動作，這裡表示透過 Homebrew 安裝 Git。只有寫入設定並不會立即安裝，還需要執行 Dotbot 部署指令。

參考資料：https://github.com/d12frosted/dotbot-brew

Dotbot 的主要定位是部署工具。使用者先準備設定檔及部署清單，再交由 Dotbot 依序建立連結、建立目錄或執行指令。

註：Dotbot 不會自動將本機修改推送到遠端，也不負責解決 Git 合併衝突。





操作範例：

```yaml
- link:
    ~/.config/terminator:
      create: true
      path: config/terminator
    ~/.vim: vim
    ~/.vimrc:
      relink: true
      path: vimrc
    ~/.zshrc:
      force: true
      path: zshrc
    ~/.hammerspoon:
      if: '[ `uname` = Darwin ]'
      path: hammerspoon
    ~/.config/:
      glob: true
      path: dotconf/config/**
    ~/:
      glob: true
      path: dotconf/*
      prefix: '.'
```

上述程式碼為 Dotbot 官方 README 中 Link 的範例。此範例展示 Dotbot 如何依照設定清單建立符號連結。

左側的路徑表示電腦上要建立連結的位置，path 表示來源檔案或目錄，通常放在 dotfiles repo 中。
例如：`~/.vim: vim` 表示在使用者家目錄建立 `.vim` 連結，指向 repo 內的 `vim`。

| create: true | 允許建立連結所需的父目錄 |
| --- | --- |
| relink: true | 允許替換既有的符號連結 |
| force: true | 允許移除既有檔案或目錄，再建立連結，可能影響原有設定 |

官方範例節錄，尚未實測

值得注意的是，雖然 Dotbot 可用於不同的作業系統，但設定檔位置、符號連結權限與 shell 指令可能因平台而異。因此，同一份部署清單不一定能直接在所有系統使用，仍須依環境調整。



對本組專題的參考價值：

本組可參考 Dotbot 使用設定清單描述來源與目標路徑的方式，減少逐一手動部署的工作。設計時也應考慮平台差異，以及目標位置已有設定檔時，如何提示使用者、保留備份或停止操作。
