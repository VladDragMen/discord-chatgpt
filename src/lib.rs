use std::env;
use rand::Rng;
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
use std::time::{Instant};

// Инициализация глобального хранилища для префиксов с использованием Mutex для потокобезопасного доступа
lazy_static! {
    // Используем Mutex для безопасного доступа в многопоточной среде
    static ref PREFIXES: Mutex<HashMap<String, String>> = Mutex::new({
        let mut m = HashMap::new();
        m.insert("585734874699399188".to_string(), "Хозяин".to_string());
        m.insert("524913624117149717".to_string(), "Кисик".to_string());
        m.insert("457149063884963862".to_string(), "Титан".to_string());
        m
    });
    static ref LAST_USED_FURRY: Mutex<HashMap<String, Instant>> = Mutex::new(HashMap::new());
}


fn create_embed(description: &str, title: Option<&str>, fields: Option<Vec<serde_json::Value>>) -> serde_json::Value {
    serde_json::json!({
        "embeds": [{
            "author": {
                "name": "Ответ от Умного Лисёнка 🦊",
                "icon_url": "https://i.imgur.com/emgIscZ.png"
            },
            "title": title.unwrap_or(""),
            "description": description,
            "color": 3447003,
            "fields": fields.unwrap_or_else(Vec::new),
            "footer": {
                "text": "Присоединяйтесь к нам! 🌟 https://discord.gg/vladvd91"
            }
        }]
    })
}

// Основная точка входа в асинхронную задачу
#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    let token = std::env::var("discord_token").unwrap();
    let bot = ProvidedBot::new(token);
    bot.listen_to_messages().await;
}

// Обработчик входящих сообщений
#[message_handler]
async fn handler(msg: Message) {
    logger::init(); // Инициализация логгера
    let token = env::var("discord_token").unwrap();

    // Значения по умолчанию для текста-заполнителя и системного приглашения
    let placeholder_text = env::var("placeholder").unwrap_or("*Генерирую ответ...*".to_string());
    let system_prompt = env::var("system_prompt").unwrap_or("Вы — полезный ассистент, отвечающий на вопросы в Discord.".to_string());

    let bot = ProvidedBot::new(token);
    let discord = bot.get_client();

    // Игнорируем сообщения от ботов
    if msg.author.bot {
        log::info!("ignored bot message");
        return;
    }

    // Обработка команд
    let user_id = msg.author.id; // Получаем ID пользователя
    let channel_id = msg.channel_id; // Получаем ID канала
    let content = msg.content; // Содержимое сообщения

    // Проверяем, начинается ли сообщение с "!"
    if !content.starts_with("!") {
        return; // Если нет, прекращаем обработку
    }

    // Обработка команды перезапуска
    if content.eq_ignore_ascii_case("!рестарт") {
        let embed_message = create_embed("Хорошо, я начинаю новый разговор.", None, None);
        _ = discord.send_message(channel_id.into(), &embed_message).await;
        store::set(&channel_id.to_string(), json!(true), None);
        log::info!("Restarted conversation for {}", channel_id);
        return;
    }

    // Показать список префиксов
    if content.eq_ignore_ascii_case("!префиксы") {
        let prefixes = PREFIXES.lock().unwrap();
        let mut response = String::new();

        for (id, prefix) in prefixes.iter() {
            let user_name = match id.as_str() {
                "585734874699399188" => "@vladvd91",
                "524913624117149717" => "@boykising",
                "457149063884963862" => "@kesama.",
                _ => "Неизвестный",
            };
            response.push_str(&format!("{}: {}\n", prefix, user_name));
        }

        let embed_message = create_embed(&response, Some("Список префиксов"), None);
        _ = discord.send_message(channel_id.into(), &embed_message).await;
        return;
    }

    if content.eq_ignore_ascii_case("!фурри") {
        let mut rng = rand::thread_rng();
        let furry_percentage: i32 = rng.gen_range(50..=10000); // Генерируем случайное число от 50 до 10000
        let response = format!("Ты фурри на {}%", furry_percentage);
        let embed_message = create_embed(&response, None, None);
        _ = discord.send_message(channel_id.into(), &embed_message).await;
        return;
    }

    
    if content.starts_with("!обнять") {
        let content_trimmed = content.trim_start_matches("!обнять").trim(); // Удаляем команду и лишние пробелы
        if !content_trimmed.is_empty() {
            // Проверяем, что после команды что-то есть
            let response = format!("{} обнял милашку {}.", msg.author.name, content_trimmed);
            let embed_message = create_embed(&response, None, None);
            _ = discord.send_message(channel_id.into(), &embed_message).await;
        } else {
            // Если пользователь не указан, отправляем сообщение об ошибке
            let response = "Пожалуйста, укажите пользователя для обнимашек! Пример: !обнять @username";
            let embed_message = create_embed(response, None, None);
            _ = discord.send_message(channel_id.into(), &embed_message).await;
        }
        return;
    }

    // Показать список доступных команд
    if content.eq_ignore_ascii_case("!команды") {
    let commands_description = create_embed(
        "Вот список команд, которые вы можете использовать:",
        Some("Список доступных команд"),
        Some(vec![
            serde_json::json!({
                "name": "!префиксы",
                "value": "Показывает список всех установленных префиксов и их владельцев.",
                "inline": false
            }),
            serde_json::json!({
                "name": "!рестарт",
                "value": "Перезапускает текущий разговор, начиная общение заново.",
                "inline": false
            }),
            // Добавленный пункт
            serde_json::json!({
                "name": "Общение с ботом",
                "value": "Чтобы общаться с ботом, необходимо начнать сообщение с \"!\". Пример: !как создать воду.",
                "inline": false
            }),
        ]),
    );

    _ = discord.send_message(channel_id.into(), &commands_description).await;
    return;
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
        "457149063884963862" => "Титан, ",
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
