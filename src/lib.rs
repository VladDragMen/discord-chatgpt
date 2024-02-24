use std::env;
use discord_flows::{model::Message, Bot, ProvidedBot, message_handler};
use flowsnet_platform_sdk::logger;
use openai_flows::{
    chat::{ChatModel, ChatOptions},
    OpenAIFlows,
};
use store_flows as store;
use serde_json::json;

use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    // Используем Mutex для безопасного доступа в многопоточной среде
    static ref PREFIXES: Mutex<HashMap<String, String>> = Mutex::new({
        let mut m = HashMap::new();
        m.insert("585734874699399188".to_string(), "Хозяин".to_string());
        m.insert("524913624117149717".to_string(), "Кисик".to_string());
        m
    });
}

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    let token = std::env::var("discord_token").unwrap();
    let bot = ProvidedBot::new(token);
    bot.listen_to_messages().await;
}

#[message_handler]
async fn handler(msg: Message) {
    logger::init();
    let token = env::var("discord_token").unwrap();
    let placeholder_text = env::var("placeholder").unwrap_or("*Генерирую ответ...*".to_string());
    let system_prompt = env::var("system_prompt").unwrap_or("Вы — полезный ассистент, отвечающий на вопросы в Discord.".to_string());

    let bot = ProvidedBot::new(token);
    let discord = bot.get_client();
    if msg.author.bot {
        log::info!("ignored bot message");
        return;
    }

    let user_id = msg.author.id; // Получаем ID пользователя, отправившего сообщение
    let channel_id = msg.channel_id;
    let content = msg.content;

    // Триггер, чтобы реагировать только на сообщения, начинающиеся с "!"
    if !content.starts_with("!") {
        return; // Если сообщение не начинается с "!", функция завершается здесь
    }

    if content.eq_ignore_ascii_case("!restart") {
        _ = discord.send_message(
            channel_id.into(),
            &serde_json::json!({
                "content": "Хорошо, я начинаю новый разговор."
            }),
        ).await;
        store::set(&channel_id.to_string(), json!(true), None);
        log::info!("Restarted converstion for {}", channel_id);
        return;
    }

    let restart = store::get(&channel_id.to_string())
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if restart {
        log::info!("Detected restart = true");
        store::set(&channel_id.to_string(), json!(false), None);
    }

    let placeholder  = discord.send_message(
        channel_id.into(),
        &serde_json::json!({
            "content": &placeholder_text
        }),
    ).await.unwrap();

    let mut openai = OpenAIFlows::new();
    openai.set_retry_times(3);
    let co = ChatOptions {
        // model: ChatModel::GPT4,
        model: ChatModel::GPT35Turbo,
        restart: restart,
        system_prompt: Some(&system_prompt),
        ..Default::default()
    }

    if content.eq_ignore_ascii_case("!prefixes") {
        let prefixes = PREFIXES.lock().unwrap(); // Безопасно получаем доступ к префиксам
        let mut response = "Существующие префиксы:\n".to_string();
        for (id, prefix) in prefixes.iter() {
            let user_name = match id.as_str() { // Пример условия, здесь вы можете добавить логику для получения имени пользователя по ID
                "585734874699399188" => "Пользователь1",
                "524913624117149717" => "Пользователь2",
                _ => "Неизвестный",
            };
            response.push_str(&format!("{}: {}\n", prefix, user_name));
        }

        _ = discord.send_message(
            channel_id.into(),
            &serde_json::json!({
                "content": response
            }),
        ).await;
        return;
    }

    match openai.chat_completion(&channel_id.to_string(), &content, &co).await {
        Ok(r) => {
            let response = format!("{}{}", response_prefix, r.choice);
            _ = discord.edit_message(
                channel_id.into(), placeholder.id.into(),
                &serde_json::json!({
                    "content": "", // Явно очищаем исходное текстовое содержимое
                    "embeds": [{
                        "author": {
                            "name": "Ответ от Умного Лисёнка 🦊",
                            "icon_url": "https://i.imgur.com/emgIscZ.png"
                        },
                        "description": format!("```elixir\n{}\n```", response),
                        "color": 3447003,
                        "footer": {
                            "text": "Присоединяйтесь к нам! 🌟 https://discord.gg/vladvd91"
                        }
                    }]
                }),
            ).await;
    }
    Err(e) => {
        _ = discord.edit_message(
            channel_id.into(), placeholder.id.into(),
            &serde_json::json!({
                "content": "Sorry, an error has occurred. Please try again later!"
            }),
        ).await;
        log::error!("OpenAI returns error: {}", e);
    }
}

}
