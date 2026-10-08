# I'm a Curly Finder spec

A file search engine for VJs: search for a clip during a show and drop it into Resolume.
This file is the source of truth for the technical spec (tag rules, query syntax, API).

## Principles

- Tags live only in paths and file names. There is no external database or dictionary file.
- The tool has no rules specific to any one collection. How to classify files is up to the data (folder names and file names).
- The index is not built ahead of time. It is built in memory at startup.

## Tags

| Source | Rule |
|---|---|
| `{...}` | Anywhere in a folder or file name. One tag per `{}`. Spaces around the contents are trimmed |
| `{...}` on a folder | Every file below the folder inherits it. A file cannot cancel it |
| Full-width `｛｝` | Counts as a tag, the same as `{}`. lint asks you to make them half-width |

- Folder names outside `{}` are not tags. You can still find them with a plain word (substring match).
  For a folder you want to match exactly (e.g. to tell `game-a` from `game-a-tour`), write `{game-a}`.
- Ignored (all of these show up in lint):
  - An unclosed `{`: ignored from there to the end (`song{so{lo}` has no tag)
  - Nested `{...}`: ignored along with the outer pair (`song{so{lo}}{3D}` has only `3D`)
  - An empty `{}`
- The same tag on both a folder and a file is counted once.

## Scanning

- Every file under the root is indexed. Folders themselves are not.
- `$RECYCLE.BIN` and `System Volume Information` (any case, at any depth) are never scanned.
- Files whose names start with `._` (resource forks that macOS leaves behind) are not indexed.
- Folders under the root that cannot be read are skipped and listed in `Index::skipped`.
  Only a root that cannot be read is an error.

## Normalization

Before matching, paths, tags and query words are all normalized like this:

1. NFKC (full-width letters and digits → half-width, half-width katakana → full-width, `！` → `!`, etc.)
2. Hiragana → katakana
3. Lower case

## Query syntax

| Syntax | Meaning |
|---|---|
| `word` | Substring match on the path relative to the root, with the `{tags}` removed (a word never matches inside a tag) |
| `#tag` | Exact match on a tag |
| `a b` | AND (separated by spaces; full-width spaces work too) |
| `a\|b` | OR. **Binds tighter than AND** (`#solo #3D\|#2D` means `#solo AND (#3D OR #2D)`) |
| `-a` | NOT. Binds tightest (`-a\|b` means `(NOT a) OR b`) |
| `( )` | Grouping |
| `"..."` | A word with spaces or symbols. Works for tags too, like `#"Blue Sky"` |

- `-` and `#` are operators only at the start of a word (`Blue-Sky` is one word).
- An empty query matches everything.
- Errors: unbalanced `(` and `)` / an unclosed `"` / no word after `|`, `-` or `#` (including `# song`)
- To search for a file name that contains `(`, `)` or `|`, put the word in `"..."`.

## Library API (for a GUI)

```rust
use curly_finder::index::{Index, SharedIndex};
use curly_finder::query::Query;

let shared = SharedIndex::new(Index::scan(root)?);  // scan(root)
let index = shared.snapshot();                       // Arc<Index>
index.query("#solo #3D|#2D")?;                       // Vec<&Entry> (in path order)
let query = Query::parse("opening")?.and_tags(["3D"]); // opening AND #3D, built on the parsed query
index.matching(&query);                              // Vec<&Entry> (in path order)
index.tags();                                        // Vec<TagCount> (most used first, with spelling variants)
index.lint();                                        // Vec<LintItem> (in path order, each folder once)
shared.rescan()?;                                    // rebuilds, then swaps in the new index
```

- `Entry`: `path` (absolute) / `rel` / `dir` / `name` / `stem` / `ext` / `tags`
- `TagCount`: `tag` (the most common spelling) / `count` (number of files) / `spellings` (count per spelling that normalizes to the same tag)
- `LintItem`: `rel` / `issues` (`full_width_brace` / `unclosed` / `stray_close` / `nested` / `empty`)
- `SharedIndex::rescan` scans outside the lock. While it runs, you can keep searching the index you got from `snapshot()`.
  How to run it in the background is up to the GUI.

## CLI

```
curly --root <DIR> query <QUERY> [--json]
curly --root <DIR> tags [--json]
curly --root <DIR> lint [--json]
curly --root <DIR> serve
```

- `--root` can also be given with the `CURLY_ROOT` environment variable.
- `query` prints one absolute path per line. With `--json`, it prints an array of `Entry`.
- `tags` prints `count<TAB>tag[<TAB>(spelling ×count, ...)]`. `lint` prints `relative path<TAB>issue, ...`.
- A broken query prints `curly: query: <message>` to stderr and exits with code 1.
- Each folder that could not be read is reported as `curly: could not read: <relative path>` on stderr. The command still succeeds.

## Serve mode

`curly serve` scans the root once, then answers requests until stdin closes (exit code 0).
Meant for an app that runs it as a child process, so it does not rescan for every query.
Each request is one line of JSON on stdin. Each response is one line of JSON on stdout, flushed right away.
Requests are answered one at a time, in order. `id` (any JSON value) is echoed back.

```
→ {"id": 12, "op": "query", "q": "opening #3D|#2D", "limit": 500}
← {"id": 12, "ok": true, "total": 34, "entries": [<Entry>, ...]}
→ {"id": 13, "op": "tags"}
← {"id": 13, "ok": true, "tags": [<TagCount>, ...]}
→ {"id": 14, "op": "rescan"}
← {"id": 14, "ok": true, "total": 13733}
→ {"id": 15, "op": "ping"}
← {"id": 15, "ok": true}
← {"id": 16, "ok": false, "error": "unbalanced ( and )"}
```

| op | Answer |
|---|---|
| `query` | `entries`: the matching `Entry`s in path order (the same objects as `query --json`), cut at `limit` (default: no limit). `total`: the number of matches before the cut |
| `tags` | `tags`: the same objects as `tags --json` |
| `rescan` | Scans the root again (`SharedIndex::rescan`). `total`: the number of entries in the new index |
| `ping` | Nothing but `ok` |

- On an error the answer is `{"id": ..., "ok": false, "error": "<message>"}`, and the server keeps serving:
  - A broken query: the message of the query error (`unbalanced ( and )` etc.)
  - `query` without `q`: `query: missing q`
  - An unknown op: `unknown op: <op>`
  - A line that is not JSON: `invalid JSON: ...` with `"id": null`. JSON that is not a request (e.g. no `op`): `invalid request: ...`, with the `id` if it has one
  - A rescan that fails: `rescan: ...`. The old index stays in use
- A scan error at startup and unreadable folders are reported on stderr, as with the other commands.
