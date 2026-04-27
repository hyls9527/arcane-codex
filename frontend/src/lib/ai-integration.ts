const OLLAMA_BASE_URL = 'http://localhost:11434';
const LM_STUDIO_BASE_URL = 'http://localhost:1234';
const HERMES_BASE_URL = 'http://localhost:18789';

export interface ChatMessage {
  role: 'system' | 'user' | 'assistant' | 'tool';
  content: string;
  name?: string;
}

export interface ChatConfig {
  model: string;
  messages: ChatMessage[];
  temperature?: number;
  maxTokens?: number;
  stream?: boolean;
  toolsRaw?: Array<{ type: string; function: { name: string; description: string; parameters: Record<string, { type: string; description?: string }> } }>;
}

export interface ChatResponse {
  content: string;
  model: string;
  usage?: { prompt_tokens: number; completion_tokens: number; total_tokens: number };
  tool_calls?: Array<{ name: string; arguments: Record<string, unknown> }>;
}

export interface ToolDefinition {
  name: string;
  description: string;
  parameters: Record<string, { type: string; description?: string }>;
  execute: (args: Record<string, unknown>) => Promise<string>;
}

export interface RAGConfig {
  embeddingModel: string;
  vectorStore: 'memory' | 'chroma' | 'pinecone';
  chunkSize?: number;
  chunkOverlap?: number;
}

export interface RAGDocument {
  id: string;
  content: string;
  embedding?: number[];
  metadata?: Record<string, unknown>;
}

export interface RAGIndex {
  add: (docs: RAGDocument[]) => Promise<void>;
  search: (query: string, topK: number) => Promise<RAGDocument[]>;
}

export async function ollamaChat(config: ChatConfig): Promise<ChatResponse> {
  const response = await fetch(`${OLLAMA_BASE_URL}/v1/chat/completions`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      model: config.model,
      messages: config.messages,
      temperature: config.temperature ?? 0.7,
      max_tokens: config.maxTokens ?? 2048,
      stream: false,
    }),
  });

  if (!response.ok) {
    throw new Error(`Ollama API error: ${response.status} ${response.statusText}`);
  }

  const data = await response.json();
  return {
    content: data.choices[0]?.message?.content || '',
    model: data.model,
    usage: data.usage,
  };
}

export async function lmStudioChat(config: ChatConfig): Promise<ChatResponse> {
  const response = await fetch(`${LM_STUDIO_BASE_URL}/v1/chat/completions`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      model: config.model,
      messages: config.messages,
      temperature: config.temperature ?? 0.7,
      max_tokens: config.maxTokens ?? 4096,
    }),
  });

  if (!response.ok) {
    throw new Error(`LM Studio API error: ${response.status} ${response.statusText}`);
  }

  const data = await response.json();
  return {
    content: data.choices[0]?.message?.content || '',
    model: data.model,
    usage: data.usage,
  };
}

export async function hermesChat(config: ChatConfig): Promise<ChatResponse> {
  const response = await fetch(`${HERMES_BASE_URL}/v1/chat/completions`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      model: config.model,
      messages: config.messages,
      temperature: config.temperature ?? 0.7,
      max_tokens: config.maxTokens ?? 2048,
    }),
  });

  if (!response.ok) {
    throw new Error(`Hermes API error: ${response.status} ${response.statusText}`);
  }

  const data = await response.json();
  return {
    content: data.choices[0]?.message?.content || '',
    model: data.model,
    usage: data.usage,
  };
}

export async function zhipuChat(config: ChatConfig & { apiKey: string }): Promise<ChatResponse> {
  const response = await fetch('https://open.bigmodel.cn/api/paas/v4/chat/completions', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${config.apiKey}`,
    },
    body: JSON.stringify({
      model: 'glm-4-flash',
      messages: config.messages,
      temperature: config.temperature ?? 0.7,
      max_tokens: config.maxTokens ?? 4096,
    }),
  });

  if (!response.ok) {
    throw new Error(`Zhipu API error: ${response.status} ${response.statusText}`);
  }

  const data = await response.json();
  return {
    content: data.choices[0]?.message?.content || '',
    model: data.model,
    usage: data.usage,
  };
}

export async function openRouterChat(config: ChatConfig & { apiKey: string }): Promise<ChatResponse> {
  const response = await fetch('https://openrouter.ai/api/v1/chat/completions', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${config.apiKey}`,
    },
    body: JSON.stringify({
      model: config.model,
      messages: config.messages,
      temperature: config.temperature ?? 0.7,
      max_tokens: config.maxTokens ?? 4096,
    }),
  });

  if (!response.ok) {
    throw new Error(`OpenRouter API error: ${response.status} ${response.statusText}`);
  }

  const data = await response.json();
  return {
    content: data.choices[0]?.message?.content || '',
    model: data.model,
    usage: data.usage,
  };
}

export async function ollamaEmbed(config: { model: string; text: string }): Promise<number[]> {
  const response = await fetch(`${OLLAMA_BASE_URL}/api/embed`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      model: config.model,
      input: config.text,
    }),
  });

  if (!response.ok) {
    throw new Error(`Ollama embed error: ${response.status} ${response.statusText}`);
  }

  const data = await response.json();
  return data.embeddings[0];
}

export function cosineSimilarity(a: number[], b: number[]): number {
  const dot = a.reduce((sum, v, i) => sum + v * b[i], 0);
  const magA = Math.sqrt(a.reduce((sum, v) => sum + v * v, 0));
  const magB = Math.sqrt(b.reduce((sum, v) => sum + v * v, 0));
  return dot / (magA * magB);
}

export class MemoryVectorStore implements RAGIndex {
  private documents: RAGDocument[] = [];

  async add(docs: RAGDocument[]): Promise<void> {
    this.documents.push(...docs);
  }

  async search(query: string, topK: number): Promise<RAGDocument[]> {
    const queryEmbedding = await ollamaEmbed({ model: 'nomic-embed-text', text: query });

    const scored = this.documents
      .filter((d) => d.embedding)
      .map((d) => ({
        doc: d,
        score: cosineSimilarity(queryEmbedding, d.embedding!),
      }))
      .sort((a, b) => b.score - a.score)
      .slice(0, topK);

    return scored.map((s) => s.doc);
  }
}

export async function createRAG(config: RAGConfig): Promise<RAGIndex> {
  switch (config.vectorStore) {
    case 'memory':
      return new MemoryVectorStore();
    case 'chroma':
      throw new Error('Chroma vector store not implemented yet. Install chromadb package.');
    case 'pinecone':
      throw new Error('Pinecone vector store not implemented yet. Install @pinecone-database/pinecone package.');
    default:
      throw new Error(`Unknown vector store: ${config.vectorStore}`);
  }
}

export async function chunkAndEmbed(config: {
  text: string;
  chunkSize?: number;
  overlap?: number;
  embeddingModel?: string;
}): Promise<RAGDocument[]> {
  const chunkSize = config.chunkSize || 500;
  const overlap = config.overlap ?? 50;
  const words = config.text.split(' ');
  const chunks: string[] = [];

  for (let i = 0; i < words.length; i += chunkSize - overlap) {
    chunks.push(words.slice(i, i + chunkSize).join(''));
  }

  const docs: RAGDocument[] = [];
  for (const chunk of chunks) {
    const embedding = await ollamaEmbed({
      model: config.embeddingModel || 'nomic-embed-text',
      text: chunk,
    });
    docs.push({
      id: `chunk-${docs.length}`,
      content: chunk,
      embedding,
    });
  }

  return docs;
}

export async function searchRAG(config: {
  index: RAGIndex;
  query: string;
  topK?: number;
  llm: (context: string) => Promise<ChatResponse>;
}): Promise<ChatResponse> {
  const topK = config.topK || 3;
  const docs = await config.index.search(config.query, topK);
  const context = docs.map((d) => d.content).join('\n\n---\n\n');

  return config.llm(context);
}

export function createAgent(config: {
  model: string;
  tools: ToolDefinition[];
  maxIterations?: number;
  systemPrompt?: string;
}) {
  const maxIterations = config.maxIterations || 10;

  return {
    async run(query: string): Promise<ChatResponse> {
      let messages: ChatMessage[] = [
        ...(config.systemPrompt ? [{ role: 'system' as const, content: config.systemPrompt }] : []),
        { role: 'user', content: query },
      ];

      for (let i = 0; i < maxIterations; i++) {
        const toolsRaw = config.tools.map((t) => ({
          type: 'function' as const,
          function: { name: t.name, description: t.description, parameters: t.parameters },
        }));

        const response = await ollamaChat({
          model: config.model,
          messages,
          toolsRaw,
        } as ChatConfig);

        if (!response.tool_calls || response.tool_calls.length === 0) {
          return response;
        }

        for (const toolCall of response.tool_calls) {
          const tool = config.tools.find((t) => t.name === toolCall.name);
          if (!tool) continue;

          const result = await tool.execute(toolCall.arguments);
          messages.push({ role: 'assistant', content: response.content });
          messages.push({ role: 'tool', content: result, name: toolCall.name });
        }
      }

      return { content: 'Max iterations reached without final answer.', model: config.model };
    },
  };
}
