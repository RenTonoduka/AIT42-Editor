import { AgentRuntime } from '@/types/worktree';

export interface RuntimeDefinition {
  id: AgentRuntime;
  label: string;
  description: string;
  defaultModel: string;
  modelOptions: string[];
  emoji: string;
  envVar?: string; // Optional: 既存のCLI認証を優先、なければ環境変数を参照
}

export const RUNTIME_DEFINITIONS: RuntimeDefinition[] = [
  {
    id: 'claude',
    label: 'Claude Code',
    description: 'claude CLIログイン済みなら使用可能',
    defaultModel: 'sonnet',
    modelOptions: ['sonnet', 'haiku', 'opus'],
    emoji: '🤖',
    envVar: 'ANTHROPIC_API_KEY',
  },
  {
    id: 'codex',
    label: 'Codex (OpenAI)',
    description: 'chatgpt/openai CLIインストール済みなら使用可能',
    defaultModel: 'gpt-4',
    modelOptions: ['gpt-4', 'gpt-4-turbo', 'gpt-3.5-turbo', 'code-davinci-002'],
    emoji: '🧠',
    envVar: 'OPENAI_API_KEY',
  },
  {
    id: 'gemini',
    label: 'Gemini CLI',
    description: 'gemini CLIインストール済み (v0.13.0+) なら使用可能',
    defaultModel: 'gemini-1.5-pro',
    modelOptions: ['gemini-1.5-pro', 'gemini-1.5-flash'],
    emoji: '✨',
    envVar: 'GOOGLE_AI_API_KEY',
  },
];

export const getRuntimeDefinition = (id: AgentRuntime): RuntimeDefinition => {
  const runtime = RUNTIME_DEFINITIONS.find((def) => def.id === id);
  if (!runtime) {
    throw new Error(`Unknown runtime: ${id}`);
  }
  return runtime;
};
