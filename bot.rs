use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use tokio::time::{sleep, Duration, Instant};
use dashmap::DashMap;
use dotenv::dotenv;
use std::env;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::{HashMap, VecDeque};

// ---- Gemini API ----
#[derive(Serialize)]
struct GeminiRequest {
    prompt: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    output: String,
}

// ---- Bot Data ----
struct BotData;

impl TypeMapKey for BotData {
    type Value = Arc<BotState>;
}

struct BotState {
    channel_cooldowns: DashMap<u64, Instant>,
    user_cooldowns: DashMap<u64, Instant>,
    queue: Mutex<VecDeque<(u64, u64, String)>>, // (channel_id, user_id, message)
    cache: DashMap<String, String>,            // prompt -> response
}

// ---- Handler ----
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let bot_state = {
            let data_read = ctx.data.read().await;
            data_read.get::<BotData>().unwrap().clone()
        };

        let now = Instant::now();

        // ---- 冷卻檢查 ----
        if let Some(last) = bot_state.channel_cooldowns.get(&msg.channel_id.0) {
            if now.duration_since(*last.value()) < Duration::from_secs(60) {
                let _ = msg.channel_id.say(&ctx.http, "頻道冷卻中，請稍候").await;
                return;
            }
        }

        if let Some(last) = bot_state.user_cooldowns.get(&msg.author.id.0) {
            if now.duration_since(*last.value()) < Duration::from_secs(60) {
                let _ = msg.channel_id.say(&ctx.http, "你冷卻中，請稍候").await;
                return;
            }
        }

        // 更新冷卻
        bot_state.channel_cooldowns.insert(msg.channel_id.0, now);
        bot_state.user_cooldowns.insert(msg.author.id.0, now);

        // ---- 快取檢查 ----
        if let Some(cached) = bot_state.cache.get(&msg.content) {
            let _ = msg.channel_id.say(&ctx.http, cached.value().clone()).await;
            return;
        }

        // ---- 排隊 ----
        {
            let mut queue = bot_state.queue.lock().await;
            queue.push_back((msg.channel_id.0, msg.author.id.0, msg.content.clone()));
        }

        // ---- 處理排隊 ----
        process_queue(ctx, bot_state).await;
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

// ---- 排隊處理函數 ----
async fn process_queue(ctx: Context, bot_state: Arc<BotState>) {
    let client = Client::new();
    let gemini_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");

    loop {
        let task_opt = {
            let mut queue = bot_state.queue.lock().await;
            queue.pop_front()
        };

        if let Some((channel_id, _user_id, prompt)) = task_opt {
            // ---- 退避重試 ----
            let mut delay = Duration::from_secs(1);
            let mut success = false;

            for _ in 0..5 {
                let response = client
                    .post("https://api.google.com/gemini/v1/complete") // 假設 endpoint
                    .header("Authorization", format!("Bearer {}", gemini_key))
                    .json(&GeminiRequest { prompt: prompt.clone() })
                    .send()
                    .await;

                match response {
                    Ok(resp) => {
                        if let Ok(json) = resp.json::<GeminiResponse>().await {
                            // ---- 快取 ----
                            bot_state.cache.insert(prompt.clone(), json.output.clone());

                            // 發送到 Discord
                            let _ = serenity::model::id::ChannelId(channel_id)
                                .say(&ctx.http, json.output)
                                .await;
                            success = true;
                            break;
                        }
                    }
                    Err(_) => {
                        sleep(delay).await;
                        delay *= 2;
                        if delay > Duration::from_secs(32) {
                            delay = Duration::from_secs(32);
                        }
                    }
                }
            }

            if !success {
                let _ = serenity::model::id::ChannelId(channel_id)
                    .say(&ctx.http, "AI 回覆失敗，請稍後再試")
                    .await;
            }

            // 等 1 秒再處理下一個，避免免費額度瞬間用完
            sleep(Duration::from_secs(1)).await;
        } else {
            break;
        }
    }
}

// ---- main ----
#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");

    let bot_state = Arc::new(BotState {
        channel_cooldowns: DashMap::new(),
        user_cooldowns: DashMap::new(),
        queue: Mutex::new(VecDeque::new()),
        cache: DashMap::new(),
    });

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<BotData>(bot_state);
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
