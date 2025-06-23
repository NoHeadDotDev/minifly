# MCP (Model Context Protocol) Setup

This repository uses MCP servers for enhanced development capabilities. To set up MCP servers locally:

## Setup

1. **Copy the template**:
   ```bash
   cp .mcp.json.template .mcp.json
   ```

2. **Configure your API keys**:
   Edit `.mcp.json` and replace placeholder values:
   ```json
   {
     "mcpServers": {
       "stability-ai": {
         "env": {
           "STABILITY_AI_API_KEY": "your-actual-api-key-here"
         }
       }
     }
   }
   ```

3. **Available MCP Servers**:
   - **playwright**: Browser automation and testing
   - **pickapicon-mcp**: Icon search and management (Lucide icons)
   - **stability-ai**: AI image generation (requires API key)
   - **Context7**: Code context and documentation

## Security

⚠️ **Important**: Never commit `.mcp.json` to git as it may contain API keys!

The `.mcp.json` file is already in `.gitignore` to prevent accidental commits.

## Getting API Keys

- **Stability AI**: Get your API key from [Stability AI Platform](https://platform.stability.ai/)
- Other services may require their own API keys - check their respective documentation

## Usage

Once configured, MCP servers will be available in compatible development environments that support the Model Context Protocol.