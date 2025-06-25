// TODO implement the few remaining opcodes that SST doesn't test

use crate::{gb::GameBoy, memory::Memory, util::*};

impl GameBoy {
    pub fn fetch_decode_execute(&mut self, opcode: u8) -> Option<u8> {
        match opcode {
            0x00 => Some(1),
            0x07 => {
                // RLCA
                let ms_bit = self.registers.a & 0x80;
                self.registers.a = (self.registers.a << 1) | (ms_bit >> 7);
                self.set_flag_c(ms_bit != 0);
                self.set_flag_n(false);
                self.set_flag_h(false);
                self.set_flag_z(false);
                Some(1)
            }

            0x0F => {
                // RRCA
                let ls_bit = self.registers.a & 1;
                self.registers.a = (self.registers.a >> 1) | (ls_bit << 7);
                self.set_flag_c(ls_bit != 0);
                self.set_flag_n(false);
                self.set_flag_h(false);
                self.set_flag_z(false);
                Some(1)
            }
            0x17 => {
                // RLA
                let msb = self.registers.a & 0x80;
                self.registers.a = (self.registers.a << 1) | self.get_flag_c();
                self.set_flag_c(msb != 0);
                self.set_flag_z(false);
                self.set_flag_n(false);
                self.set_flag_h(false);
                Some(1)
            }
            0x1F => {
                // RRA
                let lsb = self.registers.a & 1;
                self.registers.a = (self.registers.a >> 1) | (self.get_flag_c() << 7);
                self.set_flag_c(lsb != 0);
                self.set_flag_z(false);
                self.set_flag_n(false);
                self.set_flag_h(false);
                Some(1)
            }
            0x27 => {
                // DAA
                if self.get_flag_n() != 0 {
                    let mut adjustment = 0;
                    if self.get_flag_h() != 0 {
                        adjustment += 6;
                    }
                    if self.get_flag_c() != 0 {
                        adjustment += 0x60
                    }
                    self.registers.a -= adjustment;
                } else {
                    let mut adjustment = 0;
                    if (self.get_flag_h() != 0) || ((self.registers.a & 0xF) > 9) {
                        adjustment += 6;
                    }
                    if (self.get_flag_c() != 0) || (self.registers.a > 0x99) {
                        adjustment += 0x60;
                        self.set_flag_c(true);
                    }
                    self.registers.a += adjustment;
                }
                self.set_flag_z(self.registers.a == 0);
                self.set_flag_h(false);
                Some(1)
            }
            0x2F => {
                // CPL
                self.registers.a = !self.registers.a;
                self.set_flag_n(true);
                self.set_flag_h(true);
                Some(1)
            }
            0x37 => {
                // SCF
                self.set_flag_n(false);
                self.set_flag_h(false);
                self.set_flag_c(true);
                Some(1)
            }
            0x3F => {
                // CCF
                self.set_flag_n(false);
                self.set_flag_h(false);
                self.set_flag_c(self.get_flag_c() == 0);
                Some(1)
            }

            0x08 => {
                let nn_lsb: u8 = self.memory.read(self.registers.pc);
                self.registers.pc += 1;
                let nn_msb: u8 = self.memory.read(self.registers.pc);
                self.registers.pc += 1;
                let mut nn: u16 = unsigned_16(nn_msb, nn_lsb);
                self.memory.write(nn, lsb(self.registers.sp));
                nn += 1;
                self.memory.write(nn, msb(self.registers.sp));
                Some(5)
            }
            0x10 => {
                // TODO verify this is correct behaviour.
                // Not sure it matters that much, i think only CGB uses this
                self.running = false;
                Some(1)
            }
            0x18 => {
                let e = self.memory.read(self.registers.pc) as i8;
                self.registers.pc += 1;
                self.registers.pc = (self.registers.pc as i16 + e as i16) as u16;
                Some(3)
            }
            0x76 => {
                self.running = false;
                None
            }
            0xCB => {
                // 0xCB prefixed opcodes
                let cb_opcode: u8 = self.memory.read(self.registers.pc);
                let r8: u8 = cb_opcode & 0b111;
                let bit = (cb_opcode >> 3) & 0b111;
                match cb_opcode >> 6 {
                    0b00 => {
                        match (cb_opcode >> 3) & 0b111 {
                            0 => {
                                // RLC
                                self.registers.pc += 1;
                                let ms_bit = self.get_r8(r8) & 0x80;
                                self.set_flag_c(ms_bit != 0);
                                self.set_r8(r8, (self.get_r8(r8) << 1) | (ms_bit >> 7));
                                self.set_flag_n(false);
                                self.set_flag_h(false);
                                self.set_flag_z(self.get_r8(r8) == 0);
                                Some(2)
                            }
                            1 => {
                                // RRC
                                self.registers.pc += 1;
                                let ls_bit = self.get_r8(r8) & 0x01;
                                self.set_flag_c(ls_bit != 0);
                                self.set_r8(r8, (self.get_r8(r8) >> 1) | (ls_bit << 7));
                                self.set_flag_n(false);
                                self.set_flag_h(false);
                                self.set_flag_z(self.get_r8(r8) == 0);
                                Some(2)
                            }
                            2 => {
                                // RL
                                self.registers.pc += 1;
                                let ms_bit = self.get_r8(r8) & 0x80;
                                self.set_r8(r8, (self.get_r8(r8) << 1) | self.get_flag_c());
                                self.set_flag_c(ms_bit != 0);
                                self.set_flag_n(false);
                                self.set_flag_h(false);
                                self.set_flag_z(self.get_r8(r8) == 0);
                                Some(2)
                            }
                            3 => {
                                // RR
                                self.registers.pc += 1;
                                let ls_bit = self.get_r8(r8) & 0x01;
                                self.set_r8(r8, (self.get_r8(r8) >> 1) | self.get_flag_c() << 7);
                                self.set_flag_c(ls_bit != 0);
                                self.set_flag_n(false);
                                self.set_flag_h(false);
                                self.set_flag_z(self.get_r8(r8) == 0);
                                Some(2)
                            }
                            4 => {
                                // SLA
                                self.registers.pc += 1;
                                let ms_bit = self.get_r8(r8) & 0x80;
                                self.set_r8(r8, (self.get_r8(r8) << 1) | 0);
                                self.set_flag_z(self.get_r8(r8) == 0);
                                self.set_flag_n(false);
                                self.set_flag_h(false);
                                self.set_flag_c(ms_bit != 0);
                                Some(2)
                            }
                            5 => {
                                // SRA
                                self.registers.pc += 1;
                                let ms_bit = self.get_r8(r8) & 0x80;
                                let ls_bit = self.get_r8(r8) & 0x01;
                                self.set_r8(r8, (self.get_r8(r8) >> 1) | ms_bit);
                                self.set_flag_z(self.get_r8(r8) == 0);
                                self.set_flag_n(false);
                                self.set_flag_h(false);
                                self.set_flag_c(ls_bit != 0);
                                Some(2)
                            }
                            6 => {
                                // SWAP
                                self.registers.pc += 1;
                                let r8 = cb_opcode & 0b111;
                                let r8_value = self.get_r8(r8);
                                let r8_high_shift = (r8_value & 0xF0) >> 4;
                                let r8_low_shift = (r8_value & 0x0F) << 4;
                                let result = r8_high_shift | r8_low_shift;
                                self.set_r8(r8, result);
                                self.set_flag_z(result == 0);
                                self.set_flag_n(false);
                                self.set_flag_h(false);
                                self.set_flag_c(false);
                                Some(2)
                            }
                            7 => {
                                // SRL
                                self.registers.pc += 1;
                                let r8 = cb_opcode & 0b111;
                                let ls_bit = self.get_r8(r8) & 0x01;
                                self.set_r8(r8, self.get_r8(r8) >> 1);
                                self.set_flag_c(ls_bit != 0);
                                self.set_flag_n(false);
                                self.set_flag_h(false);
                                self.set_flag_z(self.get_r8(r8) == 0);
                                Some(2)
                            }
                            _ => None,
                        }
                    }
                    0b01 => {
                        self.registers.pc += 1;
                        self.set_flag_z((self.get_r8(r8) & (1 << bit)) == 0);
                        self.set_flag_n(false);
                        self.set_flag_h(true);
                        Some(2)
                    }
                    0b10 => {
                        self.registers.pc += 1;
                        self.set_r8(r8, self.get_r8(r8) & !(1 << bit));
                        Some(2)
                    }
                    0b11 => {
                        self.registers.pc += 1;
                        self.set_r8(r8, self.get_r8(r8) | (1 << bit));
                        Some(2)
                    }
                    _ => None,
                }
            }
            0xE0 => {
                let n = self.memory.read(self.registers.pc);
                self.registers.pc += 1;
                self.memory.write(unsigned_16(0xFF, n), self.registers.a);
                Some(3)
            }
            0xE8 => {
                // ADD SP i8
                let e: i8 = self.memory.read(self.registers.pc) as i8;
                self.registers.pc += 1;
                self.set_flag_z(false);
                self.set_flag_n(false);
                // ugly but I think all of these casts are necessary
                self.set_flag_h((((self.registers.sp & 0xF) as i16) + ((e & 0xF) as i16)) > 0xF);
                self.set_flag_c((((self.registers.sp & 0xFF) as i16) + (e as i16 & 0xFF)) > 0xFF);
                self.registers.sp = (self.registers.sp as i16 + e as i16) as u16;
                Some(4)
            }
            0xEA => {
                // LD (u16), A
                let lsb = self.memory.read(self.registers.pc);
                self.registers.pc += 1;
                let msb = self.memory.read(self.registers.pc);
                self.registers.pc += 1;
                self.memory.write(unsigned_16(msb, lsb), self.registers.a);
                Some(4)
            }
            0xF0 => {
                // LD A FF00 + u8
                let lsb = self.memory.read(self.registers.pc);
                self.registers.pc += 1;
                self.registers.a = self.memory.read(unsigned_16(0xFF, lsb));
                Some(3)
            }
            0xF2 => {
                self.registers.a = self.memory.read(unsigned_16(0xFF, self.registers.c));
                Some(2)
            }
            0xF8 => {
                // LD HL SP + i8
                let e: i8 = self.memory.read(self.registers.pc) as i8;
                self.registers.pc += 1;
                self.set_flag_z(false);
                self.set_flag_n(false);
                // ugly but I think all of these casts are necessary
                self.set_flag_h((((self.registers.sp & 0xF) as i16) + ((e & 0xF) as i16)) > 0xF);
                self.set_flag_c((((self.registers.sp & 0xFF) as i16) + (e as i16 & 0xFF)) > 0xFF);
                self.set_hl((self.registers.sp as i16 + e as i16) as u16);

                Some(3)
            }
            0xF9 => {
                self.registers.sp = self.get_hl();
                Some(2)
            }
            0xE2 => {
                self.memory
                    .write(unsigned_16(0xFF, self.registers.c), self.registers.a);
                Some(2)
            }
            0xFA => {
                let lsb = self.memory.read(self.registers.pc);
                self.registers.pc += 1;
                let msb = self.memory.read(self.registers.pc);
                self.registers.pc += 1;
                self.registers.a = self.memory.read(unsigned_16(msb, lsb));
                Some(4)
            }
            0xCD => {
                // CALL u16
                let ls_byte = self.memory.read(self.registers.pc);
                self.registers.pc += 1;
                let ms_byte = self.memory.read(self.registers.pc);
                self.registers.pc += 1;
                self.registers.sp -= 1;
                self.memory.write(self.registers.sp, msb(self.registers.pc));
                self.registers.sp -= 1;
                self.memory.write(self.registers.sp, lsb(self.registers.pc));
                self.registers.pc = unsigned_16(ms_byte, ls_byte);
                Some(6)
            }

            0xC9 => {
                // RET
                let lsb = self.memory.read(self.registers.sp);
                self.registers.sp += 1;
                let msb = self.memory.read(self.registers.sp);
                self.registers.sp += 1;
                self.registers.pc = unsigned_16(msb, lsb);
                Some(4)
            }
            0xD9 => {
                // RETI
                let lsb = self.memory.read(self.registers.sp);
                self.registers.sp += 1;
                let msb = self.memory.read(self.registers.sp);
                self.registers.sp += 1;
                self.registers.pc = unsigned_16(msb, lsb);
                self.ime = true; // todo dispatch on next cycle
                Some(4)
            }
            0xE9 => {
                // JP HL
                self.registers.pc = self.get_hl();
                Some(1)
            }

            0xC6 => {
                // ADD
                let left: u8 = self.registers.a;
                let right: u8 = self.memory.read(self.registers.pc);
                self.registers.pc += 1;
                self.registers.a += right;
                self.set_flag_z(self.registers.a == 0);
                self.set_flag_n(false);
                self.set_flag_h(((left & 0xF) + (right & 0xF)) > 0xF);
                self.set_flag_c(((left as u16) + (right as u16)) > 0xFF);
                Some(2)
            }
            0xCE => {
                // ADC
                let c_save: u8 = self.get_flag_c();
                let left: u8 = self.registers.a;
                let right: u8 = self.memory.read(self.registers.pc);
                self.registers.pc += 1;
                self.registers.a += right + c_save;
                self.set_flag_z(self.registers.a == 0);
                self.set_flag_n(false);
                self.set_flag_h(((left & 0xF) + (right & 0xF) + c_save) > 0xF);
                self.set_flag_c(((left as u16) + (right as u16) + (c_save as u16)) > 0xFF);
                Some(2)
            }
            0xD6 => {
                // SUB
                let left: u8 = self.registers.a;
                let right: u8 = self.memory.read(self.registers.pc);
                self.registers.pc += 1;
                self.registers.a -= right;
                self.set_flag_z(self.registers.a == 0);
                self.set_flag_n(true);
                self.set_flag_h(((left & 0xF) - (right & 0xF)) > 0xF);
                self.set_flag_c(((left as u16) - (right as u16)) > 0xFF);
                Some(1)
            }
            0xDE => {
                // SBC
                let c_save: u8 = self.get_flag_c();
                let left: u8 = self.registers.a;
                let right: u8 = self.memory.read(self.registers.pc);
                self.registers.pc += 1;
                self.registers.a = left - right - c_save;
                self.set_flag_z(self.registers.a == 0);
                self.set_flag_n(true);
                self.set_flag_h(((left & 0xF) - (right & 0xF) - c_save) > 0xF);
                self.set_flag_c(((left as u16) - (right as u16) - (c_save as u16)) > 0xFF);
                Some(1)
            }
            0xE6 => {
                // AND
                self.registers.a &= self.memory.read(self.registers.pc);
                self.registers.pc += 1;
                self.set_flag_z(self.registers.a == 0);
                self.set_flag_n(false);
                self.set_flag_h(true);
                self.set_flag_c(false);
                Some(2)
            }
            0xEE => {
                // XOR
                self.registers.a ^= self.memory.read(self.registers.pc);
                self.registers.pc += 1;
                self.set_flag_z(self.registers.a == 0);
                self.set_flag_n(false);
                self.set_flag_h(false);
                self.set_flag_c(false);
                Some(2)
            }
            0xF6 => {
                // OR
                self.registers.a |= self.memory.read(self.registers.pc);
                self.registers.pc += 1;
                self.set_flag_z(self.registers.a == 0);
                self.set_flag_n(false);
                self.set_flag_h(false);
                self.set_flag_c(false);
                Some(2)
            }
            0xFE => {
                // CP
                let left: u8 = self.registers.a;
                let right: u8 = self.memory.read(self.registers.pc);
                self.registers.pc += 1;
                let result: u8 = left - right;
                self.set_flag_z(result == 0);
                self.set_flag_n(true);
                self.set_flag_h(((left & 0xF) - (right & 0xF)) > 0xF);
                self.set_flag_c(((left as u16) - (right as u16)) > 0xFF);
                Some(1)
            }
            _ => {
                let r16 = (opcode >> 4) & 0b11;
                if (opcode & 0b11_00_1111) == 0b00_00_0001 {
                    // LD r16 u16
                    let lsb = self.memory.read(self.registers.pc);
                    self.registers.pc += 1;
                    let msb = self.memory.read(self.registers.pc);
                    self.registers.pc += 1;
                    self.set_r16_group_1(r16, unsigned_16(msb, lsb));
                    return Some(3);
                } else if (opcode & 0b11_00_1111) == 0b00_00_0011 {
                    // INC r16
                    let r16_value = self.get_r16_group_1(r16);
                    self.set_r16_group_1(r16, r16_value + 1);
                    return Some(1);
                } else if (opcode & 0b11_00_1111) == 0b00_00_1011 {
                    // DEC r16
                    let r16_value = self.get_r16_group_1(r16);
                    self.set_r16_group_1(r16, r16_value - 1);
                    return Some(1);
                } else if (opcode & 0b11_00_1111) == 0b00_00_1001 {
                    // Add HL r16
                    let left = self.get_hl();
                    let right = self.get_r16_group_1(r16);
                    let result = left + right;
                    self.set_hl(result);
                    self.set_flag_n(false);
                    self.set_flag_h((left & 0xFFF) + (right & 0xFFF) > 0xFFF);
                    self.set_flag_c(left as u32 + right as u32 > 0xFFFF);
                    return Some(2);
                } else if (opcode & 0b11_00_1111) == 0b00_00_0010 {
                    // LD (r16), A
                    let r16_value = self.get_r16_group_2(r16);
                    self.memory.write(r16_value, self.registers.a);
                    return Some(2);
                } else if (opcode & 0b11_00_1111) == 0b00_00_1010 {
                    // LD A, (r16)
                    let r16_value: u16 = self.get_r16_group_2(r16);
                    self.registers.a = self.memory.read(r16_value);
                    return Some(2);
                }

                if (opcode & 0b11_000_111) == 0b00_000_110 {
                    // LD r8, u8
                    let r8 = opcode >> 3;
                    self.set_r8(r8, self.memory.read(self.registers.pc));
                    self.registers.pc += 1;
                    return Some(2);
                } else if (opcode & 0b11_000_111) == 0b00_000_100 {
                    // INC r8
                    let r8 = opcode >> 3;
                    let r8_old = self.get_r8(r8);
                    let result = self.get_r8(r8) + 1;
                    self.set_r8(r8, result);
                    self.set_flag_z(result == 0);
                    self.set_flag_n(false);
                    self.set_flag_h((r8_old & 0xF) + 1 > 0xF);
                    return Some(1);
                } else if (opcode & 0b11_000_111) == 0b00_000_101 {
                    // DEC r8
                    let r8 = opcode >> 3;
                    let r8_old = self.get_r8(r8);
                    let result = self.get_r8(r8) - 1;
                    self.set_r8(r8, result);
                    self.set_flag_z(result == 0);
                    self.set_flag_n(true);
                    self.set_flag_h((r8_old & 0xF) - 1 > 0xF);
                    return Some(1);
                }

                if (opcode >> 5) == 0b001 {
                    // JR conditional
                    let condition;
                    match (opcode >> 3) & 0b11 {
                        0 => condition = self.get_flag_z() == 0,
                        1 => condition = self.get_flag_z() != 0,
                        2 => condition = self.get_flag_c() == 0,
                        3 => condition = self.get_flag_c() != 0,
                        _ => panic!("not possible condition - JR conditional"),
                    }
                    let e = self.memory.read(self.registers.pc) as i8;
                    self.registers.pc += 1;
                    if condition {
                        self.registers.pc = (self.registers.pc as i16 + e as i16) as u16;
                        return Some(3);
                    } else {
                        return Some(2);
                    }
                }

                if (opcode >> 6) == 0b01 {
                    let r8_source: u8 = opcode & 0b111;
                    let r8_dest: u8 = (opcode >> 3) & 0b111;
                    self.set_r8(r8_dest, self.get_r8(r8_source));
                    return Some(1);
                }

                if (opcode >> 6) == 0b10 {
                    if (opcode >> 3) & 0b111 == 0 {
                        // ADD
                        let r8: u8 = opcode & 0b111;
                        let left: u8 = self.registers.a;
                        let right: u8 = self.get_r8(r8);
                        self.registers.a += right;
                        self.set_flag_z(self.registers.a == 0);
                        self.set_flag_n(false);
                        self.set_flag_h(((left & 0xF) + (right & 0xF)) > 0xF);
                        self.set_flag_c(((left as u16) + (right as u16)) > 0xFF);
                        return Some(1);
                    } else if (opcode >> 3) & 0b111 == 1 {
                        // ADC
                        let r8: u8 = opcode & 0b111;
                        let c_save: u8 = self.get_flag_c();
                        let left: u8 = self.registers.a;
                        let right: u8 = self.get_r8(r8);
                        self.registers.a += right + c_save;
                        self.set_flag_z(self.registers.a == 0);
                        self.set_flag_n(false);
                        self.set_flag_h(((left & 0xF) + (right & 0xF) + c_save) > 0xF);
                        self.set_flag_c(((left as u16) + (right as u16) + (c_save as u16)) > 0xFF);
                        return Some(1);
                    } else if (opcode >> 3) & 0b111 == 2 {
                        // SUB A, r8
                        let r8: u8 = opcode & 0b111;
                        let left: u8 = self.registers.a;
                        let right: u8 = self.get_r8(r8);
                        self.registers.a -= right;
                        self.set_flag_z(self.registers.a == 0);
                        self.set_flag_n(true);
                        self.set_flag_h(((left & 0xF) - (right & 0xF)) > 0xF);
                        self.set_flag_c(((left as u16) - (right as u16)) > 0xFF);
                        return Some(1);
                    } else if (opcode >> 3) & 0b111 == 3 {
                        // SBC
                        let r8: u8 = opcode & 0b111;
                        let c_save: u8 = self.get_flag_c();
                        let left: u8 = self.registers.a;
                        let right: u8 = self.get_r8(r8);
                        self.registers.a = left - right - c_save;
                        self.set_flag_z(self.registers.a == 0);
                        self.set_flag_n(true);
                        self.set_flag_h(((left & 0xF) - (right & 0xF) - c_save) > 0xF);
                        self.set_flag_c(((left as u16) - (right as u16) - (c_save as u16)) > 0xFF);
                        return Some(1);
                    } else if (opcode >> 3) & 0b111 == 4 {
                        // AND r
                        self.registers.a &= self.get_r8(opcode & 0b111);
                        self.set_flag_z(self.registers.a == 0);
                        self.set_flag_n(false);
                        self.set_flag_h(true);
                        self.set_flag_c(false);
                        return Some(1);
                    } else if (opcode >> 3) & 0b111 == 5 {
                        // XOR r
                        self.registers.a ^= self.get_r8(opcode & 0b111);
                        self.set_flag_z(self.registers.a == 0);
                        self.set_flag_n(false);
                        self.set_flag_h(false);
                        self.set_flag_c(false);
                        return Some(1);
                    } else if (opcode >> 3) & 0b111 == 6 {
                        // OR r
                        self.registers.a |= self.get_r8(opcode & 0b111);
                        self.set_flag_z(self.registers.a == 0);
                        self.set_flag_n(false);
                        self.set_flag_h(false);
                        self.set_flag_c(false);
                        return Some(1);
                    } else if (opcode >> 3) & 0b111 == 7 {
                        let r8: u8 = opcode & 0b111;
                        let left: u8 = self.registers.a;
                        let right: u8 = self.get_r8(r8);
                        let result = left - right;
                        self.set_flag_z(result == 0);
                        self.set_flag_n(true);
                        self.set_flag_h(((left & 0xF) - (right & 0xF)) > 0xF);
                        self.set_flag_c(((left as u16) - (right as u16)) > 0xFF);
                        return Some(1);
                    }
                }

                if (opcode >> 6) == 0b11 {
                    match opcode & 0b1111 {
                        0b0001 => {
                            // POP r16
                            let r16 = (opcode >> 4) & 0b11;
                            let mask;
                            if r16 == 3 {
                                mask = 0xF0
                            } else {
                                mask = 0xFF
                            }
                            let lsb = self.memory.read(self.registers.sp) & mask;
                            self.registers.sp += 1;
                            let msb = self.memory.read(self.registers.sp);
                            self.registers.sp += 1;
                            self.set_r16_group_3(r16, unsigned_16(msb, lsb));
                            return Some(3);
                        }
                        0b0101 => {
                            // PUSH r16
                            let r16 = (opcode >> 4) & 0b11;
                            let r16_value = self.get_r16_group_3(r16);
                            let mask;
                            if r16 == 3 {
                                mask = 0xF0
                            } else {
                                mask = 0xFF
                            }
                            self.registers.sp -= 1;
                            self.memory.write(self.registers.sp, msb(r16_value));
                            self.registers.sp -= 1;
                            self.memory.write(self.registers.sp, lsb(r16_value) & mask);
                            return Some(3);
                        }
                        _ => (),
                    }
                }

                if (opcode & 0b111_00_111) == 0b110_00_000 {
                    // RET conditional
                    let condition;
                    match (opcode >> 3) & 0b11 {
                        0 => condition = self.get_flag_z() == 0,
                        1 => condition = self.get_flag_z() != 0,
                        2 => condition = self.get_flag_c() == 0,
                        3 => condition = self.get_flag_c() != 0,
                        _ => panic!("not possible condition - RET conditional"),
                    }
                    if condition {
                        let lsb = self.memory.read(self.registers.sp);
                        self.registers.sp += 1;
                        let msb = self.memory.read(self.registers.sp);
                        self.registers.sp += 1;
                        self.registers.pc = unsigned_16(msb, lsb);
                        return Some(5);
                    } else {
                        return Some(2);
                    }
                } else if (opcode & 0b111_00_111) == 0b110_00_100 {
                    let ls_byte = self.memory.read(self.registers.pc);
                    self.registers.pc += 1;
                    let ms_byte = self.memory.read(self.registers.pc);
                    self.registers.pc += 1;
                    let condition;
                    match (opcode >> 3) & 0b11 {
                        0 => condition = self.get_flag_z() == 0,
                        1 => condition = self.get_flag_z() != 0,
                        2 => condition = self.get_flag_c() == 0,
                        3 => condition = self.get_flag_c() != 0,
                        _ => {
                            panic!("not possible condition - CALL conditional")
                        }
                    }
                    if condition {
                        self.registers.sp -= 1;
                        self.memory.write(self.registers.sp, msb(self.registers.pc));
                        self.registers.sp -= 1;
                        self.memory.write(self.registers.sp, lsb(self.registers.pc));
                        self.registers.pc = unsigned_16(ms_byte, ls_byte);
                        return Some(6);
                    } else {
                        return Some(3);
                    }
                }

                if (opcode & 0b111_00_111) == 0b110_00_010 {
                    let condition;
                    match (opcode >> 3) & 0b11 {
                        // JP conditional
                        0 => condition = self.get_flag_z() == 0,
                        1 => condition = self.get_flag_z() != 0,
                        2 => condition = self.get_flag_c() == 0,
                        3 => condition = self.get_flag_c() != 0,
                        _ => panic!("not possible condition - JP conditional"),
                    }
                    let lsb = self.memory.read(self.registers.pc);
                    self.registers.pc += 1;
                    let msb = self.memory.read(self.registers.pc);
                    self.registers.pc += 1;
                    if condition {
                        // JP conditional, u16
                        self.registers.pc = unsigned_16(msb, lsb);
                        return Some(4);
                    } else {
                        return Some(3);
                    }
                } else if (opcode & 0b11_000_111) == 0b11_000_111 {
                    // RST
                    let exp = opcode & 0b00_111_000;
                    self.registers.sp -= 1;
                    self.memory.write(self.registers.sp, msb(self.registers.pc));
                    self.registers.sp -= 1;
                    self.memory.write(self.registers.sp, lsb(self.registers.pc));
                    self.registers.pc = unsigned_16(0x00, exp);
                    return Some(4);
                }

                if (opcode & 0b11_000_111) == 0b11_000_011 {
                    if (opcode >> 3) & 0b111 == 0 {
                        // JP unconditional, u16
                        let lsb = self.memory.read(self.registers.pc);
                        self.registers.pc += 1;
                        let msb = self.memory.read(self.registers.pc);
                        self.registers.pc += 1;
                        self.registers.pc = unsigned_16(msb, lsb);
                        return Some(4);
                    } else {
                        return None;
                    }
                }

                return None;
            }
        }
    }
}
