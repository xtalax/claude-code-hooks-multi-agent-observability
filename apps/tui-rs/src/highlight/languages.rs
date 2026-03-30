/// Known commands and language keywords for syntax highlighting.

/// Commands recognized as "known" in bash — highlighted brighter (White palette).
const LANG_COMMANDS: &[&str] = &[
    // Python
    "python", "python3", "pip", "pip3", "uv", "poetry", "pytest", "mypy", "ruff", "black", "isort",
    // Julia
    "julia",
    // JavaScript / TypeScript
    "node", "npm", "npx", "bun", "deno", "yarn", "pnpm", "tsc", "tsx", "esbuild", "vite",
    // Java
    "java", "javac", "gradle", "mvn", "maven", "ant",
    // C / C++
    "gcc", "g++", "cc", "clang", "clang++", "make", "cmake", "meson", "ninja",
    // C#
    "dotnet", "csc", "msbuild", "nuget",
    // Common tools
    "git", "docker", "cargo", "go", "ruby", "perl", "bash", "sh", "zsh",
    "curl", "wget", "ssh", "scp", "rsync", "tar", "zip", "unzip",
    "cat", "ls", "cd", "mv", "cp", "rm", "mkdir", "chmod", "chown", "ln",
    "grep", "rg", "sed", "awk", "find", "sort", "uniq", "wc", "head", "tail", "xargs",
    "echo", "printf", "export", "source", "sudo", "apt", "pacman", "brew",
    "mise", "asdf", "rustup", "ghc", "stack", "cabal",
];

/// Language keywords (union of Python, JS, Java, C, Rust).
const KEYWORDS: &[&str] = &[
    // Python
    "def", "class", "if", "elif", "else", "for", "while", "return",
    "import", "from", "as", "with", "try", "except", "finally", "raise",
    "yield", "lambda", "pass", "break", "continue", "and", "or", "not",
    "in", "is", "True", "False", "None", "print", "self", "async", "await",
    // JavaScript
    "function", "const", "let", "var", "new", "this", "catch", "throw",
    "typeof", "instanceof", "undefined", "null", "require", "module",
    // Java / C#
    "public", "private", "protected", "interface", "extends", "implements",
    "static", "final", "void", "boolean", "package", "namespace", "using",
    // C / C++
    "int", "char", "float", "double", "long", "short", "unsigned",
    "signed", "extern", "struct", "union", "enum", "typedef", "sizeof",
    "switch", "case", "goto", "include", "define", "ifdef", "endif",
    "template", "typename", "virtual", "override", "auto", "constexpr",
    "nullptr", "delete",
    // Rust
    "fn", "pub", "mod", "use", "crate", "impl", "trait", "where",
    "match", "loop", "ref", "mut", "move", "dyn", "type", "unsafe",
];

pub fn is_lang_command(word: &str) -> bool {
    let lower = word.to_ascii_lowercase();
    LANG_COMMANDS.iter().any(|&c| c == lower)
}

pub fn is_keyword(word: &str) -> bool {
    KEYWORDS.iter().any(|&k| k == word)
}

/// Detect which language mode a command implies (for inline code highlighting).
pub fn lang_mode_for(command: &str) -> &'static str {
    match command.to_ascii_lowercase().as_str() {
        "python" | "python3" => "python",
        "julia" => "julia",
        "node" | "bun" | "deno" => "javascript",
        "ruby" => "ruby",
        "perl" => "perl",
        _ => "",
    }
}
