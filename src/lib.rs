// I know explicitly returning at the end of functions is not idiomatic, but I prefer it personally.
// Also, I use tabs everywhere and I don't agree with clippy's reasoning against tabs in doc
// comments, so I will keep using tabs in doc comments
#![allow(clippy::needless_return, clippy::tabs_in_doc_comments)]


/*!
An interfacing library for your Novation Launchpads, providing both low-level access as well as
powerful high-level abstractions.

# High-level access through the Canvas API

High-level access is possible through the [Canvas] API. The [`Canvas`] trait simply represents a grid of
LEDs that can be written to, and consequently flushed to the actual hardware. A number of high-level
utilities make the Canvas API a pleasure to use.

```rust
use launchy::{CanvasMessage, Color};

let (mut canvas, input_poller) = launchy::s::Canvas::guess_polling()?;

for msg in input_poller.iter() {
	match msg {
		CanvasMessage::Press { pad } => canvas[pad] = Color::WHITE,
		CanvasMessage::Release { pad } => canvas[pad] = Color::BLACK,
	}
	canvas.flush()?;
}
```
The above `match` statement could also be written in a more concise way:
```rust
canvas[msg.pad()] = if msg.is_press() { Color::WHITE } else { Color::BLACK };
```

# Low-level access

Low-level access is provided via the `Input` and `Output` structs. Care has been taken to ensure
that this library has 100% coverage of the Launchpads' functionality: all documented Launchpad
features are accessible in Launchy's low-level API.

This means you get the real deal: rapid LED updates, double buffering capabilities, device and
firmware information.

Furthermore, we know that a low-level API is not the
place for hidden abstractions - and that's why every function in Launchy's low-level APIs
corresponds to exactly one MIDI message (unless noted otherwise in the documentation). That way, the
user has fine control over the data that's actually being sent.

## Using double-buffering to produce a continuous red flash
```rust
use launchy::Color;
use launchy::s::{DoubleBuffering, DoubleBufferingBehavior, Buffer};

let mut output = launchy::s::Output::guess()?;

// Start editing buffer A
output.control_double_buffering(DoubleBuffering {
	copy: false,
	flash: false,
	edited_buffer: Buffer::A,
	displayed_buffer: Buffer::B,
});

// Light all buttons red, using the rapid update feature - just 40 midi messages
for _ in 0..40 {
	output.set_button_rapid(
		Color::RED, DoubleBufferingBehavior::None,
		Color::RED, DoubleBufferingBehavior::None,
	)
}

// Now buffer A is completely red and B is empty. Let's leverage the Launchpad S flash
// feature to continually flash between both buffers, producing a red flash:
output.control_double_buffering(DoubleBuffering {
	copy: false,
	flash: true, // this is the important bit
	edited_buffer: Buffer::A,
	displayed_buffer: Buffer::A,
});
```
*/

pub mod util;

mod protocols;

#[macro_use]
mod canvas;
pub use canvas::*;

mod midi_io;
pub use midi_io::*;

mod errors;
pub use errors::*;

pub mod generic;

pub mod launchpad_s;
pub use launchpad_s as s;

pub mod launchpad_mini;
pub use launchpad_mini as mini;

pub mod launchpad_mk2;
pub use launchpad_mk2 as mk2;

pub mod launch_control;
pub use launch_control as control;
/// The MIDI API of the classic Launch Control and the Launch Control XL is identical
pub use launch_control as launch_control_xl;
pub use launch_control as control_xl;

pub mod prelude {
	pub use crate::midi_io::{OutputDevice, InputDevice, MsgPollingWrapper};
	pub use crate::canvas::{Canvas, Color, Pad};
}

/// Identifier used for e.g. the midi port names etc.
const APPLICATION_NAME: &str = "Launchy";
