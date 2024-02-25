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

// Инициализация глобального хранилища для префиксов с использованием Mutex для потокобезопасного доступа
lazy_static! {
    // Используем Mutex для безопасного доступа в многопоточной среде
    static ref PREFIXES: Mutex<HashMap<String, String>> = Mutex::new({
        let mut m = HashMap::new();
        m.insert("585734874699399188".to_string(), "Хозяин".to_string());
        m.insert("524913624117149717".to_string(), "Кисик".to_string());
        m
    });
}

mod commands;

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

    // Значения по умолчанию для текста-заполнителя и системного приглашения
    let placeholder_text = std::env::var("placeholder").unwrap_or("*Генерирую ответ...*".to_string());
    let system_prompt = std::env::var("system_prompt").unwrap_or("Вы — полезный ассистент, отвечающий на вопросы в Discord.".to_string());

    let token = std::env::var("discord_token").unwrap();
    let bot = ProvidedBot::new(token);
    let discord = bot.get_client();

    // Игнорируем сообщения от ботов
    if msg.author.bot {
        log::info!("ignored bot message");
        return;
    }

    // Обработка команд с использованием функции handle_command из модуля commands
    if let Err(e) = commands::handle_command(&msg, &discord, &placeholder_text, &system_prompt).await {
        log::error!("Ошибка при обработке команды: {}", e);
    }
}
    
    // Обработка команд
    let user_id = msg.author.id; // Получаем ID пользователя
    let channel_id = msg.channel_id; // Получаем ID канала
    let content = msg.content; // Содержимое сообщения

     // Проверяем, начинается ли сообщение с "!"
    if !content.starts_with("!") {
        return; // Если нет, прекращаем обработку
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
    let placeholder  = discord.send_message(
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
        _ = discord.edit_message(
            channel_id.into(), placeholder.id.into(), &embed_message
        ).await;
    }
    Err(e) => {
        let error_message = create_embed("Извините, произошла ошибка. Пожалуйста, попробуйте позже.", None, None);
        _ = discord.edit_message(
            channel_id.into(), placeholder.id.into(), &error_message
        ).await;
        log::error!("OpenAI returns error: {}", e);
    }
}

}
