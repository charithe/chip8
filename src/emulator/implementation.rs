use super::common::{Error, Result};
use super::display;
use super::interpreter::*;
use log::debug;
use rand::Rng;
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
    Draw(display::Pixels),
    WaitForKey,
    Exit,
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
    keyboard: [bool; 16],
    rom_end: usize,
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
            keyboard: [false; 16],
            rom_end: 0,
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

        self.rom_end = i + MEM_START;

        Ok(())
    }

    fn unload_rom(&mut self) {
        for i in MEM_START..MEM_SIZE {
            self.memory[i] = 0u8;
        }

        self.rom_end = MEM_START;
    }

    fn next_instruction(&mut self) -> Option<Instruction> {
        if (self.pc < MEM_START) || (self.pc >= self.rom_end) {
            return None;
        }

        let ins = ((self.memory[self.pc] as u16) << 8) | (self.memory[self.pc + 1] as u16);
        self.pc += 2;

        Some(Instruction(ins))
    }

    pub fn step(&mut self) -> StepResult {
        // decrease delay timer
        if self.dt > 0 {
            self.dt -= 1;
        }

        // decrease sound timer
        if self.st > 0 {
            self.st -= 1;
        }

        match self.next_instruction() {
            Some(ins) => {
                let op = ins.interpret()?;
                debug!("EXEC:\t{}\t{}", ins, op);

                match op {
                    Op::ADD(reg, val) => self.do_add(reg, val),
                    Op::ADDI(reg) => self.do_addi(reg),
                    Op::ADDR(reg1, reg2) => self.do_addr(reg1, reg2),
                    Op::AND(reg1, reg2) => self.do_and(reg1, reg2),
                    Op::CALL(addr) => self.do_call(addr),
                    Op::CLS => self.do_cls(),
                    Op::CPDT(reg) => self.do_cpdt(reg),
                    Op::DRW(reg1, reg2, val) => self.do_drw(reg1, reg2, val),
                    Op::JP(addr) => self.do_jp(addr),
                    Op::JPREL(addr) => self.do_jprel(addr),
                    Op::LD(reg, val) => self.do_ld(reg, val),
                    Op::LDDT(reg) => self.do_lddt(reg),
                    Op::LDI(addr) => self.do_ldi(addr),
                    Op::LDIB(reg) => self.do_ldib(reg),
                    Op::LDIM(reg) => self.do_ldim(reg),
                    Op::LDIR(reg) => self.do_ldir(reg),
                    Op::LDIS(reg) => self.do_ldis(reg),
                    Op::LDKP(reg) => self.do_ldkp(reg),
                    Op::LDR(reg1, reg2) => self.do_ldr(reg1, reg2),
                    Op::LDST(reg) => self.do_ldst(reg),
                    Op::OR(reg1, reg2) => self.do_or(reg1, reg2),
                    Op::RET => self.do_ret(),
                    Op::RND(reg, val) => self.do_rnd(reg, val),
                    Op::SE(reg, val) => self.do_se(reg, val),
                    Op::SER(reg1, reg2) => self.do_ser(reg1, reg2),
                    Op::SHL(reg) => self.do_shl(reg),
                    Op::SHR(reg) => self.do_shr(reg),
                    Op::SKNP(reg) => self.do_sknp(reg),
                    Op::SKP(reg) => self.do_skp(reg),
                    Op::SNE(reg, val) => self.do_sne(reg, val),
                    Op::SNER(reg1, reg2) => self.do_sner(reg1, reg2),
                    Op::SUB(reg1, reg2) => self.do_sub(reg1, reg2),
                    Op::SUBN(reg1, reg2) => self.do_subn(reg1, reg2),
                    Op::SYS(addr) => self.do_sys(addr),
                    Op::XOR(reg1, reg2) => self.do_xor(reg1, reg2),
                }
            }
            None => Ok(Some(Step::Exit)),
        }
    }

    pub fn key_press(&mut self, key: Input) {
        debug!("KEY PRESS: {:?}", key);
        self.keyboard[key as usize] = true;
    }

    pub fn key_release(&mut self, key: Input) {
        debug!("KEY RELEASE: {:?}", key);
        self.keyboard[key as usize] = false;
    }

    fn pressed_key(&self) -> Option<u8> {
        self.keyboard.iter().enumerate().find_map(|(i, pressed)| {
            if *pressed == true {
                Some(i as u8)
            } else {
                None
            }
        })
    }

    fn is_pressed(&self, key: u8) -> bool {
        self.keyboard[key as usize]
    }

    fn do_add(&mut self, reg: Register, val: Value) -> StepResult {
        let Register(r) = reg;
        let a = self.vx[r as usize] as u16;
        let b = val.0 as u16;
        self.vx[r as usize] = (a + b) as u8;
        Ok(Some(Step::Nop))
    }

    fn do_addi(&mut self, reg: Register) -> StepResult {
        self.i += self.vx[reg] as u16;

        Ok(Some(Step::Nop))
    }

    fn do_addr(&mut self, reg1: Register, reg2: Register) -> StepResult {
        let (v, overflow) = self.vx[reg1].overflowing_add(self.vx[reg2]);
        self.vx[reg1] = v;

        if overflow {
            self.vx[0xF] = 1;
        } else {
            self.vx[0xF] = 0;
        }

        Ok(Some(Step::Nop))
    }

    fn do_and(&mut self, reg1: Register, reg2: Register) -> StepResult {
        self.vx[reg1] &= self.vx[reg2];
        Ok(Some(Step::Nop))
    }

    fn do_call(&mut self, addr: Address) -> StepResult {
        self.push_to_stack(self.pc)?;
        self.pc = addr.into();
        Ok(Some(Step::Nop))
    }

    fn do_cls(&mut self) -> StepResult {
        self.screen.clear();
        Ok(Some(Step::Draw(self.screen.pixels())))
    }

    fn do_cpdt(&mut self, reg: Register) -> StepResult {
        self.vx[reg] = self.dt;
        Ok(Some(Step::Nop))
    }

    fn do_drw(&mut self, reg1: Register, reg2: Register, n: Value) -> StepResult {
        let x = self.vx[reg1];
        let y = self.vx[reg2];
        let sprite_data = self.memory[self.i as usize..(self.i + n.0 as u16) as usize].to_vec();

        if let Some(v) = self.screen.draw(display::Sprite::new(x, y, sprite_data)) {
            self.vx[0xF] = v;
        }

        Ok(Some(Step::Draw(self.screen.pixels())))
    }

    fn do_jp(&mut self, addr: Address) -> StepResult {
        self.pc = addr.into();
        Ok(Some(Step::Nop))
    }

    fn do_jprel(&mut self, addr: Address) -> StepResult {
        self.pc = addr.into();
        self.pc += self.vx[0x0] as usize;
        Ok(Some(Step::Nop))
    }

    fn do_ld(&mut self, reg: Register, val: Value) -> StepResult {
        self.vx[reg] = val.into();
        Ok(Some(Step::Nop))
    }

    fn do_lddt(&mut self, reg: Register) -> StepResult {
        self.dt = self.vx[reg];
        Ok(Some(Step::Nop))
    }

    fn do_ldi(&mut self, addr: Address) -> StepResult {
        self.i = addr.into();
        Ok(Some(Step::Nop))
    }

    fn do_ldib(&mut self, reg: Register) -> StepResult {
        let val = self.vx[reg];
        let bcd = to_bcd(val);

        for j in 0usize..3usize {
            self.memory[self.i as usize + j] = bcd[j]
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

    fn do_ldir(&mut self, reg: Register) -> StepResult {
        let Register(x) = reg;
        for r in 0..=x {
            self.memory[self.i as usize + r as usize] = self.vx[r as usize];
        }

        Ok(Some(Step::Nop))
    }

    fn do_ldis(&mut self, reg: Register) -> StepResult {
        let digit = self.vx[reg];
        self.i = (digit * 4) as u16;
        Ok(Some(Step::Nop))
    }

    fn do_ldkp(&mut self, reg: Register) -> StepResult {
        if let Some(key) = self.pressed_key() {
            self.vx[reg] = key;
            Ok(Some(Step::Nop))
        } else {
            self.pc -= 2;
            Ok(Some(Step::WaitForKey))
        }
    }

    fn do_ldr(&mut self, reg1: Register, reg2: Register) -> StepResult {
        self.vx[reg1] = self.vx[reg2];
        Ok(Some(Step::Nop))
    }

    fn do_ldst(&mut self, reg: Register) -> StepResult {
        self.st = self.vx[reg];
        Ok(Some(Step::Nop))
    }

    fn do_or(&mut self, reg1: Register, reg2: Register) -> StepResult {
        self.vx[reg1] |= self.vx[reg2];
        Ok(Some(Step::Nop))
    }

    fn do_ret(&mut self) -> StepResult {
        let addr = self.pop_from_stack()?;
        self.pc = addr;
        Ok(Some(Step::Nop))
    }

    fn do_rnd(&mut self, reg: Register, val: Value) -> StepResult {
        let mut rng = rand::thread_rng();
        let r = rng.gen_range(std::u8::MIN, std::u8::MAX);
        self.vx[reg] = r & val.0;

        Ok(Some(Step::Nop))
    }

    fn do_se(&mut self, reg: Register, val: Value) -> StepResult {
        if self.vx[reg] == val.into() {
            self.pc += 2;
        }

        Ok(Some(Step::Nop))
    }

    fn do_ser(&mut self, reg1: Register, reg2: Register) -> StepResult {
        if self.vx[reg1] == self.vx[reg2] {
            self.pc += 2;
        }

        Ok(Some(Step::Nop))
    }

    fn do_shl(&mut self, reg: Register) -> StepResult {
        if self.vx[reg] & 0x80 == 0 {
            self.vx[0xF] = 0;
        } else {
            self.vx[0xF] = 1;
        }

        self.vx[reg] = self.vx[reg] << 1;
        Ok(Some(Step::Nop))
    }

    fn do_shr(&mut self, reg: Register) -> StepResult {
        self.vx[0xF] = self.vx[reg] & 0x01;
        self.vx[reg] = self.vx[reg] >> 1;
        Ok(Some(Step::Nop))
    }

    fn do_sknp(&mut self, reg: Register) -> StepResult {
        let key = self.vx[reg];
        if !self.is_pressed(key) {
            self.pc += 2;
        }

        Ok(Some(Step::Nop))
    }

    fn do_skp(&mut self, reg: Register) -> StepResult {
        let key = self.vx[reg];
        if self.is_pressed(key) {
            self.pc += 2;
        }

        Ok(Some(Step::Nop))
    }

    fn do_sne(&mut self, reg: Register, val: Value) -> StepResult {
        if self.vx[reg] != val.into() {
            self.pc += 2;
        }

        Ok(Some(Step::Nop))
    }

    fn do_sner(&mut self, reg1: Register, reg2: Register) -> StepResult {
        if self.vx[reg1] != self.vx[reg2] {
            self.pc += 2;
        }

        Ok(Some(Step::Nop))
    }

    fn do_sub(&mut self, reg1: Register, reg2: Register) -> StepResult {
        let (v, overflow) = self.vx[reg1].overflowing_sub(self.vx[reg2]);

        self.vx[reg1] = v;

        if overflow {
            self.vx[0xF] = 1;
        } else {
            self.vx[0xF] = 0;
        }

        Ok(Some(Step::Nop))
    }

    fn do_subn(&mut self, reg1: Register, reg2: Register) -> StepResult {
        if self.vx[reg2] > self.vx[reg1] {
            self.vx[0xF] = 1;
        } else {
            self.vx[0xF] = 0;
        }

        self.vx[reg1] = self.vx[reg2].saturating_sub(self.vx[reg1]);
        Ok(Some(Step::Nop))
    }

    fn do_sys(&mut self, _addr: Address) -> StepResult {
        // ignored
        Ok(Some(Step::Nop))
    }

    fn do_xor(&mut self, reg1: Register, reg2: Register) -> StepResult {
        self.vx[reg1] ^= self.vx[reg2];
        Ok(Some(Step::Nop))
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
}
