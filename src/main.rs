use image as im;
use piston_window as pw;
use piston;
use rand::Rng;
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{SyncSender, Sender, Receiver};
use std::time::{Instant, Duration};
use piston_play:: {Buffer};

#[derive(Debug)]
struct DrawCommand {
    x: u32,
    y: u32,
    color: im::Rgba<u8>
}
#[derive(Debug)]
enum Command {
    NewResolution(u32, u32, SyncSender<DrawCommand>)
}

 #[derive(Debug)]
struct ControlCommand{
    command: Command
}


fn process_draw_commands (allocated_time: Duration, rx: &Receiver<DrawCommand>, buf: &mut im::RgbaImage) -> u64{
    let mut cnt = 0;
    let start = Instant::now();
    while Instant::now() - start < allocated_time {
        for _count in 0..1024 {
            cnt +=1;
            if let Ok(cmd) = rx.try_recv(){
                buf.put_pixel(cmd.x,cmd.y,cmd.color);
            }else{
                break;
            }
        }
    }
    cnt
}


fn main() {
    let mut x = 800;
    let mut y  = 600;
    let cpus = num_cpus::get();

    let (draw_tx, mut draw_rx): (SyncSender<DrawCommand>, Receiver<DrawCommand>) = mpsc::sync_channel(1024);
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

    let mut window: pw::PistonWindow =
        pw::WindowSettings::new("test", (x, y))
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut buffer = Buffer::new(x, y);

    let mut events = pw::Events::new(
        (||{
            let mut settings = pw::EventSettings::new();
            settings.ups = 2;
            settings.max_fps = 6;
            settings
        })()
    );


    let mut texture_context = window.create_texture_context();
    while let Some(e) = events.next(&mut window) {
        match e{
            piston::Event::Loop(piston::Loop::Idle(ref idle)) => {
                    let cnt = process_draw_commands(
                        Duration::from_secs_f64(idle.dt),
                        &draw_rx,
                        buffer.buf_mut_ref()
                    );
                    println!("Idle: {}, cnt:{}", idle.dt, cnt);
            }
            piston::Event::Loop(piston::Loop::AfterRender(_)) => {
            }
            piston::Event::Loop(piston::Loop::Render(_)) => {
                let start = Instant::now();
                let texture: pw::G2dTexture = pw::Texture::from_image(
                            &mut texture_context,
                            buffer.buf_ref(),
                            &pw::TextureSettings::new()
                        ).unwrap();
                let texture_time = Instant::now();
                window.draw_2d(
                    &e,
                    |context, graph_2d, _device| { //graph_2d -> https://docs.piston.rs/piston_window/gfx_graphics/struct.GfxGraphics.html
                        pw::image(
                            &texture,
                            context.transform,
                            graph_2d
                        );
                    }
                );
                let draw_time = Instant::now();
                println!("Render: {:?}, {:?} -> {:?}", texture_time - start, draw_time -texture_time, Instant::now());
                drop(texture);
            }
            piston::Event::Loop(piston::Loop::Update(_)) => {
            }
            piston::Event::Input(piston::Input::Resize(piston::ResizeArgs{window_size:_, draw_size:[new_x, new_y]}), _) => {
                println!("Resize event: {}x{} (was {}x{})", new_x, new_y, x, y);
                let (new_draw_tx, new_draw_rx): (SyncSender<DrawCommand>, Receiver<DrawCommand>) = mpsc::sync_channel(128);
                draw_rx = new_draw_rx;
                for control_tx in &control_txes{
                    control_tx.send(ControlCommand{command: Command::NewResolution(
                        new_x, new_y, new_draw_tx.clone()
                    )}).unwrap();
                }
                buffer.scale(new_x, new_y);
                x = new_x;
                y = new_y;
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
        let color = im::Rgba([
            rng.gen_range(0,255),
            rng.gen_range(0,255),
            rng.gen_range(0,255),
            rng.gen_range(0,255)]
        );
        if let Err(_) = draw_cmd.send(DrawCommand{x, y, color}){ continue ;}
    }
}
