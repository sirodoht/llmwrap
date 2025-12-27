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

## Example with tar

```
$ llmwrap tar extract archive.tar.gz

Proposed command:
tar -xf archive.tar.gz

Run this command? [Y/n]:
Executing: tar -xf archive.tar.gz
```

## Example with ffmpeg

```
$ llmwrap convert video.mp4 to gif

Proposed command:
ffmpeg -i "video.mp4" -vf "fps=10,scale=320:-1:flags=lanczos" -loop 0 "video.gif"

Run this command? [Y/n]:
```

## License

GNU Affero General Public License Version 3.0 only.
