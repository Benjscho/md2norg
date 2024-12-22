# md2norg: a Markdown to Neorg Converter

This Rust script converts Markdown files to Neorg (.norg) format. It provides 
options for recursive directory processing, output to a new directory, and replacing original files.

It's been really quickly hacked together and not tested that much so don't 
trust it. Please let me know if you find any bugs with a minimal repro in issues.

## Features

- Convert Markdown headings, code blocks, lists, and todos to Neorg format
- Output to a new directory
- Process dirs recursively

## Installation

### Option 1: Install from GitHub

1. Make sure you have Rust installed on your system. If not, you can install it
   from [https://www.rust-lang.org/](https://www.rust-lang.org/).

2. Install the converter directly from GitHub using cargo:

```bash
cargo install --git https://github.com/benjscho/md2norg.git
```

3. The binary will be installed in your Cargo bin directory. Make sure this directory is in your PATH.

4. You can now run the converter using:

```bash
md2norg [OPTIONS]
```

### Option 2: Clone and Build

1. Make sure you have Rust installed on your system. If not, you can install it
   from [https://www.rust-lang.org/](https://www.rust-lang.org/).

2. Clone this repository:

```bash
git clone https://github.com/benjscho/md2norg.git
cd md2norg
```

3. Install the cli:

```bash
cargo install --path ./
```

## Uninstallation

To uninstall the converter, run:

```bash
cargo uninstall md2norg
```

## Usage

Run the converter using the following command:

```bash
md2norg --help
```

### Examples

1. Convert files to a new directory:

```bash
md2norg --input /path/to/markdown/files --output /path/to/output/directory
```

2. Convert files recursively:

```bash
md2norg --input /path/to/markdown/files --output /path/to/output/directory --recursive
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

