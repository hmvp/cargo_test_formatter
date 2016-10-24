# Convert `cargo test` output to something a CI server recognises

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://travis-ci.org/hmvp/cargo_test_formatter.svg?branch=master)](https://travis-ci.org/hmvp/cargo_test_formatter)
[![Clippy Linting Result](https://clippy.bashy.io/github/hmvp/cargo_test_formatter/master/badge.svg)](https://clippy.bashy.io/github/hmvp/cargo_test_formatter/master/log)


# Use 

```
cargo test | cargo_test_formatter > report.xml
```

## Features

Here are the current and planned features, with their status:
- [ ] **Parse all cargo output**: It kind of works but needs cleanup
- [ ] **Export to junit format**: 
- [ ] **Export to multiple formats**: Choose the output format
