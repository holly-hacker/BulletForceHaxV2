# Code Coverage
Requirements:
- Just (`cargo install just` or [install as package](https://just.systems/man/en/chapter_4.html))
- grcov (`cargo install grcov` or [download the binary](https://github.com/mozilla/grcov/releases))
- rust llvm tools (`rustup component add llvm-tools-preview`)

Generate code coverage info by running `just cover-photon` in the repo root. This will generate an HTML page at `target/coverage/html/index.html` and an `lcov.info` file in the repo root. Editor extensions such as `ryanluker.vscode-coverage-gutters` for VS Code can make use of these files to show inline coverage info.

If this method does not work on windows, try using WSL.
