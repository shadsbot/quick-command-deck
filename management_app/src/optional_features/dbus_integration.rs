#[cfg(feature = "dbus")]
pub mod listener {
    use dbus::blocking::Connection;
    use dbus_crossroads::Crossroads;

    use crate::protos::communique::DisplayText;
    use std::sync::mpsc::Sender;

    pub fn connect(dptx: Sender<DisplayText>) -> Result<(), dbus::Error> {
        trace!("Attempting connection to dbus");
        let c = Connection::new_session();
        let c = c.unwrap();
        info!("Connected to dbus!");
        c.request_name("com.quickcommanddeck.endpoint", false, true, false)?;
        let mut cr = Crossroads::new();
        let token = cr.register("com.quickcommanddeck.endpoint", |b| {
            b.method(
                "SendText",
                ("lines",),
                ("reply",),
                move |_, _, (line,): (String,)| {
                    let reply = format!("Got: {:?}", line);
                    let chunked = line.as_bytes().chunks(16);
                    let chunked = chunked
                        .map(|x| std::str::from_utf8(x).unwrap_or_default().to_string())
                        .collect();
                    let mut msg = DisplayText::new();
                    msg.set_line(protobuf::RepeatedField::from_vec(chunked));
                    msg.set_brightness(255); // this needs to not be hardcoded
                    msg.set_duration_ms(500); // this also needs to not be hardcoded
                    dptx.send(msg);
                    Ok((reply,))
                },
            );
        });
        cr.insert("/sendText", &[token], ());
        cr.serve(&c) // serves forever
                     // anything past this point is unreachable
                     // so really this should be called in a thread
    }
}

/*
Listens for any messages coming through on dbus endpoint:
    com.quickcommanddeck.endpoint/sendText

Can be invoked with any message via a similar command:
    gdbus call --session --dest com.quickcommanddeck.endpoint \
    --object-path /sendText --method com.quickcommanddeck.endpoint.SendText\
    "your message here"

Currently hardcoded to split lines at 16 characters with preset brightness/duration.
TODO: make it not be that way
*/
