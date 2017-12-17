extern crate byteorder;

//use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian, LittleEndian, ByteOrder};
use std::fs::File;
//use std::io::prelude::*;
use std::io::Read;

const MOD: u16 = 32768;
const MAX_ADDR: u16 = 32775;
const MAX_VALID_VAL: u16 = 32775;
const MAX_MEM_ADDR: u16 = 32767;
const MEM_CAPACITY: usize = 32768;
const MAX_15_BIT_VAL: u16 = 32767;
const MAX_REG_ID: u16 = 7;


struct CPU {
    // 8 registers holding 16-bit values. This
    // vector is 8 elements long and refers to
    // r0..r7 respectively. Mem addresses 32768
    // to 32775 refer to r0..r7.
    reg: Vec<u16>,

    // 15-bit address space storing 16-bit values,
    // so this vector is 32768 elements long
    mem: Vec<u16>,

    // Unbounded stack holding 16-bit values
    stack: Vec<u16>,

    // Program counter
    pc: u16,

    halt: bool,

    terminal: String,
}

impl CPU {
    fn new() -> CPU {
        CPU {
            reg: vec![0; 8],
            mem: vec![0; MEM_CAPACITY],
            stack: vec![0],
            pc: 0,
            halt: false,
            terminal: String::new(),
        }
    }

    fn load_mem (&mut self, mem_input: Vec<u16>) -> Result<(), &'static str> {
        if mem_input.len() == 0 {
            return Err("No memory loaded.");
        }
        else if mem_input.len() > MEM_CAPACITY {
            return Err("Attempted to load memory larger than capacity.");
        }

        for ind in 0..mem_input.len() {
            self.mem[ind] = mem_input[ind];
        }
        Ok(())
    }

    fn inc_pc (&mut self) {
        self.pc += 1;
    }

    fn set_pc (&mut self, addr: u16) {
        self.pc = addr;
    }

    fn mem_read (&mut self, mem_addr: u16) -> Result<u16, &'static str> {
        // this function reads from memory if mem_addr <= 32767,
        // or from registers r0..r7 if 32768 <= mem_addr <= 32775.
        // Requires cast from u16 to usize for indexing into memory

        if mem_addr > MAX_ADDR {
            return Err("Attempted to read value outside valid memory range");
        }

        if mem_addr > MAX_MEM_ADDR {
            // Read from registers, so take modulus of mem address
            let reg_id = mem_addr % MOD;
            if reg_id > MAX_REG_ID {
                return Err("Attempted to read from register ID > 7 1");
            }
            println!("1. Reading from reg {:?}, value {:?}", reg_id, self.reg[reg_id as usize]);
            return Ok(self.reg[reg_id as usize]);
        }

        
        let ret_val = self.mem[mem_addr as usize];
        if ret_val > MAX_VALID_VAL {
            return Err("Attempted to return invalid value");
        }
        /*
        if ret_val > MAX_15_BIT_VAL {

            // Refers to a register value instead
            let reg_id = ret_val % MOD; 

            if reg_id > MAX_REG_ID {
                return Err("Attempted to read from register ID > 7 2");
            }

            println!("2. Ret_val: {:?}, Reading from reg {:?}, value {:?}", ret_val, reg_id, self.reg[reg_id as usize]);
            return Ok(self.reg[reg_id as usize]);
            //return Ok(reg_id);
        }
        */
        Ok(self.mem[mem_addr as usize])
    }


    fn mem_write (&mut self, mem_addr: u16, val: u16) -> Result<(), &'static str> {
        // This function writes to memory if mem_addr <= 32767,
        // or to registers r0..r7 if 32768 <= mem_addr <= 32775.
        // Requires cast from u16 to usize for indexing into memory
        
        if mem_addr > MAX_ADDR {
            return Err("Attempted to write value outside valid memory range");
        }

        if mem_addr > MAX_MEM_ADDR {
            // Write to registers, so take modulus of mem address
            let reg_id = mem_addr % MOD;
            if reg_id > 7 {
                return Err("Attempted to write to register ID > 7");
            }
            self.reg[reg_id as usize] = val;
            return Ok(());
        } 
        else {
            self.mem[mem_addr as usize] = val;
        }
        Ok(())
    }


    fn get_instr (&mut self) -> Result<(), &'static str> {
    // This function reads the next instruction from memory, matches
    // it to functions and executes the appropriate function.
    // Each function changes the program counter and stack pointer
    // as appropriate.

        let pc = self.pc;

        match self.mem_read(pc){
            Ok(opcd) => match opcd {
                0 => { /* HALT*/ 
                    self.halt = true;
                    println!("Halting at opcode {:?}", opcd);
                },
                1 => { /* SET a b */ 
                    self.set().unwrap();
                },
                2 => { /* PUSH a */ 
                    self.push().unwrap();
                },
                3 => { /* POP a */ 
                    self.pop().unwrap();
                },
                4 => { /* EQ a b c */ 
                    self.eq().unwrap();
                },
                5 => { /* GT a b c */ 
                    self.gt().unwrap();
                },
                6 => { /* JMP a */ 
                    self.jmp().unwrap();
                },
                7 => { /* JT a b */ 
                    self.jt().unwrap();
                    //println!("opcd: {:?}", opcd);
                },
                8 => { /* JF a b */ 
                    self.jf().unwrap();
                    //println!("opcd: {:?}", opcd);
                },
                9 => { /* ADD a b c */ 
                    self.add().unwrap();
                },
                10 => { /* MULT a b c */ self.halt = true;println!("Unknown opcode {:?}", opcd);},
                11 => { /* MOD a b c */ self.halt = true;println!("Unknown opcode {:?}", opcd);},
                12 => { /* AND a b c */ 
                    self.and().unwrap();
                },
                13 => { /* OR a b c */ 
                    self.or().unwrap();
                },
                14 => { /* NOT a b */ 
                    self.not().unwrap();
                },
                15 => { /* RMEM a b */ self.halt = true;println!("Unknown opcode {:?}", opcd);},
                16 => { /* WMEM a b */ self.halt = true;println!("Unknown opcode {:?}", opcd);},
                17 => { /* CALL a */ 
                    self.call().unwrap();
                },
                18 => { /* RET */ self.halt = true;println!("Unknown opcode {:?}", opcd);},
                19 => { 
                    self.out().unwrap();
                },
                20 => { /* IN a */ self.halt = true;println!("Unknown opcode {:?}", opcd);},
                21 => { /* NO-OP */ self.inc_pc();},
                _ => {
                    self.halt = true;
                    println!("Unrecognised instruction: {:?}", opcd);
                    return Err("Unrecognised instruction.");
                }, 

            },
            Err(msg) => return Err(msg),
        }
        Ok(())
    }

    fn set (&mut self) -> Result<(), &'static str> {
        self.inc_pc();
        let pc = self.pc;
        let reg_id = self.mem_read(pc)? % MOD;

        println!("PC at read reg id: {:?}, reg_id: {:?}", pc, reg_id);
        if reg_id > MAX_REG_ID {
            return Err("Attempted to write outside register IDs in set()");
        }

        self.inc_pc();
        let pc = self.pc;
        let value = self.mem_read(pc)?;
        self.reg[reg_id as usize] = value;
        println!("Setting register {:?} to {:?}", reg_id, value);
        self.inc_pc();
        Ok(())
    }


    fn push (&mut self) -> Result<(), &'static str> {
        // This function pushes a value onto the stack
        // and increments the stack pointer

        // Get value to push
        self.inc_pc();
        let pc = self.pc;

        let mut val = self.mem_read(pc)?;

        if val > MAX_15_BIT_VAL {
            // Refers to a value in register instead
            let reg_id = val % MOD;
            val = self.get_reg(reg_id).unwrap();
        }
        println!("Pushing val: {:?} onto stack", val);
        self.stack.push(val);

        self.inc_pc();
        Ok(())
    }


    fn pop (&mut self) -> Result<(), &'static str> {
        // This function pops the value from the top of the stack
        // and returns it. The stack pointer is also decremented.
        // If nothing on the stack, then it panics.

        // Get destination to write the result to
        self.inc_pc();
        let pc = self.pc;

        let dest = self.mem_read(pc)?;

        if let Some(val) = self.stack.pop() {
            self.mem_write(dest, val).unwrap();
            println!("Popped val {:?} off stack to dest {:?}", val, dest);
        }
        else {
            return Err("Tried to pop off an empty stack!");
        }

        self.inc_pc();
        Ok(())
    }


    fn get_reg (&mut self, reg_id: u16) -> Result<u16, &'static str> {
        if reg_id > MAX_REG_ID {
            return Err("Register ID larger than 7.");
        }
        Ok(self.reg[reg_id as usize])
    }

    fn eq (&mut self) -> Result<(), &'static str>{

        // Get destination to write the result to
        self.inc_pc();
        let pc = self.pc;

        let dest = self.mem_read(pc)?;

        // Get first value to compare
        self.inc_pc();
        let pc = self.pc;

        let mut val_1 = self.mem_read(pc)?;

        if val_1 > MAX_15_BIT_VAL {
            // Refers to a value in register instead
            let reg_id = val_1 % MOD;
            val_1 = self.get_reg(reg_id).unwrap();
        }

        // Get second value to compare
        self.inc_pc();
        let pc = self.pc;

        let mut val_2 = self.mem_read(pc)?;

        if val_2 > MAX_15_BIT_VAL {
            // Refers to a value in register instead
            let reg_id = val_2 % MOD;
            val_2 = self.get_reg(reg_id).unwrap();
        }

        let result = if val_1 == val_2 {
            1
        }
        else {
            0
        };
        println!("Eq: Writing {:?} == {:?} to dest: {:?}", val_1, val_2, dest);
        self.mem_write(dest, result).unwrap();

        self.inc_pc();

        Ok(())
    }


    fn gt (&mut self) -> Result<(), &'static str>{

        // Get destination to write the result to
        self.inc_pc();
        let pc = self.pc;

        let dest = self.mem_read(pc)?;

        // Get first value to compare
        self.inc_pc();
        let pc = self.pc;

        let mut val_1 = self.mem_read(pc)?;

        if val_1 > MAX_15_BIT_VAL {
            // Refers to a value in register instead
            let reg_id = val_1 % MOD;
            val_1 = self.get_reg(reg_id).unwrap();
        }

        // Get second value to compare
        self.inc_pc();
        let pc = self.pc;

        let mut val_2 = self.mem_read(pc)?;

        if val_2 > MAX_15_BIT_VAL {
            // Refers to a value in register instead
            let reg_id = val_2 % MOD;
            val_2 = self.get_reg(reg_id).unwrap();
        }

        let result = if val_1 > val_2 {
            1
        }
        else {
            0
        };
        println!("Gt: Writing {:?} > {:?} to dest: {:?}", val_1, val_2, dest);
        self.mem_write(dest, result).unwrap();

        self.inc_pc();

        Ok(())
    }


    fn and (&mut self) -> Result<(), &'static str>{

        // Get destination to write the result to
        self.inc_pc();
        let pc = self.pc;

        let dest = self.mem_read(pc)?;

        // Get first value to compare
        self.inc_pc();
        let pc = self.pc;

        let mut val_1 = self.mem_read(pc)?;

        if val_1 > MAX_15_BIT_VAL {
            // Refers to a value in register instead
            let reg_id = val_1 % MOD;
            val_1 = self.get_reg(reg_id).unwrap();
        }

        // Get second value to compare
        self.inc_pc();
        let pc = self.pc;

        let mut val_2 = self.mem_read(pc)?;

        if val_2 > MAX_15_BIT_VAL {
            // Refers to a value in register instead
            let reg_id = val_2 % MOD;
            val_2 = self.get_reg(reg_id).unwrap();
        }

        let result = val_1 & val_2;

        println!("And: Writing {:?} & {:?} to dest: {:?}", val_1, val_2, dest);
        self.mem_write(dest, result).unwrap();

        self.inc_pc();

        Ok(())
    }


    fn or (&mut self) -> Result<(), &'static str>{

        // Get destination to write the result to
        self.inc_pc();
        let pc = self.pc;

        let dest = self.mem_read(pc)?;

        // Get first value to compare
        self.inc_pc();
        let pc = self.pc;

        let mut val_1 = self.mem_read(pc)?;

        if val_1 > MAX_15_BIT_VAL {
            // Refers to a value in register instead
            let reg_id = val_1 % MOD;
            val_1 = self.get_reg(reg_id).unwrap();
        }

        // Get second value to compare
        self.inc_pc();
        let pc = self.pc;

        let mut val_2 = self.mem_read(pc)?;

        if val_2 > MAX_15_BIT_VAL {
            // Refers to a value in register instead
            let reg_id = val_2 % MOD;
            val_2 = self.get_reg(reg_id).unwrap();
        }

        let result = val_1 | val_2;

        println!("Or: Writing {:?} | {:?} to dest: {:?}", val_1, val_2, dest);
        self.mem_write(dest, result).unwrap();

        self.inc_pc();

        Ok(())
    }


    fn not (&mut self) -> Result<(), &'static str>{

        // Get destination to write the result to
        self.inc_pc();
        let pc = self.pc;

        let dest = self.mem_read(pc)?;

        // Get value to invert
        self.inc_pc();
        let pc = self.pc;

        let mut val = self.mem_read(pc)?;

        if val > MAX_15_BIT_VAL {
            // Refers to a value in register instead
            let reg_id = val % MOD;
            val = self.get_reg(reg_id).unwrap();
        }

        // Do bitwise not, and mask off top bit if it got set
        let result = (!val) & 0x7FFF;


        println!("Not: Writing !{:?} = {:?} to dest: {:?}", val, result, dest);
        self.mem_write(dest, result).unwrap();

        self.inc_pc();

        Ok(())
    }


    fn call (&mut self) -> Result<(), &'static str>{

        // Get destination to jump to
        self.inc_pc();
        let pc = self.pc;

        let jump_to_addr = self.mem_read(pc)?;

        // Increment program counter to point to the
        // next instruction, and push this onto the stack
        self.inc_pc();
        let pc = self.pc;

        self.stack.push(pc);
        println!("Pushed addr {:?} onto stack and jumped to {:?}", pc, jump_to_addr);
        // Set the program counter to the location to jump
        self.pc = jump_to_addr;

        Ok(())
    }



    fn jmp (&mut self) -> Result<(), &'static str> {
        self.inc_pc();
        let pc = self.pc;
        let addr = self.mem_read(pc)?;
        if addr > MAX_MEM_ADDR {
            return Err("Attempted to jump outside program memory");
        }
        println!("Jumping to {:?}", addr);
        self.set_pc(addr);
        Ok(())
    }

    fn jt (&mut self) -> Result<(), &'static str> {
        // Branches to addr if non-zero
        self.inc_pc();
        let pc = self.pc;
        let mut val_branch_if_nz = self.mem_read(pc)?;

        if val_branch_if_nz > MAX_15_BIT_VAL {
            // Refers to a value in register instead
            let reg_id = val_branch_if_nz % MOD;
            val_branch_if_nz = self.get_reg(reg_id).unwrap();
        }

        self.inc_pc();
        let pc = self.pc;
        let branch_addr = self.mem_read(pc)?;
        if val_branch_if_nz != 0 {
            //println!("JT Branching to {:?} val: {:?}, pc: {:?}", branch_addr, val_branch_if_nz, pc);
            self.set_pc(branch_addr);
        }
        else {
            //println!("JT Didn't branch: addr: {:?}, val: {:?}", branch_addr, val_branch_if_nz);
            self.inc_pc();
        }
        Ok(())
    }

    fn jf (&mut self) -> Result<(), &'static str> {
        // Branches to addr if zero
        self.inc_pc();
        let pc = self.pc;

        let mut val_branch_if_z = self.mem_read(pc)?;

        if val_branch_if_z > MAX_15_BIT_VAL {
            // Refers to a value in register instead
            let reg_id = val_branch_if_z % MOD;
            val_branch_if_z = self.get_reg(reg_id).unwrap();
        }

        self.inc_pc();
        let pc = self.pc;
        let branch_addr = self.mem_read(pc)?;

        if val_branch_if_z == 0 {
            self.set_pc(branch_addr);
            //println!("JF Branching to {:?}, val: {:?}", branch_addr, val_branch_if_z);
        }
        else {
            //println!("JF didn't branch. Addr: {:?}, val: {:?}", branch_addr, val_branch_if_z);
            self.inc_pc();
        }
        Ok(())
    }


    fn add (&mut self) -> Result<(), &'static str> {
        self.inc_pc();
        let pc = self.pc;

        let dest = self.mem_read(pc)?;

        self.inc_pc();
        let pc = self.pc;

        // Read first value from memory, read
        // from register if >32767
        let mut val_1 = self.mem_read(pc)?;
        if val_1 > MAX_15_BIT_VAL {
            // Value is in a register
            let reg_id = val_1 % MOD;
            val_1 = self.get_reg(reg_id).unwrap();
        }

        self.inc_pc();
        let pc = self.pc;

        // Read second value from memory, read
        // from register if >32767
        let mut val_2 = self.mem_read(pc)?;
        if val_2 > MAX_15_BIT_VAL {
            // Value is in a register
            let reg_id = val_2 % MOD;
            val_2 = self.get_reg(reg_id).unwrap();
        }
        println!("Add: Writing {:?} + {:?} to dest: {:?}", val_1, val_2, dest);
        self.mem_write(dest, val_1 + val_2)?;

        self.inc_pc();
        Ok(())

    }

    fn out (&mut self) -> Result<(), &'static str> {
        self.inc_pc();
        let pc = self.pc;

        let val = self.mem_read(pc)?;
        if val > 255 {
            return Err("Number too large, cannot be ascii.");
        }
        print!("{}", (val as u8) as char);
        self.inc_pc();
        Ok(())
    }

    fn run (&mut self) {
        while !self.halt {
            match self.get_instr() {
                Ok(()) => {},
                Err(msg) => {println!("{:?}", msg);}
            }
        }
    }


}

fn main() {

    // Read little-endian bytes from file into buffer
    let mut f = File::open("../../challenge.bin").unwrap();
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
        println!("Load memory returned error: {:?}", msg);
        panic!();
    }
    
    cpu.run();
}
