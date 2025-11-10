import { AgentRuntime } from '@/types/worktree';

export interface RuntimeDefinition {
  id: AgentRuntime;
  label: string;
  description: string;
  defaultModel: string;
  modelOptions: string[];
  emoji: string;
  envVar: string;
}

export const RUNTIME_DEFINITIONS: RuntimeDefinition[] = [
  {
    id: 'claude',
    label: 'Claude Code',
    description: 'Anthropic Claude Code CLI 実行',
    defaultModel: 'sonnet',
    modelOptions: ['sonnet', 'haiku', 'opus'],
    emoji: '🤖',
    envVar: 'ANTHROPIC_API_KEY',
  },
  {
    id: 'codex',
    label: 'Codex (OpenAI)',
    description: 'CodeX CLI / GPT-4 実行',
    defaultModel: 'gpt-4',
    modelOptions: ['gpt-4', 'gpt-4-turbo', 'gpt-3.5-turbo', 'code-davinci-002'],
    emoji: '🧠',
    envVar: 'OPENAI_API_KEY',
  },
  {
    id: 'gemini',
    label: 'Gemini CLI',
    description: 'Google Gemini 1.5 Pro 実行',
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
