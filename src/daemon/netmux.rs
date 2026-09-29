use std::io::Write;
use std::os::unix::net::UnixStream;

pub fn connect_and_listen() -> std::io::Result<UnixStream> {
    let mut stream = UnixStream::connect("/var/run/usbmuxd")?;
    let payload = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
    <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
    <plist version=\"1.0\">\n\
    <dict>\n\
        <key>MessageType</key>\n\
        <string>Listen</string>\n\
        <key>ClientVersionString</key>\n\
        <string>fruitjuiced</string>\n\
        <key>ProgName</key>\n\
        <string>fruitjuiced</string>\n\
    </dict>\n\
    </plist>";

    let len = (16 + payload.len()) as u32;
    let mut packet = Vec::with_capacity(len as usize);
    packet.extend_from_slice(&len.to_le_bytes());
    packet.extend_from_slice(&1u32.to_le_bytes());
    packet.extend_from_slice(&8u32.to_le_bytes());
    packet.extend_from_slice(&1u32.to_le_bytes());
    packet.extend_from_slice(payload.as_bytes());
    stream.write_all(&packet)?;
    Ok(stream)
}
