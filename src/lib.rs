#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use arboard::Clipboard;
use enigo::{
  Direction::{Click, Press, Release},
  Enigo, Key, Keyboard, Settings,
};
use std::{thread, time};

static DEFAULT_COPY_WAIT_TIME_MS: u32 = 5;

/// Returns the current selection text. If there is no selection text, returns an empty string.
/// 
/// The selection text is retrieved in a 3 steps processes:
/// 1. Save clipboard existing text and clear clipboard
/// 2. Simulate `Ctrl + C` (`Cmd + C` in Mac) keyboard input to copy selection text to clipboard
/// 3. Read clipboard to retrieve selection text and return it as result 
/// (the previous clipboard text is restored before returning to minimize side effects to users) 
/// 
/// ##### Arguments
/// * `copyWaitTimeMs` - An optional number that sets how long to wait after performing the copy
///                      operation before reading the clipboard text. It defaults to 5ms, which 
///                      works for most use cases with small selection text. However, a larger value 
///                      would be needed to support use case for large selection text that takes 
///                      longer to copy to the clipboard. 
#[napi]
pub fn get_selection_text(copy_wait_time_ms: Option<u32>) -> String {
  let mut clipboard = Clipboard::new().unwrap();

  // Save clipboard existing text
  let clipboard_existing_text = clipboard.get_text().unwrap_or(String::new());

  // Clear clipboard
  clipboard.clear().unwrap();

  // Simulate Ctrl/Cmd + C keyboard input to copy selection text to clipboard
  let mut enigo = Enigo::new(&Settings::default()).unwrap();
  let control_or_command_key = if cfg!(target_os = "macos") {
    Key::Meta
  } else {
    Key::Control
  };
  enigo.key(control_or_command_key, Press).unwrap();
  enigo.key(Key::Unicode('c'), Click).unwrap();
  enigo.key(control_or_command_key, Release).unwrap();
  
  // Wait for clipboard to be updated with copied selection text
  thread::sleep(time::Duration::from_millis(u64::from(copy_wait_time_ms.unwrap_or(DEFAULT_COPY_WAIT_TIME_MS))));

  // Read clipboard to retrieve selection text
  let selection_text = clipboard.get_text().unwrap_or(String::new());

  // Restore clipboard previous existing text to minimize side effects to users
  if !clipboard_existing_text.is_empty() {
    clipboard.set_text(&clipboard_existing_text).unwrap();
  }

  return selection_text;
}
