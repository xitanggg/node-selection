# 🖱️Node Selection

This package provides a simple Node.js util that allows you to retrieve user's current selection text on desktop.

- Support Windows and Mac
  - Mac: Need to grant accessibility permission to the calling app (`Settings` -> `Privacy & Security` -> `Accessibility`) to call Apple's [Core Graphics framework](https://developer.apple.com/documentation/coregraphics) to post [CGEvent](https://developer.apple.com/documentation/coregraphics/cgevent) to simulate copy operation
- Require Node.js >= 10

## 📦Installation

```bash
npm i @xitanggg/node-selection
```

## ℹ️Usage

**Common Usage**

```typescript
import { getSelectionText } from '@xitanggg/node-selection';

const selectionText = getSelectionText();
```

**Custom Usage**

`getSelectionText` accepts an optional `timeOutMs` as its first input argument, which sets the max time to wait for selection text to appear in the clipboard during clipboard polling. It defaults to 80ms, and can be adjusted lower or higher depending on OS and use case. Smaller selection text is faster to copy while larger selection text takes longer.

```typescript
import { getSelectionText } from '@xitanggg/node-selection';

const LONGER_COPY_TIME_OUT_MS = 100;
const selectionText = getSelectionText(LONGER_COPY_TIME_OUT_MS);
```

There is also an optional `printTimeToCopy` as its second input argument, which defaults to `false`, but if set to `true`, prints the time taken to copy selection text to clipboard to console. It can be useful for debugging and adjusting `timeOutMs` based on OS and use case.

```typescript
const PRINT_TIME_TO_COPY = true;
const selectionText = getSelectionText(
	LONGER_COPY_TIME_OUT_MS,
	PRINT_TIME_TO_COPY
);
```

## 💡Implementation

**Core Logic**

The implementation is written in Rust and is ~150 lines of code in a single file `/src/lib.rs`.

The selection text is retrieved in a 6-step process:

1. Save clipboard existing text or image
2. Clear clipboard
3. Simulate `Ctrl + C` (`Cmd + C` in Mac) keyboard input to copy selection text to clipboard
4. Poll clipboard to retrieve selection text in a loop every 1ms. The loop breaks if the selection text is found or it times out after 80ms by default
5. Restore clipboard previous text or image to minimize side effects to users
6. Return selection text as the result

**Dependencies**

For clipboard operation, it uses [Arboard (Arthur's Clipboard)](https://github.com/1Password/arboard).

For keyboard input simulation, it uses [enigo](https://github.com/enigo-rs/enigo) in Windows, and [core-foundation-rs's core-graphics](https://github.com/servo/core-foundation-rs) in Mac.

Arboard supports getting and setting clipboard text and image, which should satisfy most use cases. But it is worth noting that it doesn't support other clipboard contents, e.g. html, rtf, file. A `copy` method is provided for those who would like to implement custom logics to save and restore clipboard state or just want to call `Ctrl + C` (`Cmd + C` in Mac) to perform copy.

```typescript
import { copy } from '@xitanggg/node-selection';

// Skip custom logics to save clipboard state
copy();
// Skip custom logics to poll clipboard and restore clipboard state
```

**Build & Distribution**

It uses [NAPI-RS](https://github.com/napi-rs/napi-rs) to compile the Rust source code into binaries via GitHub actions, package it as a Node-API native addon, and then publish it to npm for distribution and easy use.

One very nice thing about the NAPI-RS tooling is that the binary has been built, so this package just works after installation, i.e. no need to build it yourself. Also, the binary is selectively installed, meaning installation only installs the binary that your system needs, e.g. windows or Mac, to keep the size small instead of including all binaries at once.
