# CLAUDE.md - Guidelines for Agentic Coding in Listen Interface

## Build/Development Commands
- `bun run dev` - Start development server
- `bun run build` - Build for production (runs TypeScript compiler first)
- `bun run lint` - Run ESLint on codebase
- `bun run preview` - Preview production build

## Code Style Guidelines
- **TypeScript**: Strict typing with no unused variables/parameters
- **React**: Functional components with hooks, JSX format
- **Imports**: Group in order: React, external libraries, internal modules, components, styles
- **Naming**: PascalCase for components, camelCase for variables/functions
- **Error Handling**: Use try/catch blocks with proper error logging
- **State Management**: Zustand for global state, React hooks for component state
- **Styling**: Tailwind CSS for all styling, follow component-specific pattern
- **File Structure**: Feature-based organization with clear separation of concerns
- **Component Pattern**: Small, focused components with clear props interfaces
- **Documentation**: JSDoc comments for complex functions and components

## Project Tech Stack
- React 18 with TypeScript
- Tanstack Router for routing
- Zustand for state management
- Tailwind for styling
- Bun as the JavaScript runtime
- Vite for build tooling