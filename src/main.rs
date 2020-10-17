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

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
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

const MAX_THREADS:usize = 7;

#[derive(Default, Debug)]
struct ThreadCommands {
    control_tx: [Option<Sender<ControlCommand>>;MAX_THREADS],
    draw_rx: [Option<Receiver<DrawCommand>>;MAX_THREADS],
    buf: [Option<piston_play::Buffer>;MAX_THREADS],
}

impl ThreadCommands{
    fn draw_rx_ref<'t>(&'t self, id: usize) -> &'t Receiver<DrawCommand>{
        match &self.draw_rx[id]{
            Some(x) => return &x,
            None => panic!("Uninitialized draw_rx"),
        }
    }
    fn command_tx_ref<'t>(&'t self, id: usize) -> &'t Sender<ControlCommand>{
        match &self.control_tx[id]{
            Some(x) => return &x,
            None => panic!("Uninitialized draw_rx {}", id),
        }
    }

}

fn main() {
    let mut x = 800;
    let mut y  = 600;
    let cpus = std::cmp::min(num_cpus::get(), MAX_THREADS);
    let color_bases = [
        [255, 0, 0],
        [0, 255, 0],
        [255, 255, 0],
        [0, 0,255],
        [255, 0,255],
        [0, 255,255],
        [255, 255, 255]
    ];
    let mut control:ThreadCommands = Default::default();

    for cpu in 0..cpus{
        let (control_tx, control_rx): (Sender<ControlCommand>, Receiver<ControlCommand>) = mpsc::channel();
        let (draw_tx, draw_rx): (SyncSender<DrawCommand>, Receiver<DrawCommand>) = mpsc::sync_channel(1024);
        control.control_tx[cpu] = Some(control_tx);
        control.draw_rx[cpu] = Some(draw_rx);
        thread::spawn(move ||{
                println!("Spawning thread for cpu {}", cpu);
                calc(draw_tx, control_rx, x, y, color_bases[cpu])
        });
    }
    println!("{:#?}", control);
    let mut window =
        pw::WindowSettings::new("test", (x, y))
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut buffer = Buffer::new(x, y);

    let mut events = pw::Events::new(
        (||{
            let mut settings = pw::EventSettings::new();
            settings.ups = 1;
            settings.max_fps = 2;
            settings
        })()
    );
    let mut cnt = 0;
    let mut idle_time: f64 = 0.0;
    let mut render_time: f64 = 0.0;


    while let Some(e) = events.next(&mut window) {
        match e{
            piston::Event::Loop(piston::Loop::Idle(ref idle)) => {
                    for cpu in 0..cpus {
                        cnt += process_draw_commands(
                            Duration::from_secs_f64(idle.dt),
                            control.draw_rx_ref(cpu),
                            buffer.buf_mut_ref()
                        );
                        idle_time += idle.dt;
                    }
                    
            }
            piston::Event::Loop(piston::Loop::AfterRender(_)) => {
            }
            piston::Event::Loop(piston::Loop::Render(_)) => {
                let start_time = Instant::now();
                let texture = buffer.as_texture(& mut window);
                window.draw_2d(
                    &e,
                    |context, graph_2d, _device| { //graph_2d -> https://docs.piston.rs/piston_window/gfx_graphics/struct.GfxGraphics.html
                        
                        println!("transform: {:?}", context.transform);
                        // [
                        //      [0.0025, 0.0, -1.0], 
                        //      [0.0, -0.0033333333333333335, 1.0]
                        //]
                        let [[xscale, _, _], [_, y_scale, _]] = context.transform;
                        pw::image(
                            &texture,
                            // context.reset().transform,
                            // [[0.00125, 0.0, -1.0], [0.0, -0.0016, 1.0]],  //left-top corner
                            // [[0.00125, 0.0, -1.0], [0.0, -0.0016666, 0.0]], //left-bottom corner
                            // [[0.00125, 0.0, 0.0], [0.0, -0.0016, 1.0]], //right-top corner
                            // [[0.00125, 0.0, 0.0], [0.0, -0.00166666, 0.0]], //right-bottom corner
                            [ //right-bottom corner
                                [xscale/2.0, 0.0, 0.0],
                                [0.0, y_scale/2.0, 0.0]
                            ],
                            graph_2d
                        );
                        pw::image(
                            &texture,
                            // [[0.00125, 0.0, -1.0], [0.0, -0.0016, 1.0]],  //left-top corner
                            // [[0.00125, 0.0, -1.0], [0.0, -0.0016666, 0.0]], //left-bottom corner
                            // [[0.00125, 0.0, 0.0], [0.0, -0.00166666, 1.0]], //right-top corner
                            [//right-top corner
                                [xscale/2.0, 0.0, 0.0],
                                [0.0, y_scale/2.0, 1.0]
                            ],
                            // [[0.00125, 0.0, 0.0], [0.0, -0.00166666, 0.0]], //right-bottom corner
                            graph_2d
                        );
                        pw::image(
                            &texture,
                            // [[0.00125, 0.0, -1.0], [0.0, -0.0016, 1.0]],  //left-top corner
                            // [[0.00125, 0.0, -1.0], [0.0, -0.0016666, 0.0]], //left-bottom corner
                            [//left-bottom corner
                                [xscale/2.0, 0.0, -1.0],
                                [0.0, y_scale/2.0, 0.0]
                            ],
                            // [[0.00125, 0.0, 0.0], [0.0, -0.0016, 1.0]], //right-top corner
                            // [[0.00125, 0.0, 0.0], [0.0, -0.00166666, 0.0]], //right-bottom corner
                            graph_2d
                        );
                        pw::image(
                            &texture,
                            // [[0.00125, 0.0, -1.0], [0.0, -0.001666666, 1.0]],  //left-top corner
                            [//left-top corner
                                [xscale/2.0, 0.0, -1.0],
                                [0.0, y_scale/2.0, 1.0]
                            ],
                            // [[0.00125, 0.0, -1.0], [0.0, -0.0016666, 0.0]], //left-bottom corner
                            // [[0.00125, 0.0, 0.0], [0.0, -0.0016, 1.0]], //right-top corner
                            // [[0.00125, 0.0, 0.0], [0.0, -0.00166666, 0.0]], //right-bottom corner
                            graph_2d
                        );

                    }
                );
                let draw_time = Instant::now();
                // println!("Render: {:?}, {:?} -> {:?}", texture_time - start, draw_time -texture_time, Instant::now());
                render_time += (draw_time-start_time).as_secs_f64();
                drop(texture);
            }
            piston::Event::Loop(piston::Loop::Update(_)) => {
                println!("total idle time: {:.2}, pixels: {}, kpps: {:.1}", idle_time, cnt, cnt as f64/idle_time/1000.0);
                cnt = 0;
                idle_time = 0.0;
            }
            piston::Event::Input(piston::Input::Resize(piston::ResizeArgs{window_size:_, draw_size:[new_x, new_y]}), _) => {
                println!("Resize event: {}x{} (was {}x{})", new_x, new_y, x, y);
                for cpu in 0..cpus{
                    let (new_draw_tx, new_draw_rx): (SyncSender<DrawCommand>, Receiver<DrawCommand>) = mpsc::sync_channel(1024);
                    control.command_tx_ref(cpu).send(ControlCommand{command: Command::NewResolution(
                            new_x, new_y, new_draw_tx
                        )}).unwrap();
                    control.draw_rx[cpu] = Some(new_draw_rx);
                    println!("Redo, cpu {}. {:#?}", cpu, control);
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

fn gen_color(rng: & mut rand::rngs::ThreadRng, range: u8) -> u8{
    if range > 0 {
        rng.gen_range(0, range)
    }
    else{
        0
    }
}

fn calc(draw: SyncSender<DrawCommand>, command: Receiver<ControlCommand>, max_x:u32, max_y:u32, color_base:[u8;3]){
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
            gen_color(& mut rng, color_base[0]),
            gen_color(& mut rng, color_base[1]),
            gen_color(& mut rng, color_base[2]),
            gen_color(& mut rng, 255)
        ]);
        if let Err(_) = draw_cmd.send(DrawCommand{x, y, color}){ continue ;}
    }
}
