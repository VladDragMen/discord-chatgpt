use discord_flows::{model::Message, Bot, ProvidedBot, message_handler};
use flowsnet_platform_sdk::logger;
use openai_flows::{chat::{ChatModel, ChatOptions}, OpenAIFlows};
use serde_json::{json, Value};
use std::{collections::HashMap, env, sync::Mutex};
use lazy_static::lazy_static;

lazy_static! {
    static ref PREFIXES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::from([
        ("585734874699399188".to_string(), "Хозяин".to_string()),
        ("524913624117149717".to_string(), "Кисик".to_string()),
    ]));
}

const EMBED_COLOR: i32 = 3447003;
const DEFAULT_DESCRIPTION: &str = "*Генерирую ответ...*";
const SYSTEM_PROMPT: &str = "Вы — полезный ассистент, отвечающий на вопросы в Discord.";
const DISCORD_INVITE_LINK: &str = "https://discord.gg/vladvd91";

fn create_embed(description: &str, title: Option<&str>, fields: Option<Vec<Value>>) -> Value {
    json!({
        "embeds": [{
            "author": {
                "name": "Ответ от Умного Лисёнка 🦊",
                "icon_url": "https://i.imgur.com/emgIscZ.png"
            },
            "title": title.unwrap_or_default(),
            "description": description,
            "color": EMBED_COLOR,
            "fields": fields.unwrap_or_default(),
            "footer": {
                "text": format!("Присоединяйтесь к нам! 🌟 {}", DISCORD_INVITE_LINK)
            }
        }]
    })
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let token = env::var("discord_token").expect("Expected a token in the environment");
    let bot = ProvidedBot::new(token);
    bot.listen_to_messages(handler).await;
}

#[message_handler]
async fn handler(msg: Message) {
    logger::init();
    
    if msg.author.bot {
        log::info!("ignored bot message");
        return;
    }
    
    if !msg.content.starts_with("!") {
        return;
    }
    
    let discord = ProvidedBot::new(env::var("discord_token").unwrap()).get_client();
    let channel_id = msg.channel_id;
    
match msg.content.to_lowercase().as_str() {
        "!рестарт" => handle_restart(&discord, &channel_id).await,
        "!префиксы" => show_prefixes(&discord, &channel_id).await,
        "!команды" => show_commands(&discord, &channel_id).await,
        _ => log::info!("Unknown command: {}", msg.content),
    }
}

async fn handle_restart(discord: &Bot, channel_id: &str) {
    // Example implementation. Adapt based on actual logic.
    let embed_message = create_embed("Перезапуск беседы.", None, None);
    let _ = discord.send_message(channel_id.into(), &embed_message).await;
}

async fn show_prefixes(discord: &Bot, channel_id: &str) {
    let prefixes = PREFIXES.lock().unwrap();
    let response = prefixes.iter().map(|(id, prefix)| format!("{}: {}", id, prefix)).collect::<Vec<_>>().join("\n");
    let embed_message = create_embed(&response, Some("Список префиксов"), None);
    let _ = discord.send_message(channel_id.into(), &embed_message).await;
}

async fn show_commands(discord: &Bot, channel_id: &str) {
    let commands_description = create_embed(
        "Вот список команд, которые вы можете использовать:",
        Some("Список доступных команд"),
        Some(vec![
            json!({"name": "!префиксы", "value": "Показывает список всех установленных префиксов и их владельцев.", "inline": false}),
            json!({"name": "!рестарт", "value": "Перезапускает текущий разговор, начиная общение заново.", "inline": false}),
        ])
    );
    let _ = discord.send_message(channel_id.into(), &commands_description).await;
}

match msg.content.to_lowercase().as_str() {
    "!рестарт" => handle_restart(&discord, &channel_id).await,
    "!префиксы" => show_prefixes(&discord, &channel_id).await,
    "!команды" => show_commands(&discord, &channel_id).await,
    // Добавление обработки команды для взаимодействия с OpenAI
    _ => handle_openai_response(&discord, &msg).await,
}

// Добавим функцию `handle_openai_response`
async fn handle_openai_response(discord: &Bot, msg: &Message) {
    let channel_id = msg.channel_id.to_string();
    let content = &msg.content;
    
    let placeholder = discord.send_message(
        &channel_id,
        &serde_json::json!({"content": DEFAULT_DESCRIPTION}),
    ).await.expect("Failed to send placeholder message");

    let mut openai = OpenAIFlows::new(); // Инициализация клиента OpenAI
    openai.set_retry_times(3);

    let co = ChatOptions {
        model: ChatModel::GPT35Turbo, // Выбор модели чата
        restart: false,
        system_prompt: Some(SYSTEM_PROMPT),
        ..Default::default()
    };

    let response_prefix = PREFIXES.lock().unwrap().get(&msg.author.id.to_string()).unwrap_or(&"".to_string()).to_string();

    match openai.chat_completion(&channel_id, content, &co).await {
        Ok(r) => {
            let response = format!("{}{}", response_prefix, r.choice);
            let embed_message = create_embed(&format!("```{}\n```", response), None, None);
            let _ = discord.edit_message(&channel_id, &placeholder.id.into(), &embed_message).await;
        }
        Err(e) => {
            let error_message = create_embed("Извините, произошла ошибка. Пожалуйста, попробуйте позже.", None, None);
            let _ = discord.edit_message(&channel_id, &placeholder.id.into(), &error_message).await;
            log::error!("OpenAI returns error: {}", e);
        }
    }
}
