use std::convert::TryInto;

use super::Button;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
/// A Launchpad MK2 input message
pub enum Message {
    /// A button was pressed
    Press { button: Button },
    /// A button was released
    Release { button: Button },
    /// Emitted after a text scroll was initiated
    TextEndedOrLooped,
    /// The response to a [device inquiry request](super::Output::request_device_inquiry)
    DeviceInquiry {
        device_id: u8,
        firmware_revision: u32,
        family_code: u16,
        family_member_code: u16,
    },
    /// The response to a [version inquiry request](super::Output::request_version_inquiry)
    VersionInquiry {
        bootloader_version: u32,
        firmware_version: u32,
        bootloader_size: u16,
    },
    /// Emitted when a fader was changed by the user, in [fader mode](super::Output::enter_fader_mode)
    FaderChange { index: u8, value: u8 },
}

/// The Launchpad MK2 input connection creator.
pub struct Input;

fn decode_short_message(data: &[u8]) -> Message {
    // first byte of a launchpad midi message is the message type
    match data {
        // Note on
        &[0x90, button, velocity] => {
            let button = decode_grid_button(button);

            match velocity {
                0 => return Message::Release { button },
                127 => return Message::Press { button },
                other => panic!("Unexpected grid note-on velocity {}", other),
            }
        }
        // Controller change
        &[0xB0, number @ 104..=111, velocity] => {
            let button = Button::ControlButton {
                index: number - 104,
            };

            match velocity {
                0 => return Message::Release { button },
                127 => return Message::Press { button },
                other => panic!("Unexpected control note-on velocity {}", other),
            }
        }
        // Fader change
        &[0xB0, number @ 21..=28, velocity] => {
            return Message::FaderChange {
                index: number - 21,
                value: velocity,
            };
        }
        // YES we have no note off message handler here because it's not used by the launchpad.
        // It sends zero-velocity note-on messages instead.
        other => panic!("Unexpected midi message: {:?}", other),
    }
}

fn decode_sysex_message(data: &[u8]) -> Message {
    match data {
        // Device-specific message types
        &[240, 0, 32, 41, 2, 24, 21, 247] => Message::TextEndedOrLooped,

        // Common message types
        &[240, 126, device_id, 6, 2, 0, 32, 41, fc1, fc2, fmc1, fmc2, fr1, fr2, fr3, fr4, 247] => {
            let family_code = u16::from_be_bytes([fc1, fc2]);
            let family_member_code = u16::from_be_bytes([fmc1, fmc2]);

            let firmware_revision =
                fr1 as u32 * 1000 + fr2 as u32 * 100 + fr3 as u32 * 10 + fr4 as u32;

            Message::DeviceInquiry {
                device_id,
                family_code,
                family_member_code,
                firmware_revision,
            }
        }
        &[240, 0, 32, 41, 0, 112, ref data @ .., bs1, bs2, 247] => {
            let data: [u8; 12] = data
                .try_into()
                .expect("Invalid version inquiry response length");

            let bootloader_version = data[0] as u32 * 10000
                + data[1] as u32 * 1000
                + data[2] as u32 * 100
                + data[3] as u32 * 10
                + data[4] as u32;

            let firmware_version = data[5] as u32 * 10000
                + data[6] as u32 * 1000
                + data[7] as u32 * 100
                + data[8] as u32 * 10
                + data[9] as u32;

            let bootloader_size = u16::from_be_bytes([bs1, bs2]);

            Message::VersionInquiry {
                bootloader_version,
                firmware_version,
                bootloader_size,
            }
        }
        other => panic!("Unexpected sysex message: {:?}", other),
    }
}

fn decode_grid_button(btn: u8) -> Button {
    let x = (btn % 10) - 1;
    let y = 8 - (btn / 10);
    return Button::GridButton { x, y };
}

impl crate::InputDevice for Input {
    const MIDI_DEVICE_KEYWORD: &'static str = "Launchpad MK2";
    const MIDI_CONNECTION_NAME: &'static str = "Launchy Mk2 Input";
    type Message = Message;

    fn decode_message(_timestamp: u64, data: &[u8]) -> Message {
        if data.len() == 3 {
            return decode_short_message(data);
        } else {
            return decode_sysex_message(data);
        }
    }
}
