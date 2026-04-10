// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::{self, Debug};

// Must be kept in sync with kobold::runtime::event
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Click,
    DblClick,
    PointerDown,
    PointerUp,
    PointerMove,
    PointerOver,
    PointerOut,
    PointerCancel,
    ContextMenu,
    Wheel,
    TouchStart,
    TouchMove,
    TouchEnd,
    TouchCancel,
    KeyDown,
    KeyUp,
    FocusIn,
    FocusOut,
    Change,
    Reset,
    Invalid,
    BeforeInput,
    Select,
}

impl Debug for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_variant())
    }
}

impl EventKind {
    pub fn as_variant(&self) -> &str {
        use EventKind::*;

        match self {
            Click => "Click",
            DblClick => "DblClick",
            PointerDown => "PointerDown",
            PointerUp => "PointerUp",
            PointerMove => "PointerMove",
            PointerOver => "PointerOver",
            PointerOut => "PointerOut",
            PointerCancel => "PointerCancel",
            ContextMenu => "ContextMenu",
            Wheel => "Wheel",
            TouchStart => "TouchStart",
            TouchMove => "TouchMove",
            TouchEnd => "TouchEnd",
            TouchCancel => "TouchCancel",
            KeyDown => "KeyDown",
            KeyUp => "KeyUp",
            FocusIn => "FocusIn",
            FocusOut => "FocusOut",
            Change => "Change",
            Reset => "Reset",
            Invalid => "Invalid",
            BeforeInput => "BeforeInput",
            Select => "Select",
        }
    }
}

impl TryFrom<&str> for EventKind {
    type Error = ();

    fn try_from(event: &str) -> Result<Self, Self::Error> {
        use EventKind::*;

        let kind = match event {
            "click" => Click,
            "dblclick" => DblClick,
            "pointerdown" => PointerDown,
            "pointerup" => PointerUp,
            "pointermove" => PointerMove,
            "poitnerover" => PointerOver,
            "poitnerout" => PointerOut,
            "pointercancel" => PointerCancel,
            "contextmenu" => ContextMenu,
            "wheel" => Wheel,
            "touchstart" => TouchStart,
            "touchmove" => TouchMove,
            "touchend" => TouchEnd,
            "touchcancel" => TouchCancel,
            "keydown" => KeyDown,
            "keyup" => KeyUp,
            "focusin" => FocusIn,
            "focusout" => FocusOut,
            "change" => Change,
            "reset" => Reset,
            "invalid" => Invalid,
            "beforeinput" => BeforeInput,
            "select" => Select,

            // aliases
            "mousedown" => PointerDown,
            "mouseup" => PointerUp,
            "mousemove" => PointerMove,
            "mouseenter" => PointerOver,
            "mouseleave" => PointerOut,
            "mouseover" => PointerOver,
            "mouseout" => PointerOut,
            "mousewheel" => Wheel,
            "pointerenter" => PointerOver,
            "pointerleave" => PointerOut,
            "keypress" => KeyUp,
            "focus" => FocusIn,
            "blur" => FocusOut,

            _ => return Err(()),
        };

        Ok(kind)
    }
}
