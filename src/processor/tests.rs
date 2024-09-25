use crate::WIDTH;
use crate::HEIGHT;

use crate::screen::Screen;
use crate::processor::processor::Processor;

#[test]
fn test_load() {
    let mut screen: Screen = Screen::new(None);
    let mut processor: Processor = Processor::new(&mut screen);

    processor.load(vec![0x1, 0x2, 0x3]);

    assert_eq!(processor.read_byte(0x200), 0x1);
    assert_eq!(processor.read_byte(0x201), 0x2);
    assert_eq!(processor.read_byte(0x202), 0x3);
    assert_eq!(processor.read_byte(0x203), 0x0);
}

#[test]
fn test_fetch() {
    let mut screen: Screen = Screen::new(None);
    let mut processor: Processor = Processor::new(&mut screen);

    processor.load(vec![0xFF, 0x01, 0x01, 0x01, 0xF0, 0xAF]);

    processor.fetch();
    assert_eq!(processor.opcode, 0xFF01);

    processor.fetch();
    assert_eq!(processor.opcode, 0x0101);

    processor.fetch();
    assert_eq!(processor.opcode, 0xF0AF);
}

#[test]
fn test_clear() {
    let mut screen: Screen = Screen::new(None);

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            screen.set_pixel(x, y, 1);
        }
    }

    let mut processor: Processor = Processor::new(&mut screen);

    processor.load(vec![0x00, 0xe0]);

    processor.fetch();
    processor.execute();

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            assert_eq!(processor.screen.get_pixel(x, y), 0)
        }
    }
}

#[test]
fn test_jump() {
    let mut screen: Screen = Screen::new(None);
    let mut processor: Processor = Processor::new(&mut screen);

    processor.load(vec![0x1A, 0xBC]);

    processor.fetch();
    processor.execute();

    assert_eq!(processor.pc, 0xABC);
}

#[test]
fn test_subroutine() {
    let mut screen: Screen = Screen::new(None);
    let mut processor: Processor = Processor::new(&mut screen);

    processor.load(vec![0x00, 0x00, 0x22, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x00, 0xEE]);

    // 4 CPU cycles
    for _ in 0..4 {
        processor.fetch();
        processor.execute();
    }

    assert_eq!(processor.pc, 0x204);
}

#[test]
fn test_set_register() {
    let mut screen: Screen = Screen::new(None);
    let mut processor: Processor = Processor::new(&mut screen);

    processor.load(vec![0x63, 0xAB, 0x60, 0xCD, 0x6F, 0xEF]);
    
    // 3 cpu cycles.
    for _ in 0..3 {
        processor.fetch();
        processor.execute();
    }

    assert_eq!(processor.V[0x3], 0xAB);
    assert_eq!(processor.V[0x0], 0xCD);
    assert_eq!(processor.V[0xF], 0xEF);
}

#[test]
fn test_skip_if_equal() {
    let mut screen: Screen = Screen::new(None);
    let mut processor: Processor = Processor::new(&mut screen);

    // 63FF -> Set register 3 to 0xFF
    // 33FF -> Compare register 3 to 0xFF
    // CDEF -> Bogus opcode, should be skipped since V[x] = 0xFF
    // 3302 -> Compare register 3 to 0x02
    // ABCD -> Bogus opcode, but will be the value loaded since V[x] = 0x02
    processor.load(vec![0x63, 0xFF, 0x33, 0xFF, 0xCD, 0xEF, 0x33, 0x02, 0xAB, 0xCD, 0x98, 0x76]);

    // 63FF -> Set register 3 to 0xFF
    processor.fetch();
    processor.execute();

    // 33FF -> Compare register 3 to 0xFF
    processor.fetch();
    processor.execute();

    // Retrieve the next opcode, the program counter should be incremented
    // since V[x] = 0xFF
    processor.fetch();

    assert_eq!(processor.opcode, 0x3302);

    // 4302 -> Compare register 3 to 0x02
    processor.execute();

    // Retrieve the next opcode, since V[x] != 0x02 we the next fetch should retrieve
    // 0xABCD as it is the next opcode in memory.
    processor.fetch();

    assert_eq!(processor.opcode, 0xABCD);  
}

#[test]
fn test_skip_if_not_equal() {
    let mut screen: Screen = Screen::new(None);
    let mut processor: Processor = Processor::new(&mut screen);

    // 6302 -> Set register 3 to 0x02
    // 43FF -> Compare register 3 to 0xFF
    // CDEF -> Bogus opcode, should be skipped since V[x] != 0xFF
    // 4302 -> Compare register 3 to 0x02
    // ABCD -> Bogus opcode, but will be the value loaded since V[x] = 0x02
    processor.load(vec![0x63, 0x02, 0x43, 0xFF, 0xCD, 0xEF, 0x43, 0x02, 0xAB, 0xCD, 0x98, 0x76]);

    // 6302 -> Set register 3 to 0x02
    processor.fetch();
    processor.execute();

    // 33FF -> Compare register 3 to 0xFF
    processor.fetch();
    processor.execute();

    // Retrieve the next opcode
    processor.fetch();

    assert_eq!(processor.opcode, 0x4302);

    // 4302 -> Compare register 3 to 0x02
    processor.execute();

    // Retrieve the next opcode
    processor.fetch();

    assert_eq!(processor.opcode, 0xABCD);
}

#[test]
fn test_skip_if_registers_equal() {
    let mut screen: Screen = Screen::new(None);
    let mut processor: Processor = Processor::new(&mut screen);

    // 63AA -> Set register V[3] to 0xAA
    // 64AA -> Set register V[4] to 0xAA
    // 65BB -> Set register V[5] to 0xBB
    // 5340 -> Compare and skip next instruction if V[3] == V[4]
    // CDEF -> Bogus opcode, should be skipped since V[3] == V[4]
    // 5350 -> Compare V[3] == V[5]. These values are not equal and so do not skip
    // ABCD -> Bogus opcode, but should be read on next fetch.
    processor.load(vec![0x63, 0xAA, 0x64, 0xAA, 0x65, 0xBB, 0x53, 0x40, 0xCD, 0xEF, 0x53, 0x50, 0xAB, 0xCD, 0x12, 0x34]);

    // 63AA -> Set register V[3] to 0xAA
    processor.fetch();
    processor.execute();

    // 64AA -> Set register V[4] to 0xAA
    processor.fetch();
    processor.execute();

    // 65BB -> Set register V[5] to 0xBB
    processor.fetch();
    processor.execute();

    // 5340 -> Compare and skip next instruction if V[3] == V[4]
    processor.fetch();
    processor.execute();
    
    // CDEF -> Bogus opcode, should be skipped since V[3] == V[4]
    // 5350 -> Compare V[3] == V[5]. These values are not equal and so do not skip
    processor.fetch();
    assert_eq!(processor.opcode, 0x5350);
    processor.execute();

    // ABCD -> Bogus opcode, but should be read on next fetch.   
    processor.fetch();
    assert_eq!(processor.opcode, 0xABCD); 
}

#[test]
fn test_skip_if_registers_not_equal() {
    let mut screen: Screen = Screen::new(None);
    let mut processor: Processor = Processor::new(&mut screen);

    // 63AA -> Set register V[3] to 0xAA
    // 64AA -> Set register V[4] to 0xBB
    // 65BB -> Set register V[5] to 0xAA
    // 9340 -> Compare and skip next instruction since V[3] != V[4]
    // CDEF -> Bogus opcode, should be skipped since V[3] != V[4]
    // 9350 -> Compare V[3] == V[5]. These values are equal and so do not skip
    // ABCD -> Bogus opcode, but should be read on next fetch.
    processor.load(vec![0x63, 0xAA, 0x64, 0xBB, 0x65, 0xAA, 0x93, 0x40, 0xCD, 0xEF, 0x93, 0x50, 0xAB, 0xCD, 0x12, 0x34]);

    // 63AA -> Set register V[3] to 0xAA
    processor.fetch();
    processor.execute();

    // 64AA -> Set register V[4] to 0xBB
    processor.fetch();
    processor.execute();

    // 65BB -> Set register V[5] to 0xAA
    processor.fetch();
    processor.execute();

    // 9340 -> Compare and skip next instruction since V[3] != V[4]
    processor.fetch();
    processor.execute();
    
    // CDEF -> Bogus opcode, should be skipped since V[3] != V[4]
    // 5350 -> Compare V[3] == V[5]. These values are equal and so do not skip
    processor.fetch();
    assert_eq!(processor.opcode, 0x9350);

    processor.execute();

    // ABCD -> Bogus opcode, but should be read on next fetch.   
    processor.fetch();
    assert_eq!(processor.opcode, 0xABCD); 
}

#[test]
fn test_add_immediate() {
    let mut screen: Screen = Screen::new(None);
    let mut processor: Processor = Processor::new(&mut screen);

    // 63AA -> Set register V[3] to 0xAA
    // 64AA -> Set register V[4] to 0xBB
    // 65BB -> Set register V[5] to 0xAA
    // 9340 -> Compare and skip next instruction since V[3] != V[4]
    // CDEF -> Bogus opcode, should be skipped since V[3] != V[4]
    // 9350 -> Compare V[3] == V[5]. These values are equal and so do not skip
    // ABCD -> Bogus opcode, but should be read on next fetch.
    processor.load(vec![0x63, 0x01, 0x73, 0x04, 0x64, 0xFF, 0x74, 0x01]);

    processor.cycle_cpu();
    processor.cycle_cpu();

    assert_eq!(processor.V[3], 0x05);

    processor.cycle_cpu();
    processor.cycle_cpu();

    assert_eq!(processor.V[4], 0x00);
}