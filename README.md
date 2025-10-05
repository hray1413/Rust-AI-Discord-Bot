# Gemini Rust Discord Bot 🤖

一個使用 **Rust** 編寫，並整合 **Google Gemini AI API** 的 Discord 機器人。  
支援頻道冷卻、使用者冷卻、訊息排隊、快取以及退避重試，適合免費 API 使用。

---

## 功能 Features

### 主要功能
- 每個頻道 1 分鐘冷卻一次訊息
- 每個使用者 1 分鐘冷卻一次訊息
- 排隊系統，訊息依序處理，避免同時大量呼叫 API
- 快取相同訊息，節省 API 配額
- 遇到 API 限流或失敗時自動退避重試

### 次要功能
- 支援從 `.env` 讀取 Discord Token 與 Gemini API Key
- 適合免費 API 使用者，降低額度浪費
- 便於擴充，可加入更多自訂功能

---

## 安裝 Installation

### 需求 Requirements
- Rust 1.70 以上
- Discord Bot Token
- Gemini API Key
- `.env` 文件管理敏感資訊

### 步驟 Steps
1. Clone 專案：
```bash
git clone https://github.com/hray1413/Rust-AI-Discord-Bot.git
cd Rust-AI-Discord-Bot
```
2. 安裝依賴：
```bash
cargo build
```
3. 建立 .env 文件：
```env
DISCORD_TOKEN=你的DiscordBotToken
GEMINI_API_KEY=你的GeminiAPIKey
```
4. 運行 Bot：
```bash
cargo run
```
---
## 使用 Usage
1. 將 Bot 加入你的 Discord 伺服器
2. 在頻道中發送訊息，Bot 將自動回覆 AI 生成的回答
3. 每個頻道與使用者有冷卻時間，避免重複呼叫 API
---
## 貢獻 Contributing
歡迎提出 Issue 或 Pull Request。
請遵守專案的 MIT 授權。
---
## 授權 License
此專案採 MIT License 授權。
詳見 [LICENSE](./LICENSE) 文件。
