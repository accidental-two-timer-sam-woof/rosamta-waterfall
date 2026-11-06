# 設計分析

## 1. 分析基準

需求範圍與驗收條件以 [SRS 1.1](../02-srs/srs.md) 為準。[初步設計 UML](../03-preliminary-design/uml.md) 的 C1–C4 用於核對責任分工，S1–S6 用於核對執行順序。設計選擇由團隊審查，與驗證情境及待定事項一併交給[詳細設計](../05-detailed-design/detailed-design.md)。

Royce 在圖 5 把初步程式設計放在分析之前，以便及早檢查執行時間、儲存空間、資料流和作業系統介面的限制。本專案先用 UML 分析跨平台寫入、接管與回復，再依圖 7 規劃小型驗證，優先檢驗最可能影響設計的假設。 [Royce，1970，圖 5、7](https://www.praxisframework.org/files/royce1970.pdf)

依 Valente 第 4 章，用類別圖核對資料與責任歸屬，用循序圖核對各情境的訊息與順序。各指令的術語與規則是否一致，依第 5 章的概念完整性判斷。服務分工則以資訊隱藏、內聚與耦合評估。依第 8 章，決策規則可獨立檢查，整合行為須在實際檔案系統驗證。 [第 4 章](https://softengbook.org/chapter4) · [第 5 章](https://softengbook.org/chapter5) · [第 8 章](https://softengbook.org/chapter8)

第一版須涵蓋全部 `essential` 需求。互動處理、一次列出所有預檢錯誤、依賴檢查、機器可讀輸出與效能目標屬於 `conditional` 需求。團隊評估後確認納入或延後，並依 SRS §2.6 與附錄 A.3 記錄結果。
