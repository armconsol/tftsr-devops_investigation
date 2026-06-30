//! Browser → RDP input translation.
//!
//! The frontend canvas reports input as JSON messages over the per-session
//! WebSocket. This module decodes those messages ([`RawInputEvent`]) and maps
//! browser `KeyboardEvent.code` values to RDP **scancode set 1** values so the
//! session layer can drive IronRDP's fast-path input.
//!
//! Extended keys (arrows, right-hand modifiers, the navigation cluster, numpad
//! Enter/`/`) carry the `0xE000` prefix that `ironrdp_input::Scancode::from_u16`
//! interprets as the extended flag.

use serde::Deserialize;

/// A single input event sent by the browser canvas.
///
/// The `type` discriminator matches the payloads produced by
/// `RemoteDesktopPage.tsx` (`mouse`, `mouse_move`, `wheel`, `keyboard`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RawInputEvent {
    /// A mouse button transition at a position (RDP-resolution coordinates).
    Mouse {
        x: i32,
        y: i32,
        button: u16,
        pressed: bool,
    },
    /// Pointer movement (RDP-resolution coordinates).
    MouseMove { x: i32, y: i32 },
    /// Vertical wheel rotation. `delta` follows the DOM sign convention
    /// (positive = scroll down / towards the user).
    Wheel { x: i32, y: i32, delta: i32 },
    /// A key transition identified by `KeyboardEvent.code`.
    Keyboard { code: String, pressed: bool },
}

impl RawInputEvent {
    /// Parse a single event from a JSON WebSocket text frame.
    pub fn from_json(text: &str) -> Option<Self> {
        serde_json::from_str(text).ok()
    }
}

/// Clamp a browser coordinate into the unsigned 16-bit range RDP expects.
pub fn clamp_coord(value: i32) -> u16 {
    value.clamp(0, u16::MAX as i32) as u16
}

/// Map a browser [`KeyboardEvent.code`] to an RDP scancode-set-1 value.
///
/// Returns `None` for codes we do not translate (rare/media keys) so the caller
/// can simply ignore them.
///
/// [`KeyboardEvent.code`]: https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/code
pub fn scancode_for_code(code: &str) -> Option<u16> {
    let sc: u16 = match code {
        // Letters
        "KeyA" => 0x1E,
        "KeyB" => 0x30,
        "KeyC" => 0x2E,
        "KeyD" => 0x20,
        "KeyE" => 0x12,
        "KeyF" => 0x21,
        "KeyG" => 0x22,
        "KeyH" => 0x23,
        "KeyI" => 0x17,
        "KeyJ" => 0x24,
        "KeyK" => 0x25,
        "KeyL" => 0x26,
        "KeyM" => 0x32,
        "KeyN" => 0x31,
        "KeyO" => 0x18,
        "KeyP" => 0x19,
        "KeyQ" => 0x10,
        "KeyR" => 0x13,
        "KeyS" => 0x1F,
        "KeyT" => 0x14,
        "KeyU" => 0x16,
        "KeyV" => 0x2F,
        "KeyW" => 0x11,
        "KeyX" => 0x2D,
        "KeyY" => 0x15,
        "KeyZ" => 0x2C,

        // Number row
        "Digit1" => 0x02,
        "Digit2" => 0x03,
        "Digit3" => 0x04,
        "Digit4" => 0x05,
        "Digit5" => 0x06,
        "Digit6" => 0x07,
        "Digit7" => 0x08,
        "Digit8" => 0x09,
        "Digit9" => 0x0A,
        "Digit0" => 0x0B,

        // Symbols / punctuation (US layout positions)
        "Minus" => 0x0C,
        "Equal" => 0x0D,
        "BracketLeft" => 0x1A,
        "BracketRight" => 0x1B,
        "Backslash" => 0x2B,
        "Semicolon" => 0x27,
        "Quote" => 0x28,
        "Backquote" => 0x29,
        "Comma" => 0x33,
        "Period" => 0x34,
        "Slash" => 0x35,

        // Whitespace / editing
        "Escape" => 0x01,
        "Backspace" => 0x0E,
        "Tab" => 0x0F,
        "Enter" => 0x1C,
        "Space" => 0x39,

        // Modifiers / locks
        "CapsLock" => 0x3A,
        "ShiftLeft" => 0x2A,
        "ShiftRight" => 0x36,
        "ControlLeft" => 0x1D,
        "AltLeft" => 0x38,
        "NumLock" => 0x45,
        "ScrollLock" => 0x46,

        // Function keys
        "F1" => 0x3B,
        "F2" => 0x3C,
        "F3" => 0x3D,
        "F4" => 0x3E,
        "F5" => 0x3F,
        "F6" => 0x40,
        "F7" => 0x41,
        "F8" => 0x42,
        "F9" => 0x43,
        "F10" => 0x44,
        "F11" => 0x57,
        "F12" => 0x58,

        // Numpad (non-extended)
        "Numpad0" => 0x52,
        "Numpad1" => 0x4F,
        "Numpad2" => 0x50,
        "Numpad3" => 0x51,
        "Numpad4" => 0x4B,
        "Numpad5" => 0x4C,
        "Numpad6" => 0x4D,
        "Numpad7" => 0x47,
        "Numpad8" => 0x48,
        "Numpad9" => 0x49,
        "NumpadMultiply" => 0x37,
        "NumpadSubtract" => 0x4A,
        "NumpadAdd" => 0x4E,
        "NumpadDecimal" => 0x53,

        // Extended keys (0xE000 prefix => Scancode::from_u16 sets `extended`)
        "ControlRight" => 0xE01D,
        "AltRight" => 0xE038,
        "MetaLeft" => 0xE05B,
        "MetaRight" => 0xE05C,
        "ContextMenu" => 0xE05D,
        "Insert" => 0xE052,
        "Delete" => 0xE053,
        "Home" => 0xE047,
        "End" => 0xE04F,
        "PageUp" => 0xE049,
        "PageDown" => 0xE051,
        "ArrowUp" => 0xE048,
        "ArrowLeft" => 0xE04B,
        "ArrowRight" => 0xE04D,
        "ArrowDown" => 0xE050,
        "NumpadEnter" => 0xE01C,
        "NumpadDivide" => 0xE035,

        _ => return None,
    };
    Some(sc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_basic_letter() {
        assert_eq!(scancode_for_code("KeyA"), Some(0x1E));
        assert_eq!(scancode_for_code("KeyZ"), Some(0x2C));
    }

    #[test]
    fn maps_digits_and_symbols() {
        assert_eq!(scancode_for_code("Digit1"), Some(0x02));
        assert_eq!(scancode_for_code("Digit0"), Some(0x0B));
        assert_eq!(scancode_for_code("Slash"), Some(0x35));
    }

    #[test]
    fn maps_control_keys() {
        assert_eq!(scancode_for_code("Enter"), Some(0x1C));
        assert_eq!(scancode_for_code("Backspace"), Some(0x0E));
        assert_eq!(scancode_for_code("Tab"), Some(0x0F));
        assert_eq!(scancode_for_code("Space"), Some(0x39));
        assert_eq!(scancode_for_code("Escape"), Some(0x01));
    }

    #[test]
    fn extended_keys_carry_e000_prefix() {
        for code in [
            "ArrowUp",
            "ArrowDown",
            "ArrowLeft",
            "ArrowRight",
            "Delete",
            "Home",
            "End",
            "PageUp",
            "PageDown",
            "ControlRight",
            "AltRight",
            "NumpadEnter",
            "NumpadDivide",
        ] {
            let sc = scancode_for_code(code).unwrap_or_else(|| panic!("missing {code}"));
            assert_eq!(sc & 0xE000, 0xE000, "{code} should be extended");
        }
    }

    #[test]
    fn non_extended_keys_have_no_prefix() {
        for code in ["KeyA", "Enter", "Numpad5", "F1", "ShiftLeft"] {
            let sc = scancode_for_code(code).unwrap();
            assert_ne!(sc & 0xE000, 0xE000, "{code} must not be extended");
        }
    }

    #[test]
    fn unknown_code_is_none() {
        assert_eq!(scancode_for_code("MediaPlayPause"), None);
        assert_eq!(scancode_for_code(""), None);
    }

    #[test]
    fn clamps_coordinates() {
        assert_eq!(clamp_coord(-5), 0);
        assert_eq!(clamp_coord(0), 0);
        assert_eq!(clamp_coord(1024), 1024);
        assert_eq!(clamp_coord(70_000), u16::MAX);
    }

    #[test]
    fn parses_mouse_button_event() {
        let ev =
            RawInputEvent::from_json(r#"{"type":"mouse","x":10,"y":20,"button":0,"pressed":true}"#)
                .unwrap();
        assert_eq!(
            ev,
            RawInputEvent::Mouse {
                x: 10,
                y: 20,
                button: 0,
                pressed: true
            }
        );
    }

    #[test]
    fn parses_mouse_move_event() {
        let ev = RawInputEvent::from_json(r#"{"type":"mouse_move","x":3,"y":4}"#).unwrap();
        assert_eq!(ev, RawInputEvent::MouseMove { x: 3, y: 4 });
    }

    #[test]
    fn parses_wheel_event() {
        let ev = RawInputEvent::from_json(r#"{"type":"wheel","x":1,"y":2,"delta":-120}"#).unwrap();
        assert_eq!(
            ev,
            RawInputEvent::Wheel {
                x: 1,
                y: 2,
                delta: -120
            }
        );
    }

    #[test]
    fn parses_keyboard_event() {
        let ev = RawInputEvent::from_json(r#"{"type":"keyboard","code":"KeyA","pressed":false}"#)
            .unwrap();
        assert_eq!(
            ev,
            RawInputEvent::Keyboard {
                code: "KeyA".to_string(),
                pressed: false
            }
        );
    }

    #[test]
    fn rejects_garbage_json() {
        assert!(RawInputEvent::from_json("not json").is_none());
        assert!(RawInputEvent::from_json(r#"{"type":"unknown"}"#).is_none());
    }
}
