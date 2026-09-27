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

## Normalization

Before matching, paths, tags and query words are all normalized like this:

1. NFKC (full-width letters and digits → half-width, half-width katakana → full-width, `！` → `!`, etc.)
2. Hiragana → katakana
3. Lower case

## Query syntax

| Syntax | Meaning |
|---|---|
| `word` | Substring match on the whole path relative to the root |
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

let shared = SharedIndex::new(Index::scan(root)?);  // scan(root)
let index = shared.snapshot();                       // Arc<Index>
index.query("#solo #3D|#2D")?;                       // Vec<&Entry> (in path order)
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
```

- `--root` can also be given with the `CURLY_ROOT` environment variable.
- `query` prints one absolute path per line. With `--json`, it prints an array of `Entry`.
- `tags` prints `count<TAB>tag[<TAB>(spelling ×count, ...)]`. `lint` prints `relative path<TAB>issue, ...`.
- A broken query prints `curly: query: <message>` to stderr and exits with code 1.
