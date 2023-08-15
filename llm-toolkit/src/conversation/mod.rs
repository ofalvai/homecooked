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

impl Conversation {
    pub fn new(user_prompt: String) -> Self {
        Self {
            messages: vec![Message {
                content: user_prompt,
                role: Role::User,
            }],
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }
}
