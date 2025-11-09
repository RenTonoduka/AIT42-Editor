# Changelog

All notable changes to AIT42-Editor will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-11-09

### Added
- Initial release of AIT42-Editor
- Multi-agent AI system with Competition, Ensemble, and Debate modes
- Workspace-specific session history management
- Real-time agent execution monitoring
- Kanban-style session organization
- Support for up to 10 parallel agents in Competition mode
- Support for up to 3 agents in Ensemble mode
- Support for 3-role Debate mode (Architect, Pragmatist, Innovator)
- Session filtering and sorting capabilities
- Worktree-based agent isolation
- Git repository integration

### Fixed
- Session history workspace isolation bug (sessions no longer appear when no workspace is open)
- Frontend and backend validation for workspace paths

### Security
- Input validation for workspace paths
- Prevention of sessions with empty workspace paths
