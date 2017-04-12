use super::*;

impl SdHandle {
    /// Send CMD0, which resets all cards into the idle state
    pub fn cmd_go_idle_state(&mut self) -> low_level::SdmmcErrorCode {
        self.registers.arg.update(|arg| arg.set_cmdarg(0)); // only stuff bits in argument
        let mut sent = self.registers.sta.read();
        println!("Cmdsent register: {:?}. ", sent);
        
        self.registers.cmd.update(|cmd| {
            // ensure reset values in unused bits
            cmd.set_sdiosuspend(false);
            cmd.set_waitpend(false);
            cmd.set_waitint(false);
            cmd.set_waitresp(WaitResp::No as u8);
            // set card to send CMD0
            cmd.set_cpsmen(true);
            cmd.set_cmdindex(0);
        });
        print!("Tried sending CMD0. ");
        sent = self.registers.sta.read();
        println!("Cmdsent register: {:?}", sent);
        
        self.get_cmd_error()
    }

    /// Sends CMD8, which enquires the card's operating parameters. The command is only supported by version 2 cards and it is mandatory to
    /// send it, when the host controller supports version 2.
    /// Returns low_level::None if version 2 is supported and a different Error code if not.
    pub fn cmd_send_if_cond(&mut self) -> low_level::SdmmcErrorCode {
        // Argument:
        // - [31:12]: Reserved (shall be set to '0')
        // - [11:8]: Supply Voltage (VHS) 0x1 (Range: 2.7-3.6 V)
        // - [7:0]: Check Pattern (recommended 0xAA)
        self.registers.arg.update(|arg| arg.set_cmdarg(0x1AA));
        
        self.registers.cmd.update(|cmd| {
            // ensure reset values in unused bits
            cmd.set_sdiosuspend(false);
            cmd.set_waitpend(false);
            cmd.set_waitint(false);
            // set card to send CMD8
            cmd.set_waitresp(WaitResp::Short as u8);
            cmd.set_cpsmen(true);
            cmd.set_cmdindex(8);
        });

        self.get_response7()
    }

    pub fn cmd_app_cmd(&mut self, rca: u32) -> low_level::SdmmcErrorCode {
        // Argument:
        // - [31:16]: RCA
        // - [15:0]: stuff bits
        self.registers.arg.update(|arg| arg.set_cmdarg(rca << 16));
        
        self.registers.cmd.update(|cmd| {
            // ensure reset values in unused bits
            cmd.set_sdiosuspend(false);
            cmd.set_waitpend(false);
            cmd.set_waitint(false);
            // set card to send CMD8
            cmd.set_waitresp(WaitResp::Short as u8);
            cmd.set_cpsmen(true);
            cmd.set_cmdindex(55);
        });

        self.get_response1(55, 0)
    }

    pub fn cmd_sd_send_op_cond(&mut self, capacity: CardCapacity) -> low_level::SdmmcErrorCode {
        // Argument:
        // - [31]: reserved
        // - [30]: HCS = OCR[30]
        // - [29]: reserved
        // - [28]: XPC
        // - [27:25]: reserved
        // - [24]: S18R
        // - [23:0]: V_dd Voltage Window = OCR[23:0]
        self.registers.arg.update(|arg| arg.set_cmdarg(0x8010_0000 | capacity as u32));
        
        self.registers.cmd.update(|cmd| {
            // ensure reset values in unused bits
            cmd.set_sdiosuspend(false);
            cmd.set_waitpend(false);
            cmd.set_waitint(false);
            // set card to send CMD8
            cmd.set_waitresp(WaitResp::Short as u8);
            cmd.set_cpsmen(true);
            cmd.set_cmdindex(41);
        });

        self.get_response3()
    }

    /// Checks whether any errors occurred while sending the previous command. The command must not expect
    /// any response.
    // TODO: Very similiar to get_response7, zusammenführen?
    // represents SDMMC_GetCmdError
    fn get_cmd_error(&self) -> low_level::SdmmcErrorCode {
        // Wait for 5000 milliseconds to receive confirmation that command was sent successfully.
        // The value 5000 is taken from SDMMC_CMDTIMEOUT in the C code.
        let timeout = ::system_clock::ticks() + 5000;
        while ::system_clock::ticks() < timeout {
            if self.registers.sta.read().cmdsent() {return low_level::NONE};
        }
        low_level::TIMEOUT
    }

    /// Checks the R1 response and waits for maximum timeout milliseconds to receive
    /// the response.
    fn get_response1(&mut self, cmd_index: u8, timeout: usize) -> low_level::SdmmcErrorCode {
        print!("Reading Response 1 after sending a command: ");
        while ::system_clock::ticks() < timeout {
            if self.registers.sta.read().ctimeout() {
                // Command timeout
                print!("Command timeout. ");
                self.registers.icr.update(|icr| icr.set_ctimeoutc(true));
                return low_level::CMD_RSP_TIMEOUT;
            }
            if self.registers.sta.read().ccrcfail() {
                // CRC failed
                print!("Command received, but CRC failed. ");
                self.registers.icr.update(|icr| icr.set_ccrcfailc(true));
                return low_level::CMD_CRC_FAIL;
            }
            if self.registers.sta.read().cmdrend() {
                // command received correctly
                print!("Command received correctly. ");
                
                // check whether received response matches the command
                if self.registers.respcmd.read().respcmd() != cmd_index {
                    return low_level::CMD_CRC_FAIL;
                }
                self.clear_all_static_status_flags();

                let response = self.registers.resp1.read().cardstatus1();
                return check_ocr_error_bits(response);
            }
        }
        print!("Software timeout. ");
        low_level::TIMEOUT
    }

    /// Checks the R3 response and waits for maximum timeout milliseconds to receive
    /// the response.
    fn get_response3(&mut self) -> low_level::SdmmcErrorCode {
        print!("Reading Response 3 after sending a command: ");
        let timeout = ::system_clock::ticks() + 5000;
        while ::system_clock::ticks() < timeout {
            if self.registers.sta.read().ctimeout() {
                // Command timeout
                print!("Command timeout. ");
                self.registers.icr.update(|icr| icr.set_ctimeoutc(true));
                return low_level::CMD_RSP_TIMEOUT;
            }
            if self.registers.sta.read().ccrcfail() || self.registers.sta.read().cmdrend() {
                // CRC failed
                print!("Command received. ");
                self.clear_all_static_status_flags();
                return low_level::NONE;
            }
        }
        print!("Software timeout. ");
        low_level::TIMEOUT
    }

    /// Tests whether response 7 can be received. If it can, the card supports version 2.0
    /// and SdmmcErrorCode::None is returned. If version 2.0 is not supported an error is returned.
    // represents SDMMC_GetCmdResp7
    fn get_response7(&mut self) -> low_level::SdmmcErrorCode {
        print!("After sending CMD8: ");
        let timeout = ::system_clock::ticks() + 5000;
        while ::system_clock::ticks() < timeout {
            if self.registers.sta.read().ctimeout() {
                // Command timeout, version 2 not supported.
                print!("Command timeout. ");
                self.registers.icr.update(|icr| icr.set_ctimeoutc(true));
                return low_level::CMD_RSP_TIMEOUT;
            }
            if self.registers.sta.read().ccrcfail() {
                // version 2 supported
                print!("Command received, but CRC failed. ");
                self.registers.icr.update(|icr| icr.set_ccrcfailc(true));
                return low_level::NONE;
            }
            if self.registers.sta.read().cmdrend() {
                // version 2 supported
                print!("Command received correctly. ");
                self.registers.icr.update(|icr| icr.set_cmdrendc(true));
                return low_level::NONE;
            }
        }
        print!("Software timeout. ");
        low_level::TIMEOUT
    }
}

fn check_ocr_error_bits(resp1: u32) -> low_level::SdmmcErrorCode {
    if (resp1 & 0xFDFFE008) == 0 {
        low_level::NONE
    } else if (resp1 & 0x8000_0000) == 0x8000_0000 {
        low_level::ADDR_OUT_OF_RANGE
    } else if (resp1 & 0x4000_0000) == 0x4000_0000 {
        low_level::ADDR_MISALIGNED
    } else if (resp1 & 0x2000_0000) == 0x2000_0000 {
        low_level::BLOCK_LEN_ERR
    } else if (resp1 & 0x1000_0000) == 0x1000_0000 {
        low_level::ERASE_SEQ_ERR
    } else if (resp1 & 0x0800_0000) == 0x0800_0000 {
        low_level::BAD_ERASE_PARAM
    } else if (resp1 & 0x0400_0000) == 0x0400_0000 {
        low_level::WRITE_PROT_VIOLATION
    } else if (resp1 & 0x0100_0000) == 0x0100_0000 {
        low_level::LOCK_UNLOCK_FAILED
    } else if (resp1 & 0x0080_0000) == 0x0080_0000 {
        low_level::COM_CRC_FAILED
    } else if (resp1 & 0x0040_0000) == 0x0040_0000 {
        low_level::ILLEGAL_CMD
    } else if (resp1 & 0x0020_0000) == 0x0020_0000 {
        low_level::CARD_ECC_FAILED
    } else if (resp1 & 0x0010_0000) == 0x0010_0000 {
        low_level::CC_ERR
    } else if (resp1 & 0x0004_0000) == 0x0004_0000 {
        low_level::STREAM_READ_UNDERRUN
    } else if (resp1 & 0x0002_0000) == 0x0002_0000 {
        low_level::STREAM_WRITE_OVERRUN
    } else if (resp1 & 0x0001_0000) == 0x0001_0000 {
        low_level::CID_CSD_OVERWRITE
    } else if (resp1 & 0x0000_8000) == 0x0000_8000 {
        low_level::WP_ERASE_SKIP
    } else if (resp1 & 0x0000_4000) == 0x0000_4000 {
        low_level::CARD_ECC_DISABLED
    } else if (resp1 & 0x0000_2000) == 0x0000_2000 {
        low_level::ERASE_RESET
    } else if (resp1 & 0x0000_0008) == 0x0000_0008 {
        low_level::AKE_SEQ_ERR
    } else {
        low_level::GENERAL_UNKNOWN_ERR
    }
}