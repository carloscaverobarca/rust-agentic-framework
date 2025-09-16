# Agentic ChatBot Web GUI

A modern React TypeScript web interface for the Agentic FAQ Assistant. This application provides a clean, responsive chat interface that connects to the Rust backend via Server-Sent Events (SSE) for real-time streaming responses.

## Features

- **Real-time streaming responses** via SSE
- **Tool usage visualization** with detailed information
- **Session management** with unique session IDs
- **Modern chat interface** with message bubbles
- **Responsive design** for mobile and desktop
- **Loading states and indicators**
- **Error handling and display**

## Architecture

- **React 18** with TypeScript
- **Vite** for fast development and building
- **Server-Sent Events** for real-time communication
- **CSS3** with modern styling and animations
- **UUID** for session management

## Getting Started

### Prerequisites

- Node.js 16+
- npm or yarn
- Running Rust backend server on port 3000

### Installation

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

The development server will start on `http://localhost:5173` (or the next available port like 5174) with proxy configuration to forward API requests to the Rust backend at `http://localhost:3000`.

## Project Structure

```
src/
├── components/
│   ├── ChatBot.tsx          # Main chat component with state management
│   ├── ChatMessage.tsx      # Individual message display component
│   └── MessageInput.tsx     # User input component with send functionality
├── services/
│   └── apiService.ts        # SSE service for backend communication
├── types/
│   └── index.ts             # TypeScript type definitions
├── styles/
│   └── ChatBot.css          # Modern chat interface styling
├── App.tsx                  # Root application component
└── main.tsx                 # React application entry point
```

## API Integration

The web GUI connects to the Rust backend's `/predict_stream` endpoint using:

- **POST requests** with JSON payload containing session_id and messages
- **Server-Sent Events** for streaming responses
- **Event types**: `assistant_output` (text streaming) and `tool_usage` (tool execution info)

## Styling

The interface features:

- **Gradient headers** with modern color schemes
- **Message bubbles** with distinct styling for users, assistants, and tools
- **Smooth animations** for message appearance and streaming indicators
- **Responsive layout** that works on mobile and desktop
- **Tool usage cards** with syntax highlighting for arguments
- **Loading states** with visual feedback

## Development

The application uses modern React patterns:

- **Functional components** with hooks
- **TypeScript** for type safety
- **CSS custom properties** for theming
- **Error boundaries** for graceful error handling
- **Optimistic UI updates** for better user experience

## Configuration

The Vite configuration includes:

- **Proxy setup** to forward `/predict_stream` requests to the backend
- **React plugin** for JSX support
- **TypeScript compilation** with strict mode
- **Development server** on port 3000
