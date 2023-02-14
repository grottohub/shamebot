// the bot will not always be listening, but is the only way to
// hit Discord's API
use database::prelude::{
    AccountabilityRequest, Client as DbClient, Guild, List, RequestStatus, Task,
};
use log::{error, info};
use serenity::{
    async_trait,
    model::{
        prelude::{ChannelId, ChannelType, GuildId, Member, PrivateChannel, Ready, UserId},
        user::User,
    },
    prelude::*,
};
use uuid::Uuid;

use crate::environment::Env;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("connected as {}", ready.user.name);
    }
}

pub struct Bot {
    client: Client,
    db_client: DbClient,
    env: Env,
}

impl Bot {
    pub async fn new() -> Self {
        let env = Env::new();
        let intents = GatewayIntents::GUILD_WEBHOOKS
            | GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES;
        let client = Client::builder(&env.discord_token, intents)
            .event_handler(Handler)
            .await
            .map_err(|e| error!("{:?}", e))
            .unwrap();

        let db_client = DbClient::new().await;

        Bot {
            client,
            db_client,
            env,
        }
    }

    pub async fn start(&mut self) {
        if let Err(err) = self.client.start().await {
            error!("{:?}", err);
        }
    }

    pub async fn get_guild_members(&self) -> Vec<Member> {
        let http = self.client.cache_and_http.http.as_ref();
        GuildId(self.env.discord_guild)
            .members(http, None, None)
            .await
            .map_err(|e| error!("{:?}", e))
            .unwrap_or_else(|_| Vec::new())
    }

    pub async fn get_text_channels(&self) -> Vec<ChannelId> {
        let http = self.client.cache_and_http.http.as_ref();
        let all_channels = GuildId(self.env.discord_guild)
            .channels(http)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();

        let mut channels: Vec<ChannelId> = Vec::new();

        if let Some(all_channels) = all_channels {
            for entry in all_channels {
                if entry.1.kind == ChannelType::Text {
                    channels.push(entry.0);
                }
            }
        }

        channels
    }

    pub async fn get_user(&self, user_id: u64) -> Option<User> {
        let http = self.client.cache_and_http.http.as_ref();
        UserId(user_id)
            .to_user(http)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok()
    }

    pub async fn create_dm(&self, user_id: u64) -> Option<PrivateChannel> {
        let http = self.client.cache_and_http.http.as_ref();
        UserId(user_id)
            .create_dm_channel(http)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok()
    }

    pub async fn send_dm(&self, user_id: u64, message: String) {
        let http = self.client.cache_and_http.http.as_ref();
        let channel = self.create_dm(user_id).await;

        if let Some(channel) = channel {
            channel
                .send_message(http, |m| m.content(message))
                .await
                .map_err(|e| error!("{:?}", e))
                .ok();
        }
    }

    pub async fn send_accountability_request(&self, request: AccountabilityRequest) {
        let http = self.client.cache_and_http.http.as_ref();
        let channel = self.create_dm(request.requested_user as u64).await;

        let task = Task::get(&self.db_client, request.task_id)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();

        if let (Some(task), Some(channel)) = (task, channel) {
            channel
                .send_message(http, |m| {
                    m.embed(|emb| {
                        emb.title("Accountability Request")
                            .description(format!(
                                "<@{:?}> has requested you as an accountability partner.",
                                request.requesting_user
                            ))
                            .field("Task", task.title, false)
                            .url(format!(
                                "{}/accountability?task={:?}",
                                self.env.shamebot_url, task.id,
                            ))
                    })
                })
                .await
                .map_err(|e| error!("{:?}", e))
                .ok();
        }
    }

    pub async fn send_task(&self, task_id: Uuid) {
        let task = Task::get(&self.db_client, task_id)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();

        let guild = Guild::get(&self.db_client, self.env.discord_guild as i64)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();

        if let (Some(task), Some(guild)) = (task, guild) {
            let checkbox = match task.checked {
                true => ":white_check_mark:",
                false => ":white_large_square:",
            };
            let owner = format!("for <@{:?}>", task.user_id);
            let mut desc = format!("");

            if let Some(content) = task.content {
                desc = format!("{}\n", content);
            }
            desc = format!("{}Finished: {}\n\n{}", desc, &checkbox, &owner);
            let url = format!("{}/tasks/{}", self.env.shamebot_url, task.id);
            let channel_id = guild.send_to.unwrap_or_default();
            ChannelId(channel_id as u64)
                .send_message(self.client.cache_and_http.http.as_ref(), |m| {
                    m.embed(|emb| emb.title(task.title).description(desc).url(url))
                })
                .await
                .map_err(|e| error!("{:?}", e))
                .ok();
        }
    }

    pub async fn send_list(&mut self, list_id: Uuid) {
        let list = List::get(&self.db_client, list_id)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();

        let tasks = List::get_tasks(&self.db_client, list_id)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();

        let guild = Guild::get(&mut self.db_client, self.env.discord_guild as i64)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();

        if let (Some(list), Some(tasks), Some(guild)) = (list, tasks, guild) {
            let owner = format!("for <@{:?}>", list.user_id);
            let url = format!("{}/lists/{}", self.env.shamebot_url, list.id);
            let channel_id = guild.send_to.unwrap_or_default();
            ChannelId(channel_id as u64)
                .send_message(self.client.cache_and_http.http.as_ref(), |m| {
                    m.embed(|emb| {
                        emb.title(list.title);

                        for task in tasks {
                            let checkbox = match task.checked {
                                true => ":white_check_mark:",
                                false => ":white_large_square:",
                            };

                            let mut desc = format!("");

                            if let Some(content) = task.content {
                                desc = format!("{}\n", content);
                            }

                            desc = format!("{}Finished: {}", desc, checkbox);

                            emb.field(task.title, desc, false);
                        }

                        emb.field("Owner", owner, false).url(url)
                    })
                })
                .await
                .map_err(|e| error!("{:?}", e))
                .ok();
        }
    }

    pub async fn send_reminder(&self, task_id: Uuid) {
        let task = Task::get(&self.db_client, task_id)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();

        let guild = Guild::get(&self.db_client, self.env.discord_guild as i64)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();

        if let (Some(task), Some(guild)) = (task, guild) {
            if task.checked {
                return;
            }

            let channel_id = guild.send_to.unwrap_or_default();
            ChannelId(channel_id as u64)
                .send_message(self.client.cache_and_http.http.as_ref(), |m| {
                    m.content(format!(
                        "hey <@{:?}>! you have _one hour_ to finish the following task:\n",
                        task.user_id,
                    ))
                })
                .await
                .map_err(|e| error!("{:?}", e))
                .ok();
        }
    }

    pub async fn send_overdue_notice(&self, task_id: Uuid) {
        let task = Task::get(&self.db_client, task_id)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();

        let request = AccountabilityRequest::get(&self.db_client, task_id)
            .await
            .map_err(|e| error!("{:?}", e));

        let guild = Guild::get(&self.db_client, self.env.discord_guild as i64)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();

        if let (Some(task), Some(guild)) = (task, guild) {
            if task.checked {
                return;
            }

            let channel_id = guild.send_to.unwrap_or_default();
            ChannelId(channel_id as u64)
                .send_message(self.client.cache_and_http.http.as_ref(), |m| {
                    let mut message = format!(
                        "your time to complete {} is up, <@{:?}>. i am very disappointed in you.",
                        task.title, task.user_id,
                    );

                    if let Ok(Some(request)) = request {
                        message = format!(
                            "{}\n\n<@{:?}>, how could you let this happen?",
                            message, request.requested_user,
                        );
                    }

                    m.content(message)
                })
                .await
                .map_err(|e| error!("{:?}", e))
                .ok();
        }
    }

    pub async fn send_pester_message(&self, task_id: Uuid) {
        let task = Task::get(&self.db_client, task_id)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();

        let request = AccountabilityRequest::get(&self.db_client, task_id)
            .await
            .map_err(|e| error!("{:?}", e));

        let guild = Guild::get(&self.db_client, self.env.discord_guild as i64)
            .await
            .map_err(|e| error!("{:?}", e))
            .ok();

        if let (Some(task), Some(guild)) = (task, guild) {
            if task.checked {
                return;
            }

            let channel_id = guild.send_to.unwrap_or_default();
            ChannelId(channel_id as u64)
                .send_message(self.client.cache_and_http.http.as_ref(), |m| {
                    let mut message = format!(
                        "hey <@{:?}>! {} still isn't finished yet >:c",
                        task.user_id,
                        task.title,
                    );

                    if let Ok(Some(request)) = request {
                        if request.status == RequestStatus::Accepted {
                            message = format!(
                                "{}\n<@{:?}> would be _very_ upset with you if you didn't finish on time.",
                                message,
                                request.requested_user,
                            );
                        }
                    }

                    if let Some(due_at) = task.due_at {
                        message = format!(
                            "{}\n\nyou have until <t:{:?}>. use your time wisely.",
                            message,
                            due_at,
                        );
                    }

                    m.content(message)
                })
                .await
                .map_err(|e| error!("{:?}", e))
                .ok();
        }
    }
}
