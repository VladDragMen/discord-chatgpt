use std::env;
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;
use std::time::{Duration, Instant};
use discord_flows::{model::Message, Bot, ProvidedBot, message_handler};
use flowsnet_platform_sdk::logger;
use openai_flows::{chat::{ChatModel, ChatOptions}, OpenAIFlows};
use store_flows as store;
use serde_json::json;

lazy_static! {
    static ref PREFIXES: Mutex<HashMap<String, String>> = Mutex::new({
        let mut m = HashMap::new();
        m.insert("585734874699399188".to_string(), "Хозяин".to_string());
        m.insert("524913624117149717".to_string(), "Кисик".to_string());
        m
    });
}

fn create_embed(description: &str, title: Option<&str>, fields: Option<Vec<serde_json::Value>>) -> serde_json::Value {
    json!({
        "embeds": [{
            "author": {
                "name": "Ответ от Умного Лисёнка 🦊",
                "icon_url": "https://i.imgur.com/emgIscZ.png"
            },
            "title": title.unwrap_or_default(),
            "description": description,
            "color": 3447003,
            "fields": fields.unwrap_or_else(Vec::new),
            "footer": {
                "text": "Присоединяйтесь к нам! 🌟 https://discord.gg/vladvd91"
            }
        }]
    })
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    logger::init(); // Инициализация логгера один раз
    let token = env::var("discord_token").expect("Expected a token in the environment");
    let bot = ProvidedBot::new(token);
    bot.listen_to_messages().await;
}

#[message_handler]
async fn handler(msg: Message) {
    if msg.author.bot {
        return;
    }

    let discord = ProvidedBot::new(env::var("discord_token").unwrap()).get_client();
    let content = msg.content.trim();

    if !content.starts_with("!") {
        return;
    }

    match content.to_lowercase().as_str() {
        "!рестарт" => handle_restart(&discord, &msg).await,
        "!префиксы" => show_prefixes(&discord, &msg).await,
        "!фурри" => handle_furry(&discord, &msg).await,
        "!обнять" => handle_hug(&discord, &msg, content).await,
        "!команды" => show_commands(&discord, &msg).await,
        _ => return,
    }
}

async fn handle_restart(discord: &ProvidedBotClient, msg: &Message) {
    let embed_message = create_embed("Хорошо, я начинаю новый разговор.", None, None);
    discord.send_message(msg.channel_id.into(), &embed_message).await.unwrap();
    store::set(&msg.channel_id.to_string(), json!(true), None).unwrap();
}

async fn show_prefixes(discord: &ProvidedBotClient, msg: &Message) {
    let prefixes = PREFIXES.lock().unwrap();
    let response = prefixes.iter()
        .map(|(id, prefix)| format!("{}: @{}", prefix, id))
        .collect::<Vec<_>>()
        .join("\n");

    let embed_message = create_embed(&response, Some("Список префиксов"), None);
    discord.send_message(msg.channel_id.into(), &embed_message).await.unwrap();
}

async fn handle_furry(discord: &ProvidedBotClient, msg: &Message) {
    let furry_percentage: i32 = rand::thread_rng().gen_range(50..=100);
    let response = format!("Ты фурри на {}%", furry_percentage);
    let embed_message = create_embed(&response, None, None);
    discord.send_message(msg.channel_id.into(), &embed_message).await.unwrap();
}

async fn handle_hug(discord: &ProvidedBotClient, msg: &Message, content: &str) {
    let target = content.strip_prefix("!обнять").unwrap_or("").trim();
    let response = if !target.is_empty() {
        format!("{} обнял милашку {}.", msg.author.name, target)
    } else {
        "Пожалуйста, укажите пользователя для обнимашек! Пример: !обнять @username".to_string()
    };
    let embed_message = create_embed(&response, None, None);
    discord.send_message(msg.channel_id.into(), &embed_message).await.unwrap();
}

async fn show_commands(discord: &ProvidedBotClient, msg: &Message) {
    let commands_description = create_embed(
        "Вот список команд, которые вы можете использовать:",
        Some("Список доступных команд"),
        Some(vec![
            json!({"name": "!префиксы", "value": "Показывает список всех установленных префиксов и их владельцев.", "inline": false}),
            json!({"name": "!рестарт", "value": "Перезапускает текущий разговор, начиная общение заново.", "inline": false}),
            json!({"name": "Общение с ботом", "value": "Чтобы общаться с ботом, необходимо начнать сообщение с \"!\". Пример: !как создать воду.", "inline": false}),
        ]),
    );
    discord.send_message(msg.channel_id.into(), &commands_description).await.unwrap();
}

    // Проверка и обработка состояния перезапуска разговора
    let restart = store::get(&channel_id.to_string())
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if restart {
        log::info!("Detected restart = true");
        store::set(&channel_id.to_string(), json!(false), None);
    }

    // Отправка сообщения-заполнителя перед получением ответа от OpenAI
    let placeholder = discord.send_message(
        channel_id.into(),
        &serde_json::json!({
            "content": &placeholder_text
        }),
    ).await.unwrap();

    // Инициализация клиента OpenAI и настройка опций для чата
    let mut openai = OpenAIFlows::new();
    openai.set_retry_times(3);
    let co = ChatOptions {
       // model: ChatModel::GPT4,
        model: ChatModel::GPT35Turbo, // Выбор модели чата
        restart: restart,
        system_prompt: Some(&system_prompt),
        ..Default::default()
    };

    // Определение префикса ответа в зависимости от ID пользователя
    let response_prefix = match msg.author.id.to_string().as_str() {
        "585734874699399188" => "Хозяин, ",
        "524913624117149717" => "Кисик, ",
        _ => ""
    };

    // Получение и обработка ответа от OpenAI
            match openai.chat_completion(&channel_id.to_string(), &content, &co).await {
    Ok(r) => {
        let response = format!("{}{}", response_prefix, r.choice);
        let embed_message = create_embed(&format!("```elixir\n{}\n```", response), None, None);

        // Отправляем ответ от OpenAI
        let _ = discord.send_message(channel_id.into(), &embed_message).await;

        // Удаляем сообщение-заполнитель
        if let Err(err) = discord.delete_message(channel_id.into(), placeholder.id.into()).await {
            log::error!("Failed to delete placeholder message: {}", err);
        }
    }
    Err(e) => {
        let error_message = create_embed("Извините, произошла ошибка. Пожалуйста, попробуйте позже.", None, None);

        // Заменяем сообщение-заполнитель сообщением об ошибке
        let _ = discord.send_message(channel_id.into(), &error_message).await;

        // Удаляем сообщение-заполнитель
        if let Err(err) = discord.delete_message(channel_id.into(), placeholder.id.into()).await {
            log::error!("Failed to delete placeholder message: {}", err);
        }

        log::error!("OpenAI returns error: {}", e);
    }
}
}
