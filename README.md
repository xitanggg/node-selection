# 🖱️Node Selection

This package provides a simple Node.js util that allows you to retrieve user's current selection text on desktop.

- Support Windows and Mac
  - Mac: Need to grant accessibility permission to the calling app (`Settings` -> `Privacy & Security` -> `Accessibility`)
- Require Node.js >= 10

## 📦Installation

```bash
npm i @xitanggg/node-selection
```

## ℹ️Usage

**Common Usage**

```typescript
import { getSelectionText } from '@xitanggg/node-selection;

const selectionText = getSelectionText();
```

**Customized Usage**

`getSelectionText` accepts an optional `copyWaitTimeMs` input argument, which sets how long to wait after performing the copy operation before reading the clipboard text. It defaults to 5ms, which works for most use cases with small selection text. However, a larger value would be needed to support use case for large selection text that takes longer to copy.

```typescript
import { getSelectionText } from '@xitanggg/node-selection;

const LONG_COPY_WAIT_TIME_MS = 10;
const selectionText = getSelectionText(LONG_COPY_WAIT_TIME_MS);
```

## 💡Implementation

**Core Logic**

The implementation is written in Rust and is ~10 lines of code (see `/src/lib.rs` for full source code)

The selection text is retrieved in a 3 steps processes:

1. Save clipboard existing text and clear clipboard
2. Simulate `Ctrl + C` (`Cmd + C` in Mac) keyboard input to copy selection text to clipboard
3. Read clipboard to retrieve selection text and return it as result (the previous clipboard text is restored before returning to minimize side effects to users)

**Dependency**

It uses [Arboard (Arthur's Clipboard)](https://github.com/1Password/arboard) to perform clipboard operation and [enigo](https://github.com/enigo-rs/enigo) to perform keyboard input simulation

**Build & Distribution**

It uses [NAPI-RS](https://github.com/napi-rs/napi-rs) to compile the Rust source code into binaries via GitHub actions, package it as a Node-API native addon, and then publish it to npm for distribution and easy use.

One very nice thing about the NAPI-RS tooling is that the binary has been built, so this package just works after installation, i.e. no need to build it yourself. Also, the binary is selectively installed, meaning installation only installs the binary that your system needs, e.g. windows or Mac, to keep the size small instead of including all binaries at once.
