import { v4 as uuidv4 } from 'uuid';
import { Message, PredictStreamRequest, ToolUsageEvent } from '../types';

export class ApiService {
  private baseUrl: string;
  private sessionId: string;

  constructor(baseUrl: string = '') {
    this.baseUrl = baseUrl;
    this.sessionId = uuidv4();
  }

  getSessionId(): string {
    return this.sessionId;
  }

  resetSession(): void {
    this.sessionId = uuidv4();
  }

  async sendMessage(
    messages: Message[],
    onAssistantOutput: (content: string) => void,
    onToolUsage: (toolUsage: ToolUsageEvent) => void,
    onError: (error: string) => void,
    onComplete: () => void
  ): Promise<void> {
    const request: PredictStreamRequest = {
      session_id: this.sessionId,
      messages,
    };

    try {
      const response = await fetch(`${this.baseUrl}/predict_stream`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Accept': 'text/event-stream',
        },
        body: JSON.stringify(request),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const reader = response.body?.getReader();
      if (!reader) {
        throw new Error('No response body reader available');
      }

      const decoder = new TextDecoder();
      let buffer = '';
      let currentEvent = '';

      try {
        while (true) {
          const { done, value } = await reader.read();
          if (done) {
            onComplete();
            break;
          }

          buffer += decoder.decode(value, { stream: true });
          const lines = buffer.split('\n');
          buffer = lines.pop() || '';

          for (const line of lines) {
            if (line.trim() === '') {
              // Empty line resets the event
              currentEvent = '';
              continue;
            }
            
            try {
              if (line.startsWith('event: ')) {
                currentEvent = line.substring(7).trim();
                continue;
              }
              
              if (line.startsWith('data: ')) {
                const data = line.substring(6).trim();
                
                if (data === '[DONE]') {
                  onComplete();
                  return;
                }

                const parsed = JSON.parse(data);
                console.log('SSE Event:', currentEvent, 'Data:', parsed); // Debug logging
                
                if (currentEvent === 'assistant_output' || currentEvent === 'content_delta') {
                  if (parsed.content !== undefined) {
                    onAssistantOutput(parsed.content);
                  }
                } else if (currentEvent === 'tool_usage') {
                  onToolUsage(parsed as ToolUsageEvent);
                } else if (currentEvent === 'stream_end') {
                  onComplete();
                  return;
                }
              }
            } catch (parseError) {
              console.warn('Failed to parse SSE line:', line, parseError);
            }
          }
        }
      } finally {
        reader.releaseLock();
      }
    } catch (error) {
      onError(error instanceof Error ? error.message : 'Unknown error occurred');
    }
  }
}