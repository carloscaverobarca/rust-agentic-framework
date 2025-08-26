import React, { useState, useRef, useEffect } from 'react';
import { Message, ToolUsageEvent } from '../types';
import { ApiService } from '../services/apiService';
import { ChatMessage } from './ChatMessage';
import { MessageInput } from './MessageInput';

export const ChatBot: React.FC = () => {
  const [messages, setMessages] = useState<Message[]>([]);
  const [currentResponse, setCurrentResponse] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [isStreaming, setIsStreaming] = useState<boolean>(false);
  const [toolUsages, setToolUsages] = useState<Map<number, ToolUsageEvent>>(new Map());
  const [error, setError] = useState<string | null>(null);
  
  const apiService = useRef(new ApiService());
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages, currentResponse]);

  const handleSendMessage = async (content: string) => {
    if (isLoading || isStreaming) return;

    const userMessage: Message = {
      role: 'User',
      content,
    };

    const updatedMessages = [...messages, userMessage];
    setMessages(updatedMessages);
    setCurrentResponse('');
    setError(null);
    setIsLoading(true);
    setIsStreaming(true);

    let accumulatedResponse = '';

    // Add a timeout to prevent getting stuck
    const timeoutId = setTimeout(() => {
      console.warn('SSE stream timeout, resetting state');
      setIsLoading(false);
      setIsStreaming(false);
      setError('Request timed out. Please try again.');
    }, 30000); // 30 second timeout

    try {
      await apiService.current.sendMessage(
        updatedMessages,
        (content: string) => {
          accumulatedResponse += content;
          setCurrentResponse(accumulatedResponse);
        },
        (toolUsage: ToolUsageEvent) => {
          const toolMessage: Message = {
            role: 'Tool',
            content: toolUsage.result,
            name: toolUsage.tool,
          };
          setMessages(prev => [...prev, toolMessage]);
          setToolUsages(prev => new Map(prev).set(prev.size, toolUsage));
        },
        (errorMessage: string) => {
          clearTimeout(timeoutId);
          setError(errorMessage);
          setIsLoading(false);
          setIsStreaming(false);
        },
        () => {
          clearTimeout(timeoutId);
          console.log('SSE stream completed, accumulated response:', accumulatedResponse);
          if (accumulatedResponse.trim()) {
            const assistantMessage: Message = {
              role: 'Assistant',
              content: accumulatedResponse,
            };
            setMessages(prev => [...prev, assistantMessage]);
          }
          setCurrentResponse('');
          setIsLoading(false);
          setIsStreaming(false);
        }
      );
    } catch (err) {
      clearTimeout(timeoutId);
      setError(err instanceof Error ? err.message : 'An error occurred');
      setIsLoading(false);
      setIsStreaming(false);
    }
  };

  const handleNewChat = () => {
    setMessages([]);
    setCurrentResponse('');
    setToolUsages(new Map());
    setError(null);
    setIsLoading(false);
    setIsStreaming(false);
    apiService.current.resetSession();
  };

  const handleReset = () => {
    console.log('Manual reset triggered');
    setCurrentResponse('');
    setError(null);
    setIsLoading(false);
    setIsStreaming(false);
  };

  return (
    <div className="chatbot-container">
      <div className="chatbot-header">
        <h1>Agentic FAQ Assistant</h1>
        <div className="header-actions">
          <span className="session-id">
            Session: {apiService.current.getSessionId().slice(0, 8)}...
          </span>
          {(isLoading || isStreaming) && (
            <button
              onClick={handleReset}
              className="new-chat-button"
              style={{ backgroundColor: '#f56565', marginRight: '0.5rem' }}
            >
              Reset
            </button>
          )}
          <button
            onClick={handleNewChat}
            className="new-chat-button"
            disabled={isLoading && !isStreaming}
          >
            New Chat
          </button>
        </div>
      </div>

      <div className="messages-container">
        {messages.length === 0 && (
          <div className="welcome-message">
            <h2>👋 Welcome to the FAQ Assistant!</h2>
            <p>Ask me anything about the company policies, procedures, or general questions.</p>
            <p>I can also use tools like file summarization to help provide better answers.</p>
          </div>
        )}

        {messages.map((message, index) => (
          <ChatMessage
            key={index}
            message={message}
            toolUsage={toolUsages.get(index)}
          />
        ))}

        {isStreaming && currentResponse && (
          <div className="chat-message assistant-message streaming">
            <div className="message-header">
              <span className="message-role">Assistant</span>
              <span className="streaming-indicator">✍️ Writing...</span>
            </div>
            <div className="message-content">
              {currentResponse.split('\n').map((line, index) => (
                <React.Fragment key={index}>
                  {line}
                  {index < currentResponse.split('\n').length - 1 && <br />}
                </React.Fragment>
              ))}
              <span className="cursor">|</span>
            </div>
          </div>
        )}

        {error && (
          <div className="error-message">
            <strong>Error:</strong> {error}
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      <MessageInput
        onSendMessage={handleSendMessage}
        disabled={isLoading || isStreaming}
        placeholder={isLoading ? "Processing..." : "Ask me anything..."}
      />
    </div>
  );
};