/// Used for the Device Inquiry message
pub enum DeviceIdQuery {
    /// Send the Device Inquiry request to a specific device id
    Specific(u8),
    /// Send the Device Inquiry request to all devices
    Any,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct DeviceInquiry {
    device_id: u8,
    family_code: u16,
    family_member_code: u16,
    firmware_revision: u32,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct VersionInquiry {
    bootloader_version: u32,
    firmware_version: u32,
    bootloader_size: u16,
}

pub(crate) fn request_device_inquiry<T>(
    output: &mut T,
    query: DeviceIdQuery,
) -> Result<(), crate::MidiError>
where
    T: crate::OutputDevice,
{
    const QUERY_DEVICE_ID_FOR_ANY: u8 = 127;

    let query_device_id = match query {
        DeviceIdQuery::Specific(device_id) => {
            assert_ne!(device_id, QUERY_DEVICE_ID_FOR_ANY);
            device_id
        }
        DeviceIdQuery::Any => QUERY_DEVICE_ID_FOR_ANY,
    };

    output.send(&[240, 126, query_device_id, 6, 1, 247])
}

pub(crate) fn request_version_inquiry<T>(output: &mut T) -> Result<(), crate::MidiError>
where
    T: crate::OutputDevice,
{
    output.send(&[240, 0, 32, 41, 0, 112, 247])
}

pub(crate) fn parse_device_query(data: &[u8]) -> Option<DeviceInquiry> {
    if let &[240, 126, device_id, 6, 2, 0, 32, 41, fc1, fc2, fmc1, fmc2, fr1, fr2, fr3, fr4, 247] =
        data
    {
        let family_code = u16::from_be_bytes([fc1, fc2]);
        let family_member_code = u16::from_be_bytes([fmc1, fmc2]);

        let firmware_revision = fr1 as u32 * 1000 + fr2 as u32 * 100 + fr3 as u32 * 10 + fr4 as u32;

        Some(DeviceInquiry {
            device_id,
            family_code,
            family_member_code,
            firmware_revision,
        })
    } else {
        None
    }
}

pub(crate) fn parse_version_query(data: &[u8]) -> Option<VersionInquiry> {
    if let &[240, 0, 32, 41, 0, 112, bl1, bl2, bl3, bl4, fw1, fw2, fw3, fw4, bs1, bs2, 247] = data {
        let bootloader_version = data[0] as u32 * 10000
            + bl1 as u32 * 1000
            + bl2 as u32 * 100
            + bl3 as u32 * 10
            + bl4 as u32;

        let firmware_version = data[5] as u32 * 10000
            + fw1 as u32 * 1000
            + fw2 as u32 * 100
            + fw3 as u32 * 10
            + fw4 as u32;

        let bootloader_size = u16::from_be_bytes([bs1, bs2]);

        Some(VersionInquiry {
            bootloader_version,
            firmware_version,
            bootloader_size,
        })
    } else {
        None
    }
}
