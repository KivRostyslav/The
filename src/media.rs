use std::thread::sleep;
use std::time::{Duration, SystemTime};
use std::ptr;

use opencl3::command_queue::{CommandQueue, CL_QUEUE_PROFILING_ENABLE};
use opencl3::context::Context;
use opencl3::device::{get_all_devices, Device, CL_DEVICE_GLOBAL_MEM_SIZE, CL_DEVICE_MAX_WORK_ITEM_SIZES, CL_DEVICE_TYPE_GPU};
use opencl3::kernel::{ExecuteKernel, Kernel};
use opencl3::memory::{Buffer, CL_MEM_READ_ONLY, CL_MEM_WRITE_ONLY};
use opencl3::program::Program;
use opencl3::types::{cl_bool, cl_char, cl_event, cl_float, cl_uchar, cl_uint, CL_BLOCKING, CL_NON_BLOCKING};


use crossbeam::channel::{Sender, Receiver};
use opencv::core::{MatTraitConst, MatTraitConstManual, Size};
use opencv::videoio::{VideoCapture, VideoCaptureTraitConst, CAP_ANY, CAP_PROP_POS_FRAMES};
use opencv::{imgcodecs, imgproc};
use opencv::prelude::Mat;
use opencv::prelude::VideoCaptureTrait;
use crate::event_loop::{self, LoopEvent};
use crate::{ascii::AsciiConverter, terminal::StringInfo};
use crate::controller::Controller;

pub enum MediaType {
    Image(Mat),
    Video(VideoCapture),
}

pub struct MediaController<'a> {
    event_loop_receiver: &'a Receiver<LoopEvent>,
    media_sender: &'a Sender<StringInfo>,
    ascii_converter: AsciiConverter,
    media_type: MediaType,

    kernel: Kernel,
    queue: CommandQueue,
    context: Context,
}

impl<'a> MediaController<'a> {
    pub fn new(uri: &String, media_sender: &'a Sender<StringInfo>, event_loop_receiver: &'a Receiver<LoopEvent>) -> Result<Self, String> {
        let mut media_type: Option<MediaType> = None;
        let result = imgcodecs::have_image_reader(uri);
        if result.is_ok() && result.unwrap() {
            //media_type = Some(MediaType::Image(imgcodecs::imread(uri, imgcodecs::IMREAD_GRAYSCALE).unwrap())); 
                        media_type = Some(MediaType::Video(VideoCapture::from_file(uri, CAP_ANY).unwrap()));           

        }
        else {
            media_type = Some(MediaType::Video(VideoCapture::from_file(uri, CAP_ANY).unwrap()));           
        }

        if media_type.is_none() {
            return Err("No supported file extensionfsdfsad".to_string());
        }
        
        let device_id = *get_all_devices(CL_DEVICE_TYPE_GPU).unwrap().first().expect("no device found in platform");
        let device = Device::new(device_id);
        let context = Context::from_device(&device).expect("context::from_device failed");
        let queue = CommandQueue::create_default_with_properties(&context, CL_QUEUE_PROFILING_ENABLE, 200).expect("commandqueue::create_default failed");
        let program = Program::create_and_build_from_source(&context, crate::ascii::PROGMRAM, "").expect("program::create_and_build_from_source failed");
        let kernel = Kernel::create(&program, "calculate").unwrap();

        Ok(Self {
            ascii_converter: AsciiConverter::new(&crate::ascii::CHARS3.to_string()),
            event_loop_receiver,
            media_sender,
            media_type: media_type.unwrap(),
            kernel,
            queue,
            context,
        })
    }
}

impl<'a> Controller for MediaController<'a> {
    fn run(&mut self) {
        match &mut self.media_type {
            MediaType::Image(x) => {
                let terminal_size = termion::terminal_size().unwrap();
                let size = Size::new(terminal_size.0 as i32, terminal_size.1 as i32);
                let mut mat = Mat::default();
                let img = imgproc::resize(x, &mut mat, size, 0.0, 0.0, imgproc::INTER_LINEAR);
                self.media_sender.send(self.ascii_converter.convert(&mat, true)).unwrap(); 
            },
            MediaType::Video(video) => {
                let fps = video.get(opencv::videoio::CAP_PROP_FPS).unwrap_or(30.0);
                let ms_per_frame = (1000.0f64 / fps).floor() as u64;
                let mut frame_index = 0i64;

                let chars = crate::ascii::NO.chars().collect::<Vec<char>>();
                let char_len = chars.iter().map(|x| x.len_utf8()).max().unwrap() as u32;
                let mut char_bytes = Vec::<u8>::new();
                for char in &chars {
                    let mut bytes = vec![0; char_len as usize];
                    char.encode_utf8(&mut bytes);
                    for byte in bytes {
                        char_bytes.push(byte);
                    }
                }

                println!("{}", char_len);
                println!("{:?}", char_bytes);
                println!("{}", String::from_utf8(char_bytes.clone()).unwrap());
                
                let mut size = 0;



                let mut chars_buffer = unsafe { Buffer::<cl_uchar>::create(&self.context, CL_MEM_READ_ONLY, chars.len() * char_len as usize, ptr::null_mut()).unwrap() };
                let mut output_buffer = unsafe { Buffer::<cl_uchar>::create(&self.context, CL_MEM_WRITE_ONLY, 1, ptr::null_mut()).unwrap() };
                let mut frame_buffer = unsafe { Buffer::<cl_uchar>::create(&self.context, CL_MEM_READ_ONLY, 1, ptr::null_mut()).unwrap() };

                let chars_buffer_write_event = unsafe { self.queue.enqueue_write_buffer(&mut chars_buffer, CL_NON_BLOCKING, 0, &char_bytes, &[]) };

                let grayscale: cl_uint = 0;
                let step: cl_uint = (255.0 / (chars.len() as f32)).ceil() as u32;
                let mut is_playing = true;
                let mut shutdown = false;
                let now = SystemTime::now();
                loop {
                    if shutdown {
                        break;
                    }

                    if !self.event_loop_receiver.is_empty() || !is_playing {
                        let event = self.event_loop_receiver.recv().unwrap();
                        print!("{:?}", event);
                        match event {
                            LoopEvent::Shutdown => { shutdown = true; },
                            LoopEvent::PlayPause => { is_playing = !is_playing; },
                            LoopEvent::Skip(x) => {
                                let frame_to_skip = x as i64 * fps as i64;
                                if x < 0 && (frame_to_skip + frame_index) < 0 {
                                    frame_index = 0; 
                                }
                                else {
                                    frame_index += frame_to_skip;
                                }
                                video.set(CAP_PROP_POS_FRAMES, frame_index as f64).unwrap();
                            },
                        }

                        continue;
                    }
                    
                    let start_time = SystemTime::now();

                    let mut frame = Mat::default();
                    let result = video.read(&mut frame);
                    if result.is_err() || !result.unwrap() || frame.empty() {
                        break;
                    }

                    let terminal_size = termion::terminal_size().unwrap();
                    let new_size = Size::new(terminal_size.0 as i32, terminal_size.1 as i32);
                    
                    let mut resized_frame = Mat::default();
                    let result = imgproc::resize(&frame, &mut resized_frame, new_size, 0.0, 0.0, imgproc::INTER_LINEAR);
                    if result.is_err() || resized_frame.empty() {
                        break;
                    }

                    if grayscale == 1 {
                        let mut gray_frame = Mat::default();
                        let result = imgproc::cvt_color(&resized_frame, &mut gray_frame, imgproc::COLOR_BGR2GRAY, 0);
                        if result.is_err() || gray_frame.empty() {
                            break;
                        }

                        resized_frame = gray_frame; 
                    }

                    let frame_bytes = resized_frame.data_bytes().unwrap();
                    if size != frame_bytes.len() {
                        size = frame_bytes.len();
                        frame_buffer = unsafe { Buffer::<cl_uchar>::create(&self.context, CL_MEM_READ_ONLY, size, ptr::null_mut()).unwrap() };
                        output_buffer = unsafe { Buffer::<cl_uchar>::create(&self.context, CL_MEM_WRITE_ONLY, size * char_len as usize / 3, ptr::null_mut()).unwrap() };
                    }
                    
                    let _ = unsafe { self.queue.enqueue_write_buffer(&mut frame_buffer, CL_BLOCKING, 0, resized_frame.data_bytes().unwrap(), &[]) };

                    let execute = unsafe {
                        ExecuteKernel::new(&self.kernel)
                            .set_arg(&frame_buffer)
                            .set_arg(&chars_buffer)
                            .set_arg(&char_len)
                            .set_arg(&grayscale)
                            .set_arg(&step)
                            .set_arg(&output_buffer)
                            .set_event_wait_list(&[chars_buffer_write_event.as_ref().unwrap().get()])
                            .set_global_work_size(size / 3)
                            .enqueue_nd_range(&self.queue).unwrap()
                    };

                    self.queue.flush().unwrap();
                    self.queue.finish().unwrap();

                    let mut string: Vec<cl_uchar> = vec![0; size * char_len as usize / 3];
                    let _ = unsafe { self.queue.enqueue_read_buffer(&output_buffer, CL_BLOCKING, 0, &mut string, &[execute.get()]).unwrap() };

                    let mut rgb = Vec::new();
                    if grayscale == 0 {
                        rgb = frame_bytes.to_vec();
                    }
                    
                    self.media_sender.send(StringInfo {string, rgb, char_len}).unwrap(); 
                    self.queue.flush().unwrap();
                    self.queue.finish().unwrap();

                    let time = start_time.elapsed();
                    if time.is_err() {
                        break;
                    }
                    
                    let time = time.unwrap();
                    frame_index += 1;
                    let deadtime_to_frame_preparing = Duration::from_millis(ms_per_frame); 
                    if time < deadtime_to_frame_preparing {
                        sleep(deadtime_to_frame_preparing - time);
                        continue;
                    }

                    let frames_to_skip = (time - deadtime_to_frame_preparing).div_duration_f64(Duration::from_millis(ms_per_frame)).ceil() as u64;
                    frame_index += frames_to_skip as i64;

                    {
                        let mut skipped = Mat::default();
                        for _ in 0..frames_to_skip {
                            if !video.read(&mut skipped).unwrap_or(false) || skipped.empty() {
                                break;
                            }
                        }
                    }
                }

                println!("{}", termion::clear::All);
                println!("{:?}", now.elapsed().unwrap().as_secs());
            },

        }
    }
}
