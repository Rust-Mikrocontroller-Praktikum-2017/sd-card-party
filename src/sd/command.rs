use super::*;

impl SdHandle {
    /// Send CMD0, which resets all cards into the idle state
    pub fn cmd_go_idle_state(&mut self) -> low_level::SdmmcErrorCode {
        self.registers.arg.update(|arg| arg.set_cmdarg(0)); // only stuff bits in argument
        
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
                return low_level::CMD_RSP_TIMEOUT;
            }
            if self.registers.sta.read().ccrcfail() {
                // version 2 supported
                print!("Command received, but CRC failed. ");
                return low_level::NONE;
            }
            if self.registers.sta.read().cmdrend() {
                // version 2 supported
                print!("Command received correctly. ");
                return low_level::NONE;
            }
        }
        print!("Software timeout. ");
        low_level::TIMEOUT
    }
}