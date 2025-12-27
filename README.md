# llmwrap

Command line tool to translate natural language requests into
shell commands. Works with OpenAI Responses API.

Inspired from https://www.npmjs.com/package/ezff

## Usage

```
# set OpenAI API key env variable in your shell
export LLMWRAP_OPENAI_API_KEY=sk-xxx >> ~/.bashrc

llmwrap <tool> <what to do>
```

## Example with ffmeg

```
$ llmwrap ffmpeg convert video.mp4 to gif

Proposed ffmpeg command:
ffmpeg -i "video.mp4" -vf "fps=10,scale=320:-1:flags=lanczos" -loop 0 "video.gif"

Run this command? [y/N]:
```

## Example with tar

```
$ llmwrap tar extract dump.tar.gz

Proposed command:
tar -xzf 'dump.tar.gz'

Run this command? [y/N]:
```

## License

GNU Affero General Public License Version 3.0 only.
