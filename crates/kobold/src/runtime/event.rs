// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use crate::internal::delegate_event;

macro_rules! declare {
    ($($variant:ident,)*) => {
        pub enum EventKind {
            $( $variant, )*
        }

        impl EventKind {
            const COUNT: u32 = 0 $(+ { let _ = EventKind::$variant; 1 })*;
        }
    }
}

// Must be kept in sync with util.js
declare! {
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

pub struct UsedEvents(u32);

impl UsedEvents {
    pub const fn empty() -> Self {
        UsedEvents(0)
    }

    pub const fn used(mut self, kind: EventKind) -> Self {
        self.0 |= 1 << kind as u32;
        self
    }

    pub const fn combine(self, other: Self) -> Self {
        UsedEvents(self.0 | other.0)
    }

    pub(crate) fn delegate(self) {
        delegate_event(0);
        delegate_event(15);

        for i in 0..EventKind::COUNT {
            if self.0 & (1 << i) > 0 {
                delegate_event(i as _);
            }
        }
    }
}
