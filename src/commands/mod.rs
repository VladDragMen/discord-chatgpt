use discord_flows::{model::Message, ProvidedBot};
use serde_json::Value;
use super::{create_embed, PREFIXES};

// Функция для отображения списка префиксов
pub async fn show_prefixes(msg: &Message, bot: &ProvidedBot) {
    let prefixes = PREFIXES.lock().unwrap();
    let mut response = String::from("Список префиксов:\n");

    for (id, prefix) in prefixes.iter() {
        let user_name = match id.as_str() {
            "585734874699399188" => "@vladvd91",
            "524913624117149717" => "@boykising",
            _ => "Неизвестный пользователь",
        };
        response.push_str(&format!("{}: {}\n", prefix, user_name));
    }

    let embed_message = create_embed(&response, Some("Префиксы"), None);
    let _ = bot.send_message(msg.channel_id, &embed_message).await;
}

// Функция для перезапуска беседы
pub async fn restart_conversation(msg: &Message, bot: &ProvidedBot) {
    let embed_message = create_embed("Хорошо, я начинаю новый разговор.", None, None);
    let _ = bot.send_message(msg.channel_id, &embed_message).await;
    // Здесь логика для сброса состояния разговора, если необходимо
}

// Функция обработки команд
pub async fn handle_command(msg: &Message, bot: &ProvidedBot) {
    if msg.content.starts_with("!") {
        match msg.content.as_str() {
            "!префиксы" => show_prefixes(msg, bot).await,
            "!рестарт" => restart_conversation(msg, bot).await,
            _ => {
                let error_message = "Команда не найдена.";
                let embed_message = create_embed(error_message, None, None);
                let _ = bot.send_message(msg.channel_id, &embed_message).await;
            }
        }
    }
}
