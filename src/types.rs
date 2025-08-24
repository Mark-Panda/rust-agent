// OpenAI相关类型现在由async-openai包提供
// 我们只保留Agent特有的类型

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Thought {
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct Action {
    pub tool_call: ToolCall,
}

#[derive(Debug, Clone)]
pub struct Observation {
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct FinalAnswer {
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum AgentStep {
    Thought(Thought),
    Action(Action),
    Observation(Observation),
    FinalAnswer(FinalAnswer),
}

#[derive(Debug, Clone)]
pub struct AgentState {
    pub messages: Vec<async_openai::types::ChatCompletionRequestMessage>,
    pub current_step: Option<AgentStep>,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            current_step: None,
        }
    }
}
