use Opcode::*;
#[derive(Copy, Clone, PartialEq)]
pub struct VRegister(pub(crate) usize);
impl From<VRegister> for u8 {
    fn from(value: VRegister) -> Self {
        0xF & value.0 as u8
    }
}
impl From<VRegister> for u16 {
    fn from(value: VRegister) -> Self {
        0xF & value.0 as u16
    }
}
pub enum Opcode {
    NoOp,
    ClS,
    Ret,
    Jp(u16),
    JpReg(u16),
    Call(u16),
    SEByte(VRegister, u8),
    SEReg(VRegister, VRegister),
    SNEByte(VRegister, u8),
    SNEReg(VRegister, VRegister),
    LdByte(VRegister, u8),
    LdReg(VRegister, VRegister),
    LdToI(u16),
    LdKey(VRegister),
    LdSpr(VRegister),
    LdDeci(VRegister),
    LdVxI(VRegister),
    LdIVx(VRegister),
    LdVxDT(VRegister),
    LdDTVx(VRegister),
    LdVxST(VRegister),
    AddVxByte(VRegister, u8),
    AddVxVy(VRegister, VRegister),
    AddIVx(VRegister),
    Or(VRegister, VRegister),
    And(VRegister, VRegister),
    Xor(VRegister, VRegister),
    Sub(VRegister, VRegister),
    ShR(VRegister, VRegister),
    SubN(VRegister, VRegister),
    ShL(VRegister, VRegister),
    Rnd(VRegister, u8),
    Drw(VRegister, VRegister, u8),
    SkP(VRegister),
    SkNP(VRegister),
    // Super Chip 1.x Instructions
    ScD(u8),
    ScR,
    ScL,
    Exit,
    LoRes,
    HiRes,
    SaveFlags(VRegister),
    LdFlags(VRegister),
    // Octo Instructions
    ScU(u8),
    LdILong(u16),
    LdIVxToVy(VRegister, VRegister),
    LdVxToVyI(VRegister, VRegister),
    SelectPlane(u8),
    StoreAudioBuffer,
    SetPitch(VRegister)
}

pub const fn nibble_op_code(opcode: u16) -> (u8, u8, u8, u8) {
    (
        ((opcode & 0xf000) >> 12) as u8,
        ((opcode & 0x0f00) >> 8) as u8,
        ((opcode & 0x00f0) >> 4) as u8,
        (opcode & 0x000f) as u8,
    )
}
pub fn decode_instruction(encoded_instruction: u16) -> Opcode {
    let nibbles = nibble_op_code(encoded_instruction);
    let n = nibbles.3;
    let kk = encoded_instruction as u8;
    let mmm = encoded_instruction & 0xfff;
    let (x_reg, y_reg) = (VRegister(nibbles.1 as usize), VRegister(nibbles.2 as usize));
    match nibbles {
        (0x0, 0x0, 0x0, 0x0) => NoOp,
        (0x0, 0x0, 0xC, _) => ScD(n),
        (0x0, 0x0, 0xD, _) => ScU(n),
        (0x0, 0x0, 0xE, 0x0) => ClS,
        (0x0, 0x0, 0xE, 0xE) => Ret,
        (0x0, 0x0, 0xF, 0xB) => ScR,
        (0x0, 0x0, 0xF, 0xC) => ScL,
        (0x0, 0x0, 0xF, 0xD) => Exit,
        (0x0, 0x0, 0xF, 0xE) => LoRes,
        (0x0, 0x0, 0xF, 0xF) => HiRes,
        (0x1, _, _, _) => Jp(mmm),
        (0x2, _, _, _) => Call(mmm),
        (0x3, _, _, _) => SEByte(x_reg, kk),
        (0x4, _, _, _) => SNEByte(x_reg, kk),
        (0x5, _, _, 0x0) => SEReg(x_reg, y_reg),
        (0x5, _, _, 2) => LdIVxToVy(x_reg, y_reg),
        (0x5, _, _, 3) => LdVxToVyI(x_reg, y_reg),
        (0x6, _, _, _) => LdByte(x_reg, kk),
        (0x7, _, _, _) => AddVxByte(x_reg, kk),
        (0x8, _, _, 0x0) => LdReg(x_reg, y_reg),
        (0x8, _, _, 0x1) => Or(x_reg, y_reg),
        (0x8, _, _, 0x2) => And(x_reg, y_reg),
        (0x8, _, _, 0x3) => Xor(x_reg, y_reg),
        (0x8, _, _, 0x4) => AddVxVy(x_reg, y_reg),
        (0x8, _, _, 0x5) => Sub(x_reg, y_reg),
        (0x8, _, _, 0x6) => ShR(x_reg, y_reg),
        (0x8, _, _, 0x7) => SubN(x_reg, y_reg),
        (0x8, _, _, 0xE) => ShL(x_reg, y_reg),
        (0x9, _, _, 0x0) => SNEReg(x_reg, y_reg),
        (0xA, _, _, _) => LdToI(mmm),
        (0xB, _, _, _) => JpReg(mmm),
        (0xC, _, _, _) => Rnd(x_reg, kk),
        (0xD, _, _, _) => Drw(x_reg, y_reg, n),
        (0xE, _, 0x9, 0xE) => SkP(x_reg),
        (0xE, _, 0xA, 0x1) => SkNP(x_reg),
        (0xF, n, 0x0, 0x1) => SelectPlane(n),
        (0xF, 0x0, 0x0, 0x2) => StoreAudioBuffer,
        (0xF, _, 0x3, 0xA) => SetPitch(x_reg),
        (0xF, _, 0x0, 0x7) => LdDTVx(x_reg),
        (0xF, _, 0x0, 0xA) => LdKey(x_reg),
        (0xF, _, 0x1, 0x5) => LdVxDT(x_reg),
        (0xF, _, 0x1, 0x8) => LdVxST(x_reg),
        (0xF, _, 0x1, 0xE) => AddIVx(x_reg),
        (0xF, _, 0x2, 0x9) => LdSpr(x_reg),
        (0xF, _, 0x3, 0x3) => LdDeci(x_reg),
        (0xF, _, 0x5, 0x5) => LdVxI(x_reg),
        (0xF, _, 0x6, 0x5) => LdIVx(x_reg),
        (0xF, _, 0x7, 0x5) => SaveFlags(x_reg),
        (0xF, _, 0x8, 0x5) => LdFlags(x_reg),
        (_, _, _, _) => NoOp,
    }
}

pub fn encode_opcode(decoded_opcode: Opcode) -> u16 {
    use Opcode::*;
    match decoded_opcode {
        NoOp => 0x0,
        ScD(n) => (0x00C0 | (n & 0xF)) as u16,
        ClS => 0x00E0,
        Ret => 0x00EE,
        ScR => 0x00FB,
        ScL => 0x00FC,
        Exit => 0x00FD,
        LoRes => 0x00FE,
        HiRes => 0x00FF,
        Jp(nnn) => (nnn & 0xFFF) | 0x1000,
        Call(nnn) => (nnn & 0xFFF) | 0x2000,
        SEByte(x_reg, kk) => 0x3000 | (u16::from(x_reg) << 8) | kk as u16,
        SNEByte(x_reg, kk) => 0x4000 | (u16::from(x_reg) << 8) | kk as u16,
        SEReg(x_reg, y_reg) => 0x5000 | (u16::from(x_reg) << 8) | u16::from(y_reg),
        LdByte(x_reg, kk) => 0x6000 | (u16::from(x_reg) << 8) | kk as u16,
        AddVxByte(x_reg, kk) => 0x7000 | (u16::from(x_reg) << 8) | kk as u16,
        LdReg(x_reg, y_reg) => 0x8000 | (u16::from(x_reg) << 8) | u16::from(y_reg) << 4,
        Or(x_reg, y_reg) => 0x8001 | (u16::from(x_reg) << 8) | (u16::from(y_reg) << 4),
        And(x_reg, y_reg) => 0x8002 | (u16::from(x_reg) << 8) | (u16::from(y_reg) << 4),
        Xor(x_reg, y_reg) => 0x8003 | (u16::from(x_reg) << 8) | (u16::from(y_reg) << 4),
        AddVxVy(x_reg, y_reg) => 0x8004 | (u16::from(x_reg) << 8) | (u16::from(y_reg) << 4),
        Sub(x_reg, y_reg) => 0x8005 | (u16::from(x_reg) << 8) | (u16::from(y_reg) << 4),
        ShR(x_reg, y_reg) => 0x8006 | (u16::from(x_reg) << 8) | (u16::from(y_reg) << 4),
        SubN(x_reg, y_reg) => 0x8007 | (u16::from(x_reg) << 8) | (u16::from(y_reg) << 4),
        ShL(x_reg, y_reg) => 0x800E | (u16::from(x_reg) << 8) | (u16::from(y_reg) << 4),
        SNEReg(x_reg, y_reg) => 0x9000 | (u16::from(x_reg) << 8) | (u16::from(y_reg) << 4),
        LdToI(nnn) => 0xA000u16 | (nnn & 0xFFF),
        JpReg(nnn) => (nnn & 0xFFF) | 0xB000,
        Rnd(x_reg, kk) => 0xC000 | (u16::from(x_reg) << 8) | kk as u16,
        Drw(x_reg, y_reg, n) => {
            0xD000 | (u16::from(x_reg) << 8) | (u16::from(y_reg) << 4) | (0xF & n as u16)
        }
        SkP(x_reg) => 0xE09E | (u16::from(x_reg) << 8),
        SkNP(x_reg) => 0xE0A1 | (u16::from(x_reg) << 8),
        LdDTVx(x_reg) => 0xF007 | (u16::from(x_reg) << 8),
        LdKey(x_reg) => 0xF00A | (u16::from(x_reg) << 8),
        LdVxDT(x_reg) => 0xF015 | (u16::from(x_reg) << 8),
        LdVxST(x_reg) => 0xF018 | (u16::from(x_reg) << 8),
        AddIVx(x_reg) => 0xF01E | (u16::from(x_reg) << 8),
        LdSpr(x_reg) => 0xF029 | (u16::from(x_reg) << 8),
        LdDeci(x_reg) => 0xF033 | (u16::from(x_reg) << 8),
        LdVxI(x_reg) => 0xF055 | (u16::from(x_reg) << 8),
        LdIVx(x_reg) => 0xF065 | (u16::from(x_reg) << 8),
        LdILong(_) => panic!("LdILong not supported in encode_opcode, use encode_as_bytes instead"),
        _ => 0x0,
    }
}
