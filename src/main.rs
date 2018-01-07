mod cpu;
use cpu::CPU;
use std::fs::File;
use std::io::{Read,self};
fn main() {

    // Read little-endian bytes from file into buffer
    let mut f = File::open("/home/dave/proj/synacor/challenge.bin").unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();

    // Convert 8-bit little endian values to 16-bit
    // big endian values to be loaded into CPU memory
    let mut data = vec![0; buf.len()/2];
    for i in 0..data.len() {
        data[i] = buf[2*i] as u16 | (buf[2*i+1] as u16) << 8;
    }

    // Instantiate CPU and load program into memory
    let mut cpu = CPU::new();
    if let Err(msg) = cpu.load_mem(data) {
        println!("Load memory returned error: {:?}" , msg);
        panic!();
    }
    
    let breakpoint_cc = 0;
    let breakpoint_pc = 0;

    cpu.run(breakpoint_cc, breakpoint_pc);
}
