# Home-cooked software

Inspired by [Robin Sloan](https://www.robinsloan.com/notes/home-cooked-app/).

What do I mean by [home-cooked software](https://oliverfalvai.com/notes/home-cooked-software/)?

> Unlike general-purpose software appealing to the widest audience, home-cooked software is purpose-built to solve my own niche problems. It's not designed to scale or solve everyone's problem and that's okay!

At the moment, all tools in this repo are written is Rust. To install any of the CLI tools, just [set up Rust](https://www.rust-lang.org/tools/install) and run `cargo --locked install --path .`.

### List of my homecooked software

- **embeddings**: A CLI tool for exploring [embeddings](https://platform.openai.com/docs/guides/embeddings/what-are-embeddings) of Markdown notes (such as your [Obsidian](https://obsidian.md) vault). Once the embeddings are computed and stored locally for all documents, you can do:

    - semantic search using arbitrary queries
    - explore related notes of a given note
    - plot embeddings in 2D and view the similarity of notes

- **gardener**: Various utilities for an Obsidian vault full of Markdown files. Current commands:
    - `export`: Convert a folder of Obsidian notes to plain Markdown syntax, but only the files having the `tags: [public]` frontmatter attribute. It also sets the `title` frontmatter attribute based on the file name
    - `clean`: Clean up unreferenced attachments (images and other files) after exporting a subset of notes with the above command

- **speedtest-to-influx**: A wrapper around the official [Speedtest CLI](https://www.speedtest.net/apps/cli) that prints a nice colorful summary to stdout and also sends the results to an InfluxDB instance. It can also run the speedtest continuously on a schedule.
