import { ChatOpenAI } from '@langchain/openai';
import { Tool } from '@langchain/core/tools';
import { AgentExecutor, initializeAgentExecutorWithOptions } from 'langchain/agents';
import { AgentStep } from '@langchain/core/agents';
import { BaseMessage } from '@langchain/core/messages';

interface ChatAgentConfig {
  openAIApiKey: string;
  modelName?: string;
  temperature?: number;
}

interface AgentInput {
  input: string;
  chat_history: AgentStep[];
}

export class ChatAgent {
  private tools: Tool[];
  private llm: ChatOpenAI;
  private executor!: AgentExecutor; // Marked as definitely assigned since it's initialized in factory method

  private constructor(config: ChatAgentConfig) {
    this.tools = [];
    this.llm = new ChatOpenAI({
      openAIApiKey: config.openAIApiKey,
      modelName: config.modelName || 'gpt-3.5-turbo',
      temperature: config.temperature ?? 0
    });
  }

  public static async initialize(config: ChatAgentConfig): Promise<ChatAgent> {
    if (!config.openAIApiKey) {
      throw new Error('OpenAI API key is required');
    }
    
    const instance = new ChatAgent(config);
    
    // Initialize executor properly
    instance.executor = await initializeAgentExecutorWithOptions(
      instance.tools,
      instance.llm,
      {
        agentType: "chat-conversational-react-description",
        verbose: true
      }
    );
    
    return instance;
  }

  public async run(input: string, history: AgentStep[] = []): Promise<string | BaseMessage> {
    try {
      const result = await this.executor.invoke({
        input,
        chat_history: history,
      } as AgentInput);
      return result.output;
    } catch (error: unknown) {
      const err = error as Error;
      return `Error running agent: ${err.message}`;
    }
  }
}
