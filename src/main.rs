use iced::{
    // button, Button, 
    Command, Align, 
    Column,
    Checkbox,
    Space,
    Row, Element, Settings, Text, Application,
    Subscription, ProgressBar, Length
};

use std::time::{Duration, Instant};
//use iced_futures::subscription::{Recipe};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use regex::Regex;
use std::env;
//use std::process;

fn average_matching_line(contents: &String, regex: &Regex) -> f32{
    let mut found = 0;
    let mut total: f32 = 0.0;

    for line in contents.lines() {
        let cap = regex.captures(line);
        if cap.is_none() { continue; }
        let cap = cap.unwrap();

        let as_str = cap.get(1).map_or("", |m| m.as_str());
        if let Ok(speed) = as_str.parse::<f32>() {
            found+= 1;
            total += speed;
        }
    }

    total / found as f32
}

fn average_matching_line_in_file(path: &Path , regex: &Regex) -> f32 {
    let display = path.display();
    let mut file = match File::open(&path){
        Err(why) => panic!("could not open {}: {}", display, why), // TODO find way to remove panic
        Ok(file) => file,
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Err(why) => panic!("Error reading file {}: {}", display, why), // TODO find way to remove panic
        Ok(_)=>(),
    }
    average_matching_line(&contents, &regex)
}

pub fn main() {
    let mut gui_mode = false;

    let args: Vec<String> = env::args().collect();

    for arg in args {
        if arg == "--gui"
        {
            gui_mode = true;
        }
    }

    if gui_mode {
        Counter::run(Settings::default())
    }

    else {
        let mut counter = Counter::new();
        counter.calculate_stats();

        let cpu_speed = format!("Cpu MHz {}. Load {}", counter.cpu_mhz, counter.load_avg);
        let mem_stats = format!("Mem: {} of {}", counter.mem_used / 1024, counter.mem_total / 1024);

        println!("{}", mem_stats);
        println!("{}", cpu_speed);
        println!("gpu_max_graphics_clock: {}", counter.gpu_max_graphics_clock);
        println!("gpu_current_graphics_clock : {} ", counter.gpu_current_graphics_clock);
        println!("gpu_max_memory_clock : {} ", counter.gpu_max_memory_clock);
        println!("gpu_current_memory_clock : {} ", counter.gpu_current_memory_clock);
        println!("gpu_temperature : {} ", counter.gpu_temperature);
        println!("gpu_memory_used : {} ", counter.gpu_memory_used);
        println!("gpu_memory_total : {} ", counter.gpu_memory_total);
        println!("gpu_graphics_utilization : {} ", counter.gpu_graphics_utilization);
        println!("gpu_memory_utilization : {} ", counter.gpu_memory_utilization);

        if counter.errors.len() > 0 {
            println!("Errors: ");
            for error in counter.errors {
                println!("{}", error);
            }
        }
    }
}

struct Counter {
    errors: Vec<String>,
    cpu_stats_visible: bool,
    disk_stats_visible: bool,

    cpu_mhz: i32,
    load_avg: String,
    mem_used: i32,
    mem_total: i32,

    cpu_mhz_regex: Regex,
    mem_total_regex: Regex,
    mem_avaliable_regex: Regex,

    extract_leading_number_regex: Regex,
    extract_load_average_regex: Regex,

    gpu_max_graphics_clock: i32,
    gpu_current_graphics_clock: i32,
    gpu_max_memory_clock: i32,
    gpu_current_memory_clock: i32,
    gpu_temperature: i32,
    gpu_memory_used: i32,
    gpu_memory_total: i32,
    gpu_graphics_utilization: i32,
    gpu_memory_utilization: i32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick(Instant),
    OnCpuVisible(bool),
    OnDiskVisible(bool),
}

impl Counter {
    fn new() -> Counter {
        Counter {
            errors: Vec::new(),
            cpu_stats_visible: true,
            disk_stats_visible: true,

            cpu_mhz: 0,
            load_avg: String::new(),
            mem_used: 0,
            mem_total: 0,

            cpu_mhz_regex: Regex::new(r"cpu MHz\s+:\s+([0-9]+)\.[0-9]").unwrap(),
            mem_total_regex: Regex::new(r"MemTotal:\s+([0-9]+)").unwrap(),
            mem_avaliable_regex: Regex::new(r"MemAvailable:\s+([0-9]+)").unwrap(),
            extract_leading_number_regex: Regex::new(r"\s*([0-9]+)").unwrap(),
            extract_load_average_regex: Regex::new(r"([0-9\.]+ [0-9\.]+ [0-9\.]+)").unwrap(),

            gpu_max_graphics_clock: 0,
            gpu_current_graphics_clock: 0,
            gpu_max_memory_clock: 0,
            gpu_current_memory_clock: 0,
            gpu_temperature: 0,
            gpu_memory_used: 0,
            gpu_memory_total: 0,
            gpu_graphics_utilization: 0,
            gpu_memory_utilization: 0,
        }
    }

    fn get_load_avg(&mut self){
        let path = Path::new("/proc/loadavg");
        let display = path.display();
        let mut file = match File::open(&path){
            Err(why) => panic!("could not open {}: {}", display, why), // TODO find way to remove panic
            Ok(file) => file,
        };

        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Err(why) => panic!("Error reading file {}: {}", display, why), // TODO find way to remove panic
            Ok(_)=>(),
        }

        self.load_avg = {
            let cap = self.extract_load_average_regex.captures(contents.as_str());
            if cap.is_none() {
                contents
            } else {
                let cap = cap.unwrap();
                let as_str = cap.get(1).map_or("", |m| m.as_str());
                as_str.to_string()
            }
        };
    }

    fn get_mem_info(&mut self){
        let path = Path::new("/proc/meminfo");
        let display = path.display();
        let mut mem_file = match File::open(&path){
            // TODO find way to remove panic
            Err(why) => panic!("could not open {}: {}", display, why), 
            Ok(file) => file,
        };

        let mut contents = String::new();
        match mem_file.read_to_string(&mut contents) {
            // TODO find way to remove panic
            Err(why) => panic!("Error reading file {}: {}", display, why), 
            Ok(_)=>(),
        }

        let mem_avaliable = average_matching_line(&contents, &self.mem_avaliable_regex ) as i32;
        self.mem_total = average_matching_line(&contents, &self.mem_total_regex ) as i32;
        self.mem_used = self.mem_total - mem_avaliable;



    }

    fn calculate_stats(&mut self) -> () {
        self.cpu_mhz = average_matching_line_in_file(
            &Path::new("/proc/cpuinfo"), 
            &self.cpu_mhz_regex) as i32;

        self.get_load_avg();
        self.get_mem_info();
        self.get_gpu_stats();
    }

    fn get_gpu_stats(&mut self){
        let cmd = "nvidia-smi";
        let args = &["--format=csv",
        "--query-gpu=clocks.max.graphics,clocks.current.graphics,clocks.max.memory,clocks.current.memory,temperature.gpu,memory.used,memory.total,utilization.gpu,utilization.memory"];
        if let Ok(contents) = String::from_utf8(
            std::process::Command::new(cmd)
            .args(args)
            .output()
            .expect(format!("Failed to execute [{}]", cmd).as_str())
            .stdout) {
            let lines : Vec<&str> = contents.lines().collect();
            if lines.len() != 2 {
                self.errors.push(format!("GPU stats: Invalid numebr of lines. Found [{}]", lines.len()));
            }

            let line = lines[1];
            let elements: Vec<&str> = line.split(",").collect();

            if elements.len() != 9 {
                self.errors.push(format!("Unexpected numebr of stats. Found [{}]", elements.len()));
            }

            let gpu_max_graphics_clock;
            let gpu_current_graphics_clock;
            let gpu_max_memory_clock;
            let gpu_current_memory_clock;
            let gpu_temperature;
            let gpu_memory_used;
            let gpu_memory_total;
            let gpu_graphics_utilization;
            let gpu_memory_utilization;

            {
                let get = |index| -> i32
                {
                    let val: &str = elements[index];
                    let val :String = val.to_string();
                    average_matching_line( &val, &self.extract_leading_number_regex) as i32
                };

                gpu_max_graphics_clock = get(0);
                gpu_current_graphics_clock = get(1);
                gpu_max_memory_clock = get(2);
                gpu_current_memory_clock = get(3);
                gpu_temperature = get(4);
                gpu_memory_used = get(5);
                gpu_memory_total = get(6);
                gpu_graphics_utilization =  get(7);
                gpu_memory_utilization = get(8);
            }

            self.gpu_max_graphics_clock = gpu_max_graphics_clock;
            self.gpu_current_graphics_clock = gpu_current_graphics_clock;
            self.gpu_max_memory_clock = gpu_max_memory_clock;
            self.gpu_current_memory_clock = gpu_current_memory_clock;
            self.gpu_temperature = gpu_temperature;
            self.gpu_memory_used = gpu_memory_used;
            self.gpu_memory_total = gpu_memory_total;
            self.gpu_graphics_utilization = gpu_graphics_utilization;
            self.gpu_memory_utilization = gpu_memory_utilization;
        }
    }
}

impl Application for Counter {
    type Message = Message;
    type Flags = ();
    type Executor = iced_futures::executor::AsyncStd;

    fn new (_flags: () ) -> (Counter, Command<Message>) {
        let mut stats = Counter::new();
        stats.calculate_stats();
        ( stats, Command::none(), )
    }

    fn title(&self)  -> String {
        String::from("system-stats")
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(1000)).map(Message::Tick)
    }

    fn update(&mut self, message:Message) -> Command<Message> {
        match message {
            Message::Tick{..} => { self.calculate_stats(); Command::none() },
            Message::OnCpuVisible(val) => { 
                self.cpu_stats_visible = val;
                Command::none() 
            },
            Message::OnDiskVisible(val) =>{
                self.disk_stats_visible = val;
                Command::none()
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let labeled_bar = | label, low, high | -> Row<Message> {
            Row::new()
                .align_items(Align::Center)
                .spacing(5)
                .push(Text::new(label).size(20).width(Length::Units(160)) )
                .push(ProgressBar::new(0.0..=1.0, low/high))
        };


        let cpu_speed = format!("Cpu MHz {}", self.cpu_mhz);

        let mem = {
            let convert=1024.0 * 1024.0;
            let mem_stats = format!("Mem: {:.1} of {:.1}", self.mem_used as f32 / convert, self.mem_total as f32 / convert);
            labeled_bar(mem_stats, self.mem_used as f32 , self.mem_total as f32)
        };


        let gpu = {
            let gpu_mem = format!("GPU Mem: {} of {}", self.gpu_memory_used, self.gpu_memory_total);
            let mem_bar = labeled_bar(gpu_mem, self.gpu_memory_used as f32 , self.gpu_memory_total as f32);

            let gpu_graphics_clock = format!("Gfx Clock {} of {}", 
                                             self.gpu_current_graphics_clock as f32,
                                             self.gpu_max_graphics_clock as f32);

            let gpu_graphics_clock_bar = labeled_bar(gpu_graphics_clock, 
                                                    self.gpu_current_graphics_clock as f32,
                                                    self.gpu_max_graphics_clock as f32);

//            let gpu_memory_clock = format!("Memory Clock {} of {}", 
//                                           self.gpu_current_memory_clock,
//                                           self.gpu_max_memory_clock);
//            let gpu_memory_clock_bar = labeled_bar(gpu_memory_clock,
//                                                  self.gpu_current_memory_clock as f32,
//                                                  self.gpu_max_memory_clock as f32);

            let gpu_temp_bar = labeled_bar(format!("GPU temp {}", self.gpu_temperature),
                                          self.gpu_temperature as f32 / 100.0, 1.0);

            let gfx_util = labeled_bar(format!("Gfx utilization {}", self.gpu_graphics_utilization),
                                      self.gpu_graphics_utilization as f32 / 100.0, 1.0);

            let mem_util = labeled_bar(format!("Mem utilization {}", self.gpu_memory_utilization),
                                      self.gpu_memory_utilization as f32 / 100.0, 1.0);

            Column::new()
                .spacing(5)
                .align_items(Align::Center)
                .push(  mem_bar )
                .push(gpu_graphics_clock_bar)
                //.push(gpu_memory_clock_bar)
                .push(gpu_temp_bar)
                .push(gfx_util)
                .push(mem_util)
        };

        let cpu_stats = 
            if self.cpu_stats_visible {
                Column::new()
                .padding(20)
                .align_items(Align::Center)
                .push( Row::new()
                    .push(Text::new(cpu_speed).size(20))
                    .push(Space::with_width(Length::Fill))
                    .push(Text::new(format!("Load: {}", self.load_avg)).size(20)))
                .push(mem)
                .push(gpu)
            }
        else {
            Column::new()
        };

        let disk_stats = 
            if self.disk_stats_visible {
                Column::new()
            } else {
                Column::new()
            };

        let visibility_checks = Row::new()
            .push( Checkbox::new(self.cpu_stats_visible, "Cpu stats", Message::OnCpuVisible))
            .push( Checkbox::new(self.disk_stats_visible, "Disk stats", Message::OnDiskVisible))
                   ;

        let main = Column::new()
            .push(visibility_checks)
            .push(cpu_stats)
            .push(disk_stats);

        main.into()
    }
}


mod time {
    use iced_futures::subscription::{Recipe};
    use iced::futures;
    use async_std;

    pub fn every( duration: std::time::Duration,)->
        iced::Subscription<std::time::Instant> {
            iced::Subscription::from_recipe(Every(duration))
        }

    struct Every(std::time::Duration);

    impl<H, I> Recipe<H,I> for Every where
        H: std::hash::Hasher,
    {
        type Output = std::time::Instant;

        fn hash(&self, state: &mut H){
            use std::hash::Hash;

            std::any::TypeId::of::<Self>().hash(state);
            self.0.hash(state);
        }

        fn stream(
            self: Box<Self>,
            _input: futures::stream::BoxStream<'static, I>,
            ) -> futures::stream::BoxStream<'static, Self::Output>{
                use futures::stream::StreamExt;

                async_std::stream::interval(self.0)
                    .map(|_| std::time::Instant::now())
                    .boxed()
        }
    }
}
