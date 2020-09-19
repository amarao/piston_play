extern crate piston_window;
extern crate image;
use image::{RgbaImage, Rgba};
use piston_window::*;
use piston;
use rand::Rng;
use std::thread;
// use std::time::Duration;
use std::sync::mpsc;
use std::sync::mpsc::{SyncSender, Sender, Receiver};

struct DrawCommand {
    x: u32,
    y: u32,
    color: image::Rgba<u8>
}
#[derive(Debug)]
enum Command {
    NewResolution(u32, u32, SyncSender<DrawCommand>)
}

 #[derive(Debug)]
struct ControlCommand{
    command: Command
}
fn scale(buf: RgbaImage, old_x:u32, old_y:u32, new_x:u32, new_y:u32) -> RgbaImage{
    image::ImageBuffer::from_fn(new_x, new_y, |x, y| {
        if x < old_x && y < old_y {
            *(buf.get_pixel(x, y))
        }else{
            Rgba([255,255,255,255])
        }
    })
}

fn main() {
    let mut x = 800;
    let mut y  = 600;
    let cpus = num_cpus::get();

    let (mut draw_tx, mut draw_rx): (SyncSender<DrawCommand>, Receiver<DrawCommand>) = mpsc::sync_channel(128);
    let mut control_txes: Vec<Sender<ControlCommand>> = Vec::new();
    for cpu in 1..cpus{
        let (control_tx, control_rx): (Sender<ControlCommand>, Receiver<ControlCommand>) = mpsc::channel();
        control_txes.push(control_tx);
        let thread_draw_tx = draw_tx.clone();
        thread::spawn(move ||{
                println!("Spawning thread for cpu {}", cpu);
                calc(thread_draw_tx, control_rx, x, y)
        });
    }

    let mut window: PistonWindow =
        WindowSettings::new("test", (x, y))
        .exit_on_esc(true)
        .build()
        .unwrap();
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into()
    };
    let mut buf = image::ImageBuffer::from_fn(x, y, |_, __| { image::Rgba([255,255,255,255])});
    let mut texture: G2dTexture = Texture::from_image(
                &mut texture_context,
                &buf,
                &TextureSettings::new()
            ).unwrap();
    let mut events = Events::new(EventSettings::new().lazy(false));
    events.set_ups(5);
    let mut draw_per_sec = 10000;
    let mut cnt = 0;

    while let Some(e) = events.next(&mut window) {
        match e{
            piston::Event::Loop(piston::Loop::Idle(ref idle)) => {
                let start = std::time::Instant::now();
                let mut draws = (idle.dt*draw_per_sec as f64) as i32;
                if draws < 100 {
                    draws = 100;
                }
                cnt = 0;
                'full: for _bucket in 0..10 {
                    // println!("bucket: {}, cnt: {}", bucket, cnt);
                    for _count in 0..draws/10 {
                        if let Ok(cmd) = draw_rx.try_recv(){
                            buf.put_pixel(cmd.x,cmd.y,cmd.color);
                            cnt += 1;
                        }else{
                            break 'full;
                        }
                    }
                    let spent = (std::time::Instant::now() - start).as_secs_f64();
                    if  spent > idle.dt * 2.0 && draw_per_sec > 10000 {
                        draw_per_sec -= draw_per_sec / 10;
                    }
                    if spent < idle.dt / 2.0 {
                        draw_per_sec += draw_per_sec / 10;
                    }
                }
            }
            piston::Event::Loop(piston::Loop::AfterRender(_)) => {}
            piston::Event::Loop(piston::Loop::Render(_)) => {
                texture.update(&mut texture_context, &buf).unwrap();
                window.draw_2d(&e, |c, g, device| {
                        texture_context.encoder.flush(device);
                        image(&texture, c.transform, g);
                });
            }
            piston::Event::Loop(piston::Loop::Update(_)) => {
                println!{"last cycle draw {} pixels, calculated speed is {} pps.", cnt, draw_per_sec};

            }
            piston::Event::Input(piston::Input::Resize(piston::ResizeArgs{window_size:_, draw_size:[new_x, new_y]}), _) => {
                println!("Resize event: {}x{} (was {}x{})", new_x, new_y, x, y);
                // drop(draw_rx);
                let (new_draw_tx, new_draw_rx): (SyncSender<DrawCommand>, Receiver<DrawCommand>) = mpsc::sync_channel(128);
                draw_rx = new_draw_rx;
                for control_tx in &control_txes{
                    println!("{:?}", control_tx.send(ControlCommand{command: Command::NewResolution(
                        new_x, new_y, new_draw_tx.clone()
                    )}));
                }
                buf = scale(buf, x, y, new_x, new_y);
                x = new_x;
                y = new_y;
                texture = Texture::from_image(
                    &mut texture_context,
                    &buf,
                    &TextureSettings::new()
                ).unwrap();
            },
            piston::Event::Input(_, _) => {
            },
            ref something => {
                println!("Unexpected something: {:?}", something);
            },
        }
        window.event(&e);
    }
}

fn calc(draw: SyncSender<DrawCommand>, command: Receiver<ControlCommand>, max_x:u32, max_y:u32){
    let mut cur_x = max_x;
    let mut cur_y = max_y;
    let mut draw_cmd = draw;
    let mut rng = rand::thread_rng();
    loop{
        match command.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                return;
            },
            Ok(ControlCommand{command: Command::NewResolution(new_x, new_y, new_draw)}) => {
                    println!("new resolution:{}x{}", new_x, new_y);
                    cur_x = new_x;
                    cur_y = new_y;
                    draw_cmd = new_draw;
            },
            _ => {}
        }
        let x = rng.gen_range(0,cur_x);
        let y = rng.gen_range(0,cur_y);
        let color = image::Rgba([
            rng.gen_range(0,255),
            rng.gen_range(0,255),
            rng.gen_range(0,255),
            rng.gen_range(0,255)]
        );
        if let Err(e) = draw_cmd.send(DrawCommand{x, y, color}){
            println!("err: {}", e);
        }
    }
}
