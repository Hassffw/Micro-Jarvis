#[macro_use]
extern crate diesel;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::database::establish_connection;
use crate::models::User;
use std::env;
use dotenv::dotenv;

pub mod database;
pub mod models;
pub mod schema;
pub mod ml;

#[derive(Serialize, Deserialize, Clone, Default)]
struct Assistant {
    user: User,
    conversation_history: Vec<String>,
}

impl Assistant {
    fn new(user: User) -> Self {
        Assistant {
            user,
            conversation_history: Vec::new(),
        }
    }

    async fn process_message(&mut self, message: &str) -> String {
        self.conversation_history.push(message.to_string());
        
        let context = self.build_context();
        let response = self.query_perplexity(&context, message).await;
        
        self.conversation_history.push(response.clone());
        self.save_user();
        
        response
    }

    fn build_context(&self) -> String {
        format!(
            "Du bist ein persönlicher Assistent für {}. Ihre Interessen sind: {}. Ihre Ziele sind: {}. 
             Bisheriger Gesprächsverlauf: {}",
            self.user.name,
            self.user.interests.join(", "),
            self.user.goals.join(", "),
            self.conversation_history.join("\n")
        )
    }

    async fn query_perplexity(&self, context: &str, question: &str) -> String {
        let client = reqwest::Client::new();
        let api_key = env::var("PERPLEXITY_API_KEY").expect("PERPLEXITY_API_KEY muss gesetzt sein");
        
        let response = client.post("https://api.perplexity.ai/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "model": "mistral-7b-instruct",
                "messages": [
                    {"role": "system", "content": context},
                    {"role": "user", "content": question}
                ]
            }))
            .send()
            .await
            .expect("API-Anfrage fehlgeschlagen");

        let result: serde_json::Value = response.json().await.expect("Ungültige API-Antwort");
        result["choices"][0]["message"]["content"].as_str().unwrap_or("Entschuldigung, ich konnte keine Antwort generieren.").to_string()
    }

    fn save_user(&self) {
        let conn = establish_connection().get().unwrap();
        User::update(&conn, self.user.id, self.user.clone());
    }
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Verfügbare Befehle:")]
enum Command {
    #[command(description = "Zeigt diese Nachricht an.")]
    Help,
    #[command(description = "Startet ein neues Gespräch.")]
    Start,
    #[command(description = "Fügt ein neues Interesse hinzu.")]
    AddInterest(String),
    #[command(description = "Fügt ein neues Ziel hinzu.")]
    AddGoal(String),
}

type AssistantContainer = Arc<Mutex<Assistant>>;

async fn handle_message(
    bot: Bot,
    message: Message,
    assistant: AssistantContainer,
) -> ResponseResult<()> {
    if let Some(text) = message.text() {
        if let Some(user) = message.from() {
            let conn = establish_connection().get().unwrap();
            let db_user = User::find_or_create_by_telegram_id(&conn, user.id.0 as i64, &user.first_name)
                .expect("Failed to find or create user");

            let mut assistant = assistant.lock().await;
            assistant.user = db_user;
            let response = assistant.process_message(text).await;

            // Update the user in the database
            User::update(&conn, assistant.user.id, assistant.user.clone())
                .expect("Failed to update user");

            bot.send_message(message.chat.id, response).await?;
        }
    }
    Ok(())
}
async fn handle_command(
    bot: Bot,
    message: Message,
    command: Command,
    assistant: AssistantContainer,
) -> ResponseResult<()> {
    dotenv().ok();
    let allowed_user_id: i64 = env::var("ALLOWED_USER_ID")
        .expect("ALLOWED_USER_ID muss gesetzt sein")
        .parse()
        .expect("ALLOWED_USER_ID muss eine gültige i64 sein");

    if message.from().map(|user| user.id.0) == Some(allowed_user_id.try_into().unwrap()) {
        let mut assistant = assistant.lock().await;
        match command {
            Command::Help => {
                bot.send_message(message.chat.id, Command::descriptions().to_string()).await?;
            }
            Command::Start => {
                assistant.conversation_history.clear();
                bot.send_message(message.chat.id, "Neues Gespräch gestartet. Wie kann ich Ihnen helfen?").await?;
            }
            Command::AddInterest(interest) => {
                assistant.user.interests.push(interest);
                assistant.save_user();
                bot.send_message(message.chat.id, "Interesse hinzugefügt.").await?;
            }
            Command::AddGoal(goal) => {
                assistant.user.goals.push(goal);
                assistant.save_user();
                bot.send_message(message.chat.id, "Ziel hinzugefügt.").await?;
            }
        };
    } else {
        bot.send_message(message.chat.id, "Sie sind nicht berechtigt, diesen Bot zu verwenden.").await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(Update::filter_message().filter_command::<Command>().endpoint(handle_command))
        .branch(Update::filter_message().endpoint(handle_message));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![Arc::new(Mutex::new(Assistant::new(User::default())))])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
