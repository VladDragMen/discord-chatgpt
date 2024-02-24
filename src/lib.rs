use std::env;
use discord_flows::{model::Message, Bot, ProvidedBot, message_handler};
use flowsnet_platform_sdk::logger;
use openai_flows::{
    chat::{ChatModel, ChatOptions},
    OpenAIFlows,
};
use store_flows as store;
use serde_json::json;

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
    let channel_id = msg.channel_id;
    let content = msg.content;

    // Триггер, чтобы реагировать только на сообщения, начинающиеся с "!"
    if !content.starts_with("!") {
        return; // Если сообщение не начинается с "!", функция завершается здесь
    }

    if content.eq_ignore_ascii_case("restart") {
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
    };

    match openai.chat_completion(&channel_id.to_string(), &content, &co).await {
    Ok(r) => {
        _ = discord.edit_message(
            channel_id.into(), placeholder.id.into(),
            &serde_json::json!({
                "embeds": [{
                    "title": "Ответ от Умного Лисёнка",
                    "description": format!("```elixir\n{}\n```", r.choice),
                    "color": 3447003, // Голубой цвет рамки
                    "footer": {
                        "text": "https://discord.gg/vladvd91"
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
