import React from 'react';
import { Message, ToolUsageEvent } from '../types';

interface ChatMessageProps {
  message: Message;
  toolUsage?: ToolUsageEvent;
}

export const ChatMessage: React.FC<ChatMessageProps> = ({ message, toolUsage }) => {
  const getMessageClass = () => {
    switch (message.role) {
      case 'User':
        return 'chat-message user-message';
      case 'Assistant':
        return 'chat-message assistant-message';
      case 'Tool':
        return 'chat-message tool-message';
      default:
        return 'chat-message';
    }
  };

  const formatContent = (content: string) => {
    return content.split('\n').map((line, index) => (
      <React.Fragment key={index}>
        {line}
        {index < content.split('\n').length - 1 && <br />}
      </React.Fragment>
    ));
  };

  return (
    <div className={getMessageClass()}>
      <div className="message-header">
        <span className="message-role">{message.role}</span>
        {message.name && (
          <span className="message-name">({message.name})</span>
        )}
      </div>
      <div className="message-content">
        {formatContent(message.content)}
      </div>
      {toolUsage && (
        <div className="tool-usage-info">
          <div className="tool-usage-header">
            🔧 Tool: {toolUsage.tool} ({toolUsage.duration_ms}ms)
          </div>
          <div className="tool-usage-args">
            <strong>Args:</strong> {JSON.stringify(toolUsage.args, null, 2)}
          </div>
        </div>
      )}
    </div>
  );
};