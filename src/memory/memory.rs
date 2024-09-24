#![allow(dead_code)]

use crate::collections::Stack;
use crate::screen::Screen;

const MEM_SIZE: usize = 4096;
const REGISTER_COUNT: usize = 16;

const MEM_START: usize = 0x200;

/// Struct that will not only hold all the information necessary but will
/// have the implementation to execute instructions based on its state.
#[allow(non_snake_case)]
pub struct Memory<'b, 'c> {

    /// Index register to point at locations in memory.
    pub I: usize,
    
    /// Register for pointing at the current instruction to load.
    pub pc: usize,

    /// Used to store the current operation code.
    pub opcode: u16,

    /// Used for timed events in programs and is decremented at a rate
    /// of 60hz until it reaches 0.
    pub delay_timer: u8,

    /// Stack used to store addresses to call and return from subroutines
    pub stack: Stack<usize>,

    /// We have 16 general purpose registers from V0 to VF, so we can represent
    /// each register as an array and use hexadecimal formatting to index each value.
    pub V: [u8; REGISTER_COUNT],

    /// Array used to actually behave like the main memory for a Chip-8 Interpreter.
    memory: [u8; MEM_SIZE],

    /// Screen reference for our actual program.
    /// The lifetimes specified include
    /// b -> holds the lifetime reference of Screen
    /// c -> holds the lifetime reference of Canvas inside Screen
    pub screen: &'b mut Screen<'c>,
}

#[allow(unused_variables)]
impl<'b, 'c> Memory<'b, 'c> {

    /// Contructor for our memory struct.
    pub fn new(screen: &'b mut Screen<'c>) -> Self {
        Memory {
            I: 0,
            pc: MEM_START,
            opcode: 0,
            delay_timer: 0,
            stack: Stack::new(MEM_SIZE),
            V: [0; REGISTER_COUNT],
            memory: [0; MEM_SIZE],
            screen,
        }
    }

    /// Loads the program into memory.
    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[MEM_START .. (MEM_START + program.len())].copy_from_slice(&program[..]);
    }

    /// Used to peek at the value of a specific memory location.
    pub fn read_byte(&self, index: usize) -> u8 {
        self.memory[index]
    }

    /// Wrapper function to call one fetch execute cycle.
    pub fn cycle_cpu(&mut self) {
        self.fetch();
        self.execute();
    }

    /// Fetch the next two bytes in memory and load them into our opcode.
    pub fn fetch(&mut self) {
        let mut hi: u16 = self.memory[self.pc] as u16;
        hi <<= 8;

        let lo: u16 = self.memory[self.pc + 1] as u16;
        self.opcode = hi | lo;
        self.pc += 2;
    }

    /// Determines the instruction to execute based on the current
    /// value of our opcode variable.
    pub fn execute(&mut self) {

        // Tuple for each nibble value present in our opcode
        let nibbles: (u8, u8, u8, u8) = (
            ((self.opcode & 0xF000) >> 12) as u8,
            ((self.opcode & 0x0F00) >> 8) as u8,
            ((self.opcode & 0x00F0) >> 4) as u8,
            ((self.opcode & 0x000F) >> 0) as u8,
        );

        // Read the potential input from the instruction based on the defined
        // Chip-8 conventions.
        let nnn: usize = (self.opcode & 0x0FFF) as usize;
        let nn: u8 = (self.opcode & 0x00FF) as u8;
        let x: usize = nibbles.1 as usize;
        let y: usize = nibbles.2 as usize;
        let n: u8 = nibbles.3 as u8;

        match nibbles {
            (0x0, 0x0, 0xe, 0x0)    => self.clear_screen(),
            (0x0, 0x0, 0xE, 0xE)    => self.return_from_subroutine(),
            (0x1, _, _, _)          => self.jump(nnn),
            (0x2, _, _, _)          => self.call_subroutine(nnn),
            (0x3, _, _, _)          => self.skip_if_equal(x, nn),
            (0x4, _, _, _)          => self.skip_if_not_equal(x, nn),
            (0x5, _, _, _)          => self.skip_if_registers_equal(x, y),
            (0x9, _, _, _)          => self.skip_if_registers_not_equal(x, y),
            (0x6, _, _, _)          => self.set_register(x, nn),
            (0x7, _, _, _)          => self.add_immediate(x, nn),
            (0xA, _, _, _)          => self.set_index(nnn),
            (0xD, _, _, _)          => self.display(x, y, n),
            (0x8, _, _, 0x0)        => self.set_vx_vy(x, y),
            (0x8, _, _, 0x1)        => self.binary_or(x, y),
            (0x8, _, _, 0x2)        => self.binary_and(x, y),
            (0x8, _, _, 0x3)        => self.logical_xor(x, y),
            (0x8, _, _, 0x4)        => self.add_registers(x, y),
            _ => {},
        }


    }

    /// OPCODE - 0x00E0
    /// 
    /// Calls the method on screen which will update all the pixels
    /// to zero and redraws the canvas.
    fn clear_screen(&mut self) {
        self.screen.clear();
    }

    /// OPCODE - 0x00EE
    /// 
    /// This will allow us to return from a subroutine by retrieving the last 
    /// address from the stack andsetting it to the program counter.
    fn return_from_subroutine(&mut self) {
        self.pc = self.stack.pop().unwrap();
    }

    /// OPCODE - 0x1NNN
    ///
    /// Sets the program counter to the parameter passed in the method.  We
    /// do not need to preserve the value of the program counter when jumping.
    fn jump(&mut self, nnn: usize) {
        self.pc = nnn;
    }

    /// OPCODE - 0x2NNN
    ///
    /// Sets the program counter to the parameter passed in the method.  Before
    /// doing so, we need to preserve the current value of the program counter
    /// by pushing it onto the stack.
    fn call_subroutine(&mut self, nnn: usize) {
        self.stack.push(self.pc);
        self.pc = nnn;
    }

    /// OPCODE - 0x3XNN
    ///
    /// Will look at the value in register V[x] and will increment the program counter
    /// by two, skipping the next instruction, if the value is equal to NN.
    fn skip_if_equal(&mut self, x: usize, nn: u8) {
        if self.V[x] == nn {
            self.pc += 2;
        }
    }

    /// OPCODE - 0x4XNN
    ///
    /// Will look at the value in register V[x] and will increment the program counter
    /// by two, skipping the next instruction, if the value is NOT equal to NN.
    fn skip_if_not_equal(&mut self, x: usize, nn: u8) {
        if self.V[x] != nn {
            self.pc += 2;
        }
    }

    /// OPCODE - 0x5XY0
    ///
    /// Will look at the values in register V[x] and V[y] and will increment the program
    /// counter by two, skipping the next instruction, if the register values are equal.
    fn skip_if_registers_equal(&mut self, x: usize, y: usize) {
        if self.V[x] == self.V[y] {
            self.pc += 2;
        }
    }

    /// OPCODE - 0xXY0
    ///
    /// Will look at the values in register V[x] and V[y] and will increment the program
    /// counter by two, skipping the next instruction, if the register values are equal.
    fn skip_if_registers_not_equal(&mut self, x: usize, y: usize) {
        if self.V[x] != self.V[y] {
            self.pc += 2;
        }
    }

    /// OPCODE - 0x6XNN
    /// 
    /// Sets the register V[x] to the value NN.
    fn set_register(&mut self, x: usize, nn: u8) {
        self.V[x] = nn;
    }

    /// OPCODE - 0x7XNN
    /// 
    /// Adds NN to the register V[x]. Chip-8 does not set any carry flag
    /// when an overflow occurs so just handle the wrapping of the value.
    fn add_immediate(&mut self, x: usize, nn: u8) {
        self.V[x] = nn.wrapping_add(self.V[x]);
    }

    /// OPCODE - 0xANNN
    /// 
    /// Sets the index register I to the value NNN.
    fn set_index(&mut self, nnn: usize) {
        self.I = nnn;
    }

    /// OPCODE - 0x8XY0
    /// 
    /// Sets the value of V[x] to V[y]
    fn set_vx_vy(&mut self, x: usize, y: usize) {
        self.V[x] = self.V[y];
    }

    /// OPCODE - 0x8XY1
    /// 
    /// V[x] is set to the OR of V[x] and V[y]
    fn binary_or(&mut self, x: usize, y: usize) {
        self.V[x] |= self.V[y]
    }

    /// OPCODE - 0x8XY2
    /// 
    /// V[x] is set to the AND of V[x] and V[y]
    fn binary_and(&mut self, x: usize, y: usize) {
        self.V[x] &= self.V[y]
    }

    /// OPCODE - 0x8XY3
    /// 
    /// V[x] is set to the XOR of V[x] and V[y]
    fn logical_xor(&mut self, x: usize, y: usize) {
        self.V[x] ^= self.V[y]
    }

    /// OPCODE - 0x8XY4
    /// 
    /// V[x] is set to the value of V[x] + V[y]
    /// In this instance, the addition will affect the carry flag if we end up overflowing.
    fn add_registers(&mut self, x: usize, y: usize) {
        let (result, overflowed) = self.V[x].overflowing_add(self.V[y]);

        self.V[x] = result;
        
        if overflowed {
            self.V[0xF] = 1;
        } else {
            self.V[0xF] = 0;
        }
    }

    /// OPCODE - 0xDXYN
    /// 
    /// This is the function for displaying Chip-8 graphics.
    fn display(&mut self, x: usize, y: usize, n: u8) {
        let vx = self.V[x] % 64;
        let vy = self.V[y] % 32;

        self.V[0xF] = 0;

        for y_val in 0..n {
            let sprite_data = self.memory[self.I + (y_val as usize)];

            // Sprite data contains an 8 bit value for each of the 8 pixels in the row
            // (x, yVal), so we need evaluate each potential bit and set the pixels accordingly
            for x_val in 0u8..8 {
                if sprite_data & (0x80 >> x_val) != 0 {

                    let x_coord = vx + x_val;
                    let y_coord = vy + y_val;

                    if self.screen.get_pixel(x_coord as usize, y_coord as usize) == 1{
                        self.V[0xF] = 1;
                    }

                    self.screen.update_pixel(x_coord as usize, y_coord as usize);
                }
            }
        }
        
        self.screen.update_screen = true;
    }
}