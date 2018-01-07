use std::io;
use std::fs::File;
use std::io::Write;

const MOD: u16 = 32768;
const MAX_ADDR: u16 = 32775;
const MAX_VALID_VAL: u16 = 32775;
const MAX_MEM_ADDR: u16 = 32767;
const MEM_CAPACITY: usize = 32768;
const MAX_15_BIT_VAL: u16 = 32767;
const MAX_REG_ID: u16 = 7;

// TODO:
// - Consistent error handling - pass Err() upwards, use ?,
//   get rid of lazy unwrap()s unless it makes sense to panic
// - Pass new program counter as output of inc_pc()
// - Standardise the checking of addresses for registers
// - Make use of more idiomatic control flow expressions for assigning
//   values, e.g. if let Some(), etc. Replace let mut val_1 etc.
// - Write opcode 20 to read from stdin
// - Add checks for invalid numbers > 32775
// - Write binary -> assembly translator, replacing opcodes and registers
//   with names, and ascii codes with letters where appropriate
// - Wrap little-endian > big-endian conversion into own module

pub struct CPU {
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

    // Cycle counter
    cc: u32,

    // Execution halt flag
    halt: bool,

    // Flag for enabling/disabling setting a breakpoint at
    // a specific cycle count
    break_at_cc: bool,

    // Flag for enabling/disabling setting a breakpoint at
    // a specific program counter
    break_at_pc: bool,

    input_buffer: String,

    logging: bool,

    logfile: File,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            reg: vec![0; 8],
            mem: vec![0; MEM_CAPACITY],
            stack: vec![],
            pc: 0,
            halt: false,
            cc: 0,
            break_at_cc: false,
            break_at_pc: false,
            input_buffer: String::new(),
            logging: false,
            logfile: File::create("inst_log.txt").unwrap(),
        }
    }

    pub fn load_mem (&mut self, mem_input: Vec<u16>) -> Result<(), &'static str> {
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

    fn mem_dump (&self, minus: usize, plus: usize) {
        println!("Dumping mem from pc-{:?}={:?} to pc+{:?}={:?}\nCurrent pc: {:?}",
            minus, self.pc - minus as u16,
            plus, self.pc + plus as u16,
            self.pc);

        println!("{:?}", 
            self.mem.iter()
                .skip((self.pc as usize)-minus).take(minus + plus + 1)
                .collect::<Vec<_>>());
    }

    fn reg_dump (&self) {
        println!("Dumping registers at pc={:?}", self.pc);
        println!("{:?}", self.reg);
    }


    fn get_reg (&mut self, reg_id: u16) -> Result<u16, &'static str> {
        if reg_id > MAX_REG_ID {
            return Err("Register ID larger than 7.");
        }
        Ok(self.reg[reg_id as usize])
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
                    println!("Halting at opcode {:?}. PC: {:?}, CC: {:?}", opcd, self.pc, self.cc);
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
                },
                8 => { /* JF a b */ 
                    self.jf().unwrap();
                },
                9 => { /* ADD a b c */ 
                    self.add().unwrap();
                },
                10 => { /* MULT a b c */ 
                    self.mult().unwrap();
                },
                11 => { /* MOD a b c */ 
                    self.modulo().unwrap();
                },
                12 => { /* AND a b c */ 
                    self.and().unwrap();
                },
                13 => { /* OR a b c */ 
                    self.or().unwrap();
                },
                14 => { /* NOT a b */ 
                    self.not().unwrap();
                },
                15 => { /* RMEM a b */ 
                    self.rmem().unwrap();
                },
                16 => { /* WMEM a b */ 
                    self.wmem().unwrap();
                },
                17 => { /* CALL a */ 
                    self.call().unwrap();
                },
                18 => { /* RET */ 
                    self.ret().unwrap();
                },
                19 => { 
                    self.out().unwrap();
                },
                20 => { /* IN a */ 
                    self.in_stdin().unwrap();
                },
                21 => { /* NO-OP */ self.inc_pc();},
                _ => {
                    self.halt = true;
                    println!("Unrecognised instruction: {:?}", opcd);
                    self.mem_dump(5,10);
                    self.reg_dump();
                    return Err("Unrecognised instruction.");
                }, 

            },
            Err(msg) => return Err(msg),
        }
        Ok(())
    }

    /* opcodes */

    fn set (&mut self) -> Result<(), &'static str> {
        // This function sets a given register to a given value
        // SET a b

        self.inc_pc();
        let pc = self.pc;
        let reg_id = self.mem_read(pc)? % MOD;

        if reg_id > MAX_REG_ID {
            return Err("Attempted to write outside register IDs in set()");
        }

        self.inc_pc();
        let pc = self.pc;
        let mut value = self.mem_read(pc)?;

        if value > MAX_15_BIT_VAL {
            // Setting from another register's value
            let reg_id2 = value % MOD;
            value = self.get_reg(reg_id2).unwrap();
        }

        self.reg[reg_id as usize] = value;

        //println!("Setting register {:?} to {:?}", reg_id, value);
        self.inc_pc();

        if self.logging {
            write!(self.logfile, "set r{} {}\n", reg_id, value).unwrap();
        }

        Ok(())
    }


    fn push (&mut self) -> Result<(), &'static str> {
        // This function pushes a value onto the stack
        // and increments the stack pointer
        // PUSH a

        // Get value to push
        self.inc_pc();
        let pc = self.pc;

        let mut val = self.mem_read(pc)?;

        if val > MAX_15_BIT_VAL {
            // Refers to a value in register instead
            let reg_id = val % MOD;
            val = self.get_reg(reg_id).unwrap();
            if self.logging {
                write!(self.logfile, "push r{}\n", reg_id).unwrap();
            }
        }
        else {
            if self.logging {
                write!(self.logfile, "push {}\n", val).unwrap();
            }
        }
        //println!("Pushing val: {:?} onto stack", val);
        self.stack.push(val);

        self.inc_pc();

        Ok(())
    }


    fn pop (&mut self) -> Result<(), &'static str> {
        // This function pops the value from the top of the stack
        // and writes it to memory.
        // If nothing on the stack, then it panics.
        // POP a

        // Get destination to write the result to
        self.inc_pc();
        let pc = self.pc;

        let dest = self.mem_read(pc)?;

        if let Some(val) = self.stack.pop() {
            self.mem_write(dest, val).unwrap();
            if self.logging {
                write!(self.logfile, "pop {}\n", dest).unwrap();
            }
        }
        else {
            return Err("Tried to pop off an empty stack!");
        }

        self.inc_pc();

        

        Ok(())
    }


    fn eq (&mut self) -> Result<(), &'static str>{
        // This function sets <a> to 1 if <b> is equal to <c>;
        // set it to 0 otherwise
        // EQ a b c

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

        self.mem_write(dest, (val_1==val_2) as u16).unwrap();

        if self.logging {
            write!(self.logfile, "eq {} {} {}\n", dest, val_1, val_2).unwrap();
        }

        self.inc_pc();

        Ok(())
    }


    fn gt (&mut self) -> Result<(), &'static str>{
        // set <a> to 1 if <b> is greater than <c>; set it to 0 otherwise
        // GT a b c

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
        //println!("Gt: Writing {:?} > {:?} to dest: {:?}", val_1, val_2, dest);
        self.mem_write(dest, result).unwrap();

        self.inc_pc();

        if self.logging {
            write!(self.logfile, "gt {} {} {}\n", dest, val_1, val_2).unwrap();
        }

        Ok(())
    }


    fn jmp (&mut self) -> Result<(), &'static str> {
        // jump to <a>
        // JMP a

        self.inc_pc();
        let pc = self.pc;
        let addr = self.mem_read(pc)?;
        if addr > MAX_MEM_ADDR {
            return Err("Attempted to jump outside program memory");
        }
        //println!("Jumping to {:?}", addr);
        self.set_pc(addr);

        if self.logging {
            write!(self.logfile, "jmp {}\n", addr).unwrap();
        }

        Ok(())
    }


    fn jt (&mut self) -> Result<(), &'static str> {
        // if <a> is nonzero, jump to <b>
        // JT a b

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

        if self.logging {
            write!(self.logfile, "jt {} {}\n", val_branch_if_nz, branch_addr).unwrap();
        }

        Ok(())
    }

    fn jf (&mut self) -> Result<(), &'static str> {
        //   if <a> is zero, jump to <b>
        // JF a b

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

        if self.logging {
            write!(self.logfile, "jf {} {}\n", val_branch_if_z, branch_addr).unwrap();
        }

        Ok(())
    }


    fn add (&mut self) -> Result<(), &'static str> {
        // assign into <a> the sum of <b> and <c> (modulo 32768)
        // ADD a b c

        self.inc_pc();
        let pc = self.pc;

        // Get destination address to write to
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
        //println!("Add: Writing {:?} + {:?} to dest: {:?}", val_1, val_2, dest);
        self.mem_write(dest, (val_1 + val_2) % MOD)?;

        self.inc_pc();

        if self.logging {
            write!(self.logfile, "add {} {} {}\n", dest, val_1, val_2).unwrap();
        }

        Ok(())
    }


    fn mult (&mut self) -> Result<(), &'static str> {
        // store into <a> the product of <b> and <c> (modulo 32768)
        // MULT a b c

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
        //println!("Mult: Writing {:?} * {:?} to dest: {:?}", val_1, val_2, dest);
        self.mem_write(dest, ((val_1 as u32 * val_2 as u32 ) % MOD as u32) as u16)?;

        self.inc_pc();

        if self.logging {
            write!(self.logfile, "mult {} {} {}\n", dest, val_1, val_2).unwrap();
        }

        Ok(())
    }

    fn modulo (&mut self) -> Result<(), &'static str> {
        // store into <a> the remainder of <b> divided by <c>
        // MOD a b c

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
        //println!("Mod: Writing {:?} % {:?} to dest: {:?}", val_1, val_2, dest);

        self.mem_write(dest, val_1 % val_2)?;

        self.inc_pc();

        if self.logging {
            write!(self.logfile, "mod {} {} {}\n", dest, val_1, val_2).unwrap();
        }

        Ok(())
    }

    fn and (&mut self) -> Result<(), &'static str>{
        // stores into <a> the bitwise and of <b> and <c>
        // AND a b c

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

        //println!("And: Writing {:?} & {:?} to dest: {:?}", val_1, val_2, dest);
        self.mem_write(dest, result).unwrap();

        self.inc_pc();

        if self.logging {
            write!(self.logfile, "and {} {} {}\n", dest, val_1, val_2).unwrap();
        }

        Ok(())
    }


    fn or (&mut self) -> Result<(), &'static str>{
        // stores into <a> the bitwise or of <b> and <c>
        // OR a b c 

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

        //println!("Or: Writing {:?} | {:?} to dest: {:?}", val_1, val_2, dest);
        self.mem_write(dest, result).unwrap();

        self.inc_pc();

        if self.logging {
            write!(self.logfile, "or {} {} {}\n", dest, val_1, val_2).unwrap();
        }

        Ok(())
    }


    fn not (&mut self) -> Result<(), &'static str>{
        // stores 15-bit bitwise inverse of <b> in <a>
        // NOT a b

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


        //println!("Not: Writing !{:?} = {:?} to dest: {:?}", val, result, dest);
        self.mem_write(dest, result).unwrap();

        self.inc_pc();

        if self.logging {
            write!(self.logfile, "not {} {}\n", dest, val).unwrap();
        }

        Ok(())
    }


    fn rmem (&mut self) -> Result<(), &'static str> {
        // read memory at address <b> and write it to <a>
        // RMEM a b

        //let pc = self.pc;
        //println!("Mem dump at pc = {:?}: {:?}", pc, self.mem.iter().skip(pc as usize).take(30).collect::<Vec<_>>());
        
        // Get destination to write the result to
        self.inc_pc();
        let pc = self.pc;

        let dest_addr = self.mem_read(pc)?;

        // Get source address to read from.
        // Can be memory or register
        self.inc_pc();
        let pc = self.pc;

        let mut src_addr = self.mem_read(pc)?;
        
        if src_addr > MAX_15_BIT_VAL {
            // Address to read is instead held in register
            let reg_id = src_addr % MOD;
            src_addr = self.get_reg(reg_id).unwrap();
        }

        // Read value from source address
        let val = self.mem_read(src_addr)?;
        if val > MAX_15_BIT_VAL {
            return Err("RMEM: Register contained another register's address.
                     Probably not valid.");
        }

        // Write this value to destination
        self.mem_write(dest_addr, val)?;

        //println!("RMEM: Reading from addr {:?} val {:?} and writing to addr {:?}", src_addr, val, dest_addr);
        self.inc_pc();

        if self.logging {
            write!(self.logfile, "rmem {} {}\n", dest_addr, val).unwrap();
        }

        Ok(())
    }


    fn wmem (&mut self) -> Result<(), &'static str> {
        // write the value from <b> into memory at address <a>
        // WMEM a b

        //let pc = self.pc;
        //println!("Mem dump at pc = {:?}: {:?}", pc-2, self.mem.iter().skip(pc as usize - 2).take(30).collect::<Vec<_>>());
        
        // Get destination to write the result to
        self.inc_pc();
        let pc = self.pc;


        let mut dest_addr = self.mem_read(pc)?;

        if dest_addr > MAX_15_BIT_VAL {
            // Destination address to write to is
            // held in a register
            let reg_id = dest_addr % MOD;
            dest_addr = self.get_reg(reg_id).unwrap()
        }

        // Get source address to read from.
        // Can be address in memory or a register address
        self.inc_pc();
        let pc = self.pc;

        let src_addr = self.mem_read(pc)?;

        let val = if src_addr > MAX_15_BIT_VAL {
            // Value to write is held in register
            let reg_id = src_addr % MOD;
            self.get_reg(reg_id).unwrap()
        }
        else {
            src_addr
            //self.mem_read(src_addr).unwrap()
        };


        // Write this value to destination
        self.mem_write(dest_addr, val)?;
        //println!("WMEM: Writing value {:?} to {:?}", val, dest_addr);
        
        self.inc_pc();

        if self.logging {
            write!(self.logfile, "wmem {} {}\n", dest_addr, val).unwrap();
        }

        Ok(())
    }


    fn call (&mut self) -> Result<(), &'static str> {
        // write the address of the next instruction to the stack and jump to <a>.
        // CALL a

        // Get destination to jump to
        self.inc_pc();
        let pc = self.pc;

        let mut jump_to_addr = self.mem_read(pc)?;

        if jump_to_addr > MAX_15_BIT_VAL {
            // Refers to a value in register instead
            let reg_id = jump_to_addr % MOD;
            jump_to_addr = self.get_reg(reg_id).unwrap();
        }

        // Increment program counter to point to the
        // next instruction, and push this onto the stack
        self.inc_pc();
        let pc = self.pc;

        self.stack.push(pc);
        //println!("CALL: Pushed addr {:?} onto stack and jumped to {:?}", pc, jump_to_addr);

        // Set the program counter to the location to jump
        self.pc = jump_to_addr;

        if self.logging {
            write!(self.logfile, "call {}\n", jump_to_addr).unwrap();
        }

        Ok(())
    }


    fn ret (&mut self) -> Result<(), &'static str> {
        // remove the top element from the stack and jump to it; empty stack = halt
        // RET

        if self.stack.is_empty() {
            self.halt = true;
            println!("RET: Halting at empty stack");
            return Err("Ret: Empty stack");
        }
        else {
            let ret_addr = self.stack.pop().unwrap();

            // Set the program counter to the location to jump
            self.pc = ret_addr;

            if self.logging {
                write!(self.logfile, "ret to {}\n", ret_addr).unwrap();
            }

            return Ok(());
        }
    }


    fn out (&mut self) -> Result<(), &'static str> {
        // write the character represented by ascii code <a> to the terminal
        // OUT a
        // if self.logging {
        //     self.logging = false;
        //     println!("Disabling instruction logging");
        // }
        
        self.inc_pc();
        let pc = self.pc;

        let mut val = self.mem_read(pc)?;

        if val > MAX_15_BIT_VAL {
            // ASCII code is in register
            let reg_id = val % MOD;
            val = self.get_reg(reg_id).unwrap();
        }


        if val > 255 {
            self.halt = true;
                println!("ERROR: Invalid ASCII code: {:?}", val);
                self.mem_dump(5,10);
                self.reg_dump();
            return Err("Number too large, cannot be ascii.");
        }
        print!("{}", (val as u8) as char);

        if self.logging {
            write!(self.logfile, "out {}\n", (val as u8) as char).unwrap();
        }

        self.inc_pc();
        Ok(())
    }

    fn in_stdin (&mut self) -> Result<(), &'static str> {
        // read character from stdin and write ascii code to <a>
        // IN a
        
        // Get destination to write the result to
        self.inc_pc();
        let pc = self.pc;

        let dest = self.mem_read(pc)?;

        // Check if there are still characters in buffer to be read.
        // If not, then read string from stdin to fill buffer
        if self.input_buffer.is_empty() {
            io::stdin().read_line(&mut self.input_buffer).unwrap();
        }

        if self.input_buffer == "DUMP\n" {
            println!("PC: {:?}", self.pc);
            self.reg_dump();
            self.mem_dump(0,10);
            let mut buf = File::create("memdump.txt").unwrap();
            for i in 0..self.mem.len() {
                write!(buf,"{}\n", self.mem[i]).unwrap();
            }
            
        }
        else if self.input_buffer == "LOG_START\n" {
            println!("Enabling instruction logging");
            self.logging = true;
            self.input_buffer.clear();
            io::stdin().read_line(&mut self.input_buffer).unwrap();
        }
        else if self.input_buffer == "LOG_END\n" {
            println!("Disabling instruction logging");
            self.logging = false;
            self.input_buffer.clear();
            io::stdin().read_line(&mut self.input_buffer).unwrap();
        }
        else if self.input_buffer == "FIX\n" {
            println!("Setting r7 to 5");
            self.reg[7] = 5;
            self.input_buffer.clear();
            io::stdin().read_line(&mut self.input_buffer).unwrap();
        }


        

        // Read first character in buffer and write to <a>, then remove
        let ch = self.input_buffer.remove(0);

        if ch as u16 > 127 {
            return Err("Invalid ASCII code");
        } 
        
        self.mem_write(dest, ch as u16).unwrap();
        self.inc_pc();
        Ok(())
    }

    pub fn run (&mut self, breakpoint_cc: u32, breakpoint_pc: u16) {

        if breakpoint_cc != 0 {
            self.break_at_cc = true;
        }
        if breakpoint_pc != 0 {
            self.break_at_pc = true;
        }

        while !self.halt {
            match self.get_instr() {
                Ok(()) => {},
                Err(msg) => {
                    println!("{:?}", msg);
                    break;
                }
            }
            self.cc += 1;

            // Debug
            if self.cc == breakpoint_cc && self.break_at_cc {
                self.halt = true;
                println!("Breakpoint at cycle count = {:?}", self.cc);
                self.mem_dump(5,10);
                self.reg_dump();
            }

            if self.pc == breakpoint_pc && self.break_at_pc {
                self.halt = true;
                println!("Breakpoint at program counter = {:?}", self.pc);
                self.mem_dump(5,10);
                self.reg_dump();
            }
        }
    }
}