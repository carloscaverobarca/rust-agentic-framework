export interface Message {
  role: 'User' | 'Assistant' | 'Tool';
  content: string;
  name?: string;
}

export interface PredictStreamRequest {
  session_id: string;
  messages: Message[];
}

export interface ToolUsageEvent {
  tool: string;
  args: Record<string, any>;
  duration_ms: number;
  result: string;
}

export interface SSEEvent {
  event: 'assistant_output' | 'tool_usage';
  data: string | ToolUsageEvent;
}

export interface ChatState {
  messages: Message[];
  isLoading: boolean;
  isStreaming: boolean;
  currentResponse: string;
}