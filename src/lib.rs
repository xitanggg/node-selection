#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use arboard::{ Clipboard, ImageData };
use std::{ thread, time };

#[cfg(target_os = "macos")]
use core_graphics::{
    event::{ CGEvent, CGEventTapLocation, KeyCode, CGEventFlags },
    event_source::{ CGEventSource, CGEventSourceStateID },
};

#[cfg(not(target_os = "macos"))]
use enigo::{ Direction::{ Click, Press, Release }, Enigo, Key, Keyboard, Settings };

static DEFAULT_TIME_OUT_MS: u32 = 80;

/// Returns the current selection text. If there is no selection text, returns an empty string.
///
/// The selection text is retrieved through a 6-step process:
/// 1. Save clipboard existing text or image
/// 2. Clear clipboard
/// 3. Simulate `Ctrl + C` (`Cmd + C` in Mac) keyboard input to copy selection text to clipboard
/// 4. Poll clipboard to retrieve selection text in a loop every 1ms. The loop breaks if the
///    selection text is found or it times out after 80ms by default
/// 5. Restore clipboard previous text or image to minimize side effects to users
/// 6. Return selection text as the result
///
/// ##### Arguments
/// * `timeOutMs` - An optional number that sets the max time to wait for selection text to
///                 appear in the clipboard during clipboard polling. Default to 80ms. Can be
///                 adjusted lower or higher depending on OS and use case. Smaller selection text
///                 is faster to copy while larger selection text takes longer.
/// * `printTimeToCopy` - An optional boolean that if set to true, print the time taken to copy
///                       selection text to clipboard to console. Default to false. Useful for
///                       debugging and adjusting `timeOutMs`.
#[napi]
pub fn get_selection_text(time_out_ms: Option<u32>, print_time_to_copy: Option<bool>) -> String {
    let mut clipboard = Clipboard::new().unwrap();

    // 1. Save clipboard existing text or image
    let clipboard_text = clipboard.get_text().unwrap_or(String::new());
    let clipboard_image = clipboard
        .get_image()
        .unwrap_or(ImageData { width: 0, height: 0, bytes: [].as_ref().into() });

    // 2. Clear clipboard
    clipboard.clear().unwrap();

    // 3. Simulate `Ctrl + C` (`Cmd + C` in Mac) keyboard input to copy selection text to clipboard
    copy();

    // 4. Poll clipboard to retrieve selection text in a loop every 1ms. The loop breaks if the
    //    selection text is found or it times out after 80ms by default
    let start_time = time::Instant::now();
    let time_out_ms_in_u128 = u128::from(time_out_ms.unwrap_or(DEFAULT_TIME_OUT_MS));
    let mut selection_text = String::new();

    while start_time.elapsed().as_millis() < time_out_ms_in_u128 {
        selection_text = clipboard.get_text().unwrap_or(String::new());
        if !selection_text.is_empty() {
            if print_time_to_copy.unwrap_or(false) {
                println!(
                    "Time taken to copy selection text to clipboard: {}ms",
                    start_time.elapsed().as_millis()
                );
            }
            break;
        }
        thread::sleep(time::Duration::from_millis(1));
    }

    // 5. Restore clipboard previous text or image to minimize side effects to users
    let should_restore_clipboard_text = !clipboard_text.is_empty();
    let should_restore_clipboard_image = clipboard_image.width > 0;
    if should_restore_clipboard_text {
        clipboard.set_text(&clipboard_text).unwrap();
    } else if should_restore_clipboard_image {
        clipboard.set_image(clipboard_image).unwrap();
    }

    // 6. Return selection text as the result
    return selection_text;
}

/// Simulate `Ctrl + C` (`Cmd + C` in Mac) keyboard input to copy selection text to clipboard
///
/// Useful for those who would like to implement custom logics to save and restore clipboard state
/// or just want to perform copy
///
/// As for implementation, Windows calls into Enigo to simulate keyboard input. But for Mac, it calls
/// into Mac's Core Graphics CGEvent library directly to work around 2 issues with Enigo's current
/// implementation, which causes additional delay (https://github.com/enigo-rs/enigo/issues/105)
/// and subjects to mouse movement/keyboard interruption (https://github.com/enigo-rs/enigo/issues/201).
/// Calling into CGEvent and setting event flag solves both issues.
#[napi]
pub fn copy() {
    _copy();
}

// Define CG key code for "c" key
// Reference: https://github.com/phracker/MacOSX-SDKs/blob/master/MacOSX10.13.sdk/System/Library/Frameworks/Carbon.framework/Versions/A/Frameworks/HIToolbox.framework/Versions/A/Headers/Events.h#L206
#[cfg(target_os = "macos")]
static C_KEY_CODE: u16 = 0x08;

#[cfg(target_os = "macos")]
fn _copy() {
    // Implementation reference: https://stackoverflow.com/questions/2008126/cgeventpost-possible-bug-when-simulating-keyboard-events

    // Event source state id reference: https://developer.apple.com/documentation/coregraphics/cgeventsourcestateid
    let event_source_state_id = CGEventSourceStateID::CombinedSessionState;
    let event_source = CGEventSource::new(event_source_state_id).unwrap();
    // Event tap location reference: https://developer.apple.com/documentation/coregraphics/cgeventtaplocation
    let event_tap_location = CGEventTapLocation::HID;

    let press_cmd_c_event = CGEvent::new_keyboard_event(
        event_source.clone(),
        C_KEY_CODE,
        true
    ).unwrap();
    press_cmd_c_event.set_flags(CGEventFlags::CGEventFlagCommand); // Set flags to Cmd
    press_cmd_c_event.post(event_tap_location);

    let release_c_event = CGEvent::new_keyboard_event(
        event_source.clone(),
        C_KEY_CODE,
        false
    ).unwrap();
    release_c_event.set_flags(CGEventFlags::CGEventFlagNull); // Reset flags to null
    release_c_event.post(event_tap_location);

    // Release Cmd Key for completeness. May or may not be necessary
    // given Apple's documentation is not clear on this.
    let release_cmd_event = CGEvent::new_keyboard_event(
        event_source.clone(),
        KeyCode::COMMAND,
        false
    ).unwrap();
    release_cmd_event.post(event_tap_location);
}

#[cfg(not(target_os = "macos"))]
fn _copy() {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    enigo.key(Key::Control, Press).unwrap();
    enigo.key(Key::Unicode('c'), Click).unwrap();
    enigo.key(Key::Control, Release).unwrap();
}
