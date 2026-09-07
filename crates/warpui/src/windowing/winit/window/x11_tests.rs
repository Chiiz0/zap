use std::os::unix::net::UnixStream;

use x11rb::rust_connection::DefaultStream;

use super::*;

#[test]
fn user_attention_does_not_panic_after_x11_disconnect() {
    let (client, server) = UnixStream::pair().unwrap();
    let (stream, _) = DefaultStream::from_unix_stream(client).unwrap();
    let conn = RustConnection::for_connected_stream(
        stream,
        xproto::Setup {
            resource_id_mask: 0x000f_ffff,
            maximum_request_length: u16::MAX,
            ..Default::default()
        },
    )
    .unwrap();
    drop(server);

    // 填满发送缓冲，使断连在发送阶段可见，而不只是让 flush 返回错误。
    let send_failed = (0..1024).any(|_| {
        WmHints::default()
            .set(&conn, 1)
            .map(|cookie| cookie.ignore_error())
            .is_err()
    });
    assert!(send_failed);
    let manager = X11Manager {
        conn,
        screen_index: 0,
        atoms: Atoms {
            utf8_string: 0,
            net_active_window: 0,
            net_supporting_wm_check: 0,
            net_wm_name: 0,
        },
    };
    manager.set_user_attention(1, true);
    manager.set_user_attention(1, false);
}
