use bRAC::proto::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut conn = connect("wracs://meex.lol", None)?;

    send_message(&mut conn, "<dude> hi RAC-loving kikes!")?;
    register_user(&mut conn, "dude", "password")?;
    send_message_auth(&mut conn, "dude", "password", "my auth message")?;

    let (mut all_messages, last_size) = read_messages(&mut conn, 10, 0, false)?.unwrap(); // limits with 10 messages

    /* imagine that new messages were written here */

    let (mut new_messages, last_size) = read_messages(&mut conn, 10, last_size, true)?.unwrap(); // chunked reading!

    all_messages.append(&mut new_messages);

    println!("all_messages: {all_messages:?}. last_size: {last_size}");

    Ok(())
}
