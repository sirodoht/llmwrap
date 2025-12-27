# llmwrap

Command line tool to translate natural-language requests into a single runnable
shell command. Work with OpenAI Responses API.

Inspired from https://www.npmjs.com/package/ezff

## Usage

```
llmwrap <tool> <what to do>
```

## Example

```
export LLMWRAP_OPENAI_API_KEY=sk-xxx
llmwrap ffmpeg convert video.mp4 to gif

Proposed ffmpeg command:
ffmpeg -i "video.mp4" -vf "fps=10,scale=320:-1:flags=lanczos" -loop 0 "video.gif"

Run this command? [y/N]:
```

## License

GNU Affero General Public License Version 3.0 only.
