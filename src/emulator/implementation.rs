use super::common::{Error, Result};
use super::display;
use super::interpreter::*;
use log::debug;
use std::io::Read;

const REG_COUNT: usize = 16;
const MEM_SIZE: usize = 4096;
const MEM_START: usize = 512;
const STACK_SIZE: usize = 16;

const FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, //0
    0x20, 0x60, 0x20, 0x20, 0x70, //1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, //3
    0x90, 0x90, 0xF0, 0x10, 0x10, //4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, //5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, //6
    0xF0, 0x10, 0x20, 0x40, 0x40, //7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
    0xF0, 0x90, 0xF0, 0x90, 0x90, //A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, //B
    0xF0, 0x80, 0x80, 0x80, 0xF0, //C
    0xE0, 0x90, 0x90, 0x90, 0xE0, //D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
    0xF0, 0x80, 0xF0, 0x80, 0x80, //F
];

pub type StepResult = Result<Option<Step>>;

pub enum Step {
    Nop,
    ReadKey,
    Draw(display::Pixels),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Input {
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    Quit,
}

pub struct Emulator {
    vx: [u8; REG_COUNT], // V0 - VF registers
    dt: u8,              // delay timer
    st: u8,              // sound timer
    sp: u8,              // stack pointer
    i: u16,              // I
    pc: usize,           // program counter
    memory: [u8; MEM_SIZE],
    stack: [usize; STACK_SIZE],
    screen: display::Screen,
    input: Option<Input>,
}

impl Emulator {
    pub fn new<R: Read>(rom_data: R) -> Result<Emulator> {
        let mut emu = Emulator {
            vx: [0u8; REG_COUNT],
            dt: 0u8,
            st: 0u8,
            sp: 0u8,
            i: 0u16,
            pc: MEM_START,
            memory: [0u8; MEM_SIZE],
            stack: [0; STACK_SIZE],
            screen: display::Screen::default(),
            input: None,
        };

        emu.memory[..80].copy_from_slice(&FONT_SET[..]);
        emu.load_rom(rom_data)?;
        Ok(emu)
    }

    fn load_rom<R: Read>(&mut self, rom_data: R) -> Result<()> {
        self.unload_rom();
        debug!("Loading ROM");

        let mut i = 0;
        for byte in rom_data.bytes() {
            self.memory[i + MEM_START] = byte?;

            i += 1;
            if i + MEM_START >= MEM_SIZE {
                return Err(Error::InvalidROM);
            }
        }

        Ok(())
    }

    fn unload_rom(&mut self) {
        for i in MEM_START..MEM_SIZE {
            self.memory[i] = 0u8;
        }
    }

    fn next_instruction(&mut self) -> Option<Instruction> {
        if (self.pc < MEM_START) || (self.pc >= MEM_SIZE - 2) {
            return None;
        }

        let ins = ((self.memory[self.pc] as u16) << 8) | (self.memory[self.pc + 1] as u16);
        self.pc += 2;

        Some(Instruction(ins))
    }

    pub fn step(&mut self) -> StepResult {
        match self.next_instruction() {
            Some(ins) => {
                let op = ins.interpret()?;
                debug!("{}\t{}", ins, op);

                match op {
                    Op::CLS => self.do_cls(),
                    Op::JP(addr) => self.do_jp(addr),
                    Op::CALL(addr) => self.do_call(addr),
                    Op::LD(reg, val) => self.do_ld(reg, val),
                    Op::LDI(addr) => self.do_ldi(addr),
                    Op::LDIS(reg) => self.do_ldis(reg),
                    Op::LDIB(reg) => self.do_ldib(reg),
                    Op::LDIR(reg) => self.do_ldir(reg),
                    Op::LDIM(reg) => self.do_ldim(reg),
                    Op::DRW(reg1, reg2, val) => self.do_drw(reg1, reg2, val),
                    _ => unreachable!(),
                }
            }
            _ => Ok(None),
        }
    }

    pub fn send_input(&mut self, input: Input) {
        self.input = Some(input);
    }

    fn do_cls(&mut self) -> StepResult {
        self.screen.clear();
        Ok(Some(Step::Draw(self.screen.pixels())))
    }

    fn do_call(&mut self, addr: Address) -> StepResult {
        self.push_to_stack(self.pc)?;
        self.pc = addr.into();
        Ok(Some(Step::Nop))
    }

    fn do_jp(&mut self, addr: Address) -> StepResult {
        self.pc = addr.into();
        Ok(Some(Step::Nop))
    }

    fn do_ld(&mut self, reg: Register, val: Value) -> StepResult {
        let x: usize = reg.into();
        self.vx[x] = val.0;
        Ok(Some(Step::Nop))
    }

    fn do_ldi(&mut self, addr: Address) -> StepResult {
        self.i = addr.into();
        Ok(Some(Step::Nop))
    }

    fn do_ldis(&mut self, reg: Register) -> StepResult {
        let digit = self.get_register(reg);
        self.i = (digit * 4) as u16;
        Ok(Some(Step::Nop))
    }

    fn do_ldib(&mut self, reg: Register) -> StepResult {
        let val = self.get_register(reg);
        let bcd = to_bcd(val);

        for j in 0usize..3usize {
            self.memory[self.i as usize + j] = bcd[j]
        }

        Ok(Some(Step::Nop))
    }

    fn do_ldir(&mut self, reg: Register) -> StepResult {
        let Register(x) = reg;
        for r in 0..=x {
            self.memory[self.i as usize + r as usize] = self.vx[r as usize];
        }

        Ok(Some(Step::Nop))
    }

    fn do_ldim(&mut self, reg: Register) -> StepResult {
        let Register(x) = reg;
        for r in 0..=x {
            self.vx[r as usize] = self.memory[self.i as usize + r as usize];
        }

        Ok(Some(Step::Nop))
    }

    fn do_drw(&mut self, reg1: Register, reg2: Register, n: Value) -> StepResult {
        let x = self.get_register(reg1);
        let y = self.get_register(reg2);
        let sprite_data = self.memory[self.i as usize..(self.i + n.0 as u16) as usize].to_vec();

        if let Some(v) = self.screen.draw(display::Sprite::new(x, y, sprite_data)) {
            self.vx[0xF] = v;
        }

        Ok(Some(Step::Draw(self.screen.pixels())))
    }

    fn push_to_stack(&mut self, val: usize) -> Result<()> {
        if self.sp as usize >= STACK_SIZE {
            return Err(Error::StackOverflow);
        }

        self.stack[self.sp as usize] = val;
        self.sp += 1;
        Ok(())
    }

    fn pop_from_stack(&mut self) -> Result<usize> {
        if self.sp == 0 {
            return Err(Error::StackUnderflow);
        }

        self.sp -= 1;
        Ok(self.stack[self.sp as usize])
    }

    fn get_register(&self, reg: Register) -> u8 {
        let Register(x) = reg;
        self.vx[x as usize]
    }
}
