pub enum Role {
    System,
    User,
    Assistant,
}

pub struct Message {
    pub content: String,
    pub role: Role,
}

pub struct Conversation {
    pub messages: Vec<Message>,
}

pub fn with_user(user_prompt: String) -> Conversation {
    Conversation {
        messages: vec![Message {
            content: user_prompt,
            role: Role::User,
        }],
    }
}

pub fn with_system(user_prompt: String, system_prompt: String) -> Conversation {
    Conversation {
        messages: vec![
            Message {
                content: user_prompt,
                role: Role::User,
            },
            Message {
                content: system_prompt,
                role: Role::System,
            },
        ],
    }
}
