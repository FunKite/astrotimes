# AI Insights Guide

Use local Ollama integration for AI-powered astronomical interpretations.

## What Are AI Insights?

AI insights provide narrative summaries of astronomical data using a local language model running on your computer. This feature interprets raw data and provides context about the sky.

## Prerequisites

- **Ollama** - Download from https://ollama.com/
- **Installed LLM** - Any Ollama model (llama2 recommended)
- **~4GB RAM** - Minimum for model execution

## Setup

### 1. Install Ollama

```bash
# macOS/Linux/Windows
# Download from https://ollama.com/

# After installation, verify
ollama --version
```

### 2. Start Ollama

```bash
ollama serve
# Listens on http://localhost:11434
```

### 3. Install a Model

```bash
# In another terminal
ollama pull llama2   # Recommended (4GB)
ollama pull neural-chat  # Lighter (3.8GB)
ollama pull mistral  # Fast (5GB)
```

List available models:
```bash
ollama list
```

## Using AI Insights

### Command Line

```bash
solunatus --city "New York" --ai-insights

# With specific model
solunatus --city "Boston" --ai-insights --ai-model "llama2"

# Custom server
solunatus --city "Paris" --ai-insights --ai-server "http://192.168.1.100:11434"

# Adjust refresh interval (1-60 minutes)
solunatus --city "London" --ai-insights --ai-refresh-minutes 5
```

### Interactive Mode

1. Run: `solunatus --city "Your City"`
2. Press `a` to configure AI
3. Enable insights
4. Select model
5. Set refresh interval
6. Press Enter to save

## Features

- **Automatic refresh** - Updates narrative every N minutes
- **Error recovery** - Shows last valid insight if network fails
- **Configurable models** - Use any Ollama model
- **Flexible timing** - Refresh from 1 to 60 minutes
- **No cloud** - Runs entirely on your computer

## Output

AI insights appear in:

- **Text output** - `--ai-insights` flag with text display
- **Watch mode** - Separate panel below astronomical data
- **JSON output** - Included in API response

Example insight:
```
AI Insights (llama2, updated 2 min ago):
Tonight marks the approach of the full moon, a time when the lunar 
disc reaches its maximum brightness across Earth's night sky. The moon 
will rise in the east after sunset and climb steadily higher throughout 
the evening, perfect for lunar observation with binoculars or a telescope.
```

## Troubleshooting

### "Connection refused"
- Verify Ollama is running: `ollama serve`
- Check server address matches: `http://localhost:11434`

### "Model not found"
- Install model: `ollama pull llama2`
- Use model name exactly: `--ai-model "llama2"`

### Slow responses
- Reduce refresh interval: `--ai-refresh-minutes 10`
- Use faster model: `ollama pull neural-chat`
- Close other applications

### High CPU usage
- Use lighter model
- Increase refresh interval
- Run on separate computer

## Performance Notes

Model sizes and typical response times:
- **neural-chat** (3.8GB) - ~3 seconds
- **mistral** (5GB) - ~4 seconds
- **llama2** (4GB) - ~5 seconds
- **llama3** (8GB) - ~8 seconds

## Disabling AI Insights

```bash
# Don't use --ai-insights flag
solunatus --city "New York"

# Or in watch mode, press 'a' and disable
```

## Advanced Usage

### Run Ollama on Different Computer

```bash
# On Ollama server machine
ollama serve  # Listens on all interfaces by default

# On solunatus machine
solunatus --city "Paris" --ai-insights --ai-server "http://192.168.1.100:11434"
```

### Use Custom Model

```bash
ollama pull mistral
solunatus --city "London" --ai-insights --ai-model "mistral"
```

### Batch Processing with Insights

```bash
#!/bin/bash
for city in "New York" "London" "Tokyo"; do
  solunatus --city "$city" --ai-insights --json >> data.jsonl
done
```

## Privacy Note

- All processing happens locally on your computer
- No data is sent to external servers
- Ollama runs as a local service only
- You maintain complete control over data

## See Also

- **[Ollama Documentation](https://ollama.com/)** - Model selection and setup
- **[CLI Reference](cli-reference.md)** - Command-line options
- **[Interactive Mode](interactive-mode.md)** - Watch mode features
