use rtstest::{render_queue_to_framebuffer, OpenGlRenderQueue, PlaceholderTexture};
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const WORLD_WIDTH: usize = 64;
const WORLD_HEIGHT: usize = 48;
const FRAME_TIME: Duration = Duration::from_millis(33);

const PY_VIEWER: &str = r#"
import struct
import sys
import tkinter as tk

if len(sys.argv) < 3:
    print('usage: viewer.py <w> <h>', file=sys.stderr)
    sys.exit(1)

W = int(sys.argv[1])
H = int(sys.argv[2])
FRAME_BYTES = W * H * 4

root = tk.Tk()
root.title('RTS realtime render (закройте окно для выхода)')
img = tk.PhotoImage(width=W, height=H)
canvas = tk.Canvas(root, width=W * 8, height=H * 8, highlightthickness=0)
canvas.pack()
canvas_img = canvas.create_image(0, 0, image=img, anchor='nw')
canvas.scale(canvas_img, 0, 0, 8, 8)

buffer = bytearray()


def update_frame():
    global buffer
    chunk = sys.stdin.buffer.read1(65536)
    if not chunk:
        root.destroy()
        return
    buffer.extend(chunk)

    while len(buffer) >= FRAME_BYTES:
        frame = bytes(buffer[:FRAME_BYTES])
        del buffer[:FRAME_BYTES]

        rows = []
        for y in range(H):
            row_colors = []
            base = y * W * 4
            for x in range(W):
                px = struct.unpack_from('<I', frame, base + x * 4)[0]
                r = (px >> 16) & 0xFF
                g = (px >> 8) & 0xFF
                b = px & 0xFF
                row_colors.append(f'#{r:02x}{g:02x}{b:02x}')
            rows.append('{' + ' '.join(row_colors) + '}')
        img.put(' '.join(rows), to=(0, 0))

    root.after(1, update_frame)

root.after(1, update_frame)
root.mainloop()
"#;

fn main() {
    fs::create_dir_all("target").expect("failed to create target directory");
    let script_path = "target/realtime_viewer.py";
    fs::write(script_path, PY_VIEWER).expect("failed to write realtime viewer script");

    let mut child = Command::new("python3")
        .arg("-u")
        .arg(script_path)
        .arg(WORLD_WIDTH.to_string())
        .arg(WORLD_HEIGHT.to_string())
        .stdin(Stdio::piped())
        .spawn()
        .expect("failed to start realtime viewer window");

    let mut stdin = child.stdin.take().expect("failed to open viewer stdin");

    let mut phase: f32 = 0.0;
    let mut last = Instant::now();

    loop {
        if let Ok(Some(_)) = child.try_wait() {
            break;
        }

        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f32();
        last = now;
        phase += dt;

        let mut render_queue = OpenGlRenderQueue::default();
        render_queue.queue_placeholder_quad(PlaceholderTexture::GoldMine, 8.0, 8.0, 10.0);
        render_queue.queue_placeholder_quad(PlaceholderTexture::Forest, 38.0, 9.0, 12.0);
        render_queue.queue_placeholder_quad(PlaceholderTexture::Barracks, 24.0, 28.0, 14.0);

        let worker_x = 2.0 + (phase * 2.0).sin() * 14.0 + 16.0;
        let worker_y = 2.0 + (phase * 1.4).cos() * 9.0 + 14.0;
        render_queue.queue_placeholder_quad(PlaceholderTexture::Worker, worker_x, worker_y, 3.0);

        let frame = render_queue_to_framebuffer(&render_queue, WORLD_WIDTH, WORLD_HEIGHT);
        let argb = frame.to_argb_u32_buffer();
        let mut bytes = Vec::with_capacity(argb.len() * 4);
        for px in argb {
            bytes.extend_from_slice(&px.to_le_bytes());
        }

        if stdin.write_all(&bytes).is_err() {
            break;
        }

        thread::sleep(FRAME_TIME);
    }
}
