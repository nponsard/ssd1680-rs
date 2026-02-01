use super::{
    commands::SsdCommand,
    config::{DisplayConfig, UpdateRamOption, VDBMode},
    error::Error,
};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::{delay::DelayNs, digital::Wait, spi::SpiDevice};

/// Struct representing the connexion with the SSD1680 display driver.
pub struct SSD1680<
    RST: OutputPin,
    DC: OutputPin,
    BUSY: InputPin + Wait,
    DELAY: DelayNs,
    SPI: SpiDevice,
> {
    rst_pin: RST,
    dc: DC,
    busy: BUSY,
    spi: SPI,
    delay: DELAY,
    config: DisplayConfig,
}

impl<
    RST: OutputPin,
    DC: OutputPin,
    BUSY: InputPin + Wait,
    DELAY: DelayNs,
    SPI: SpiDevice,
    S,
    R,
    D,
    B,
> SSD1680<RST, DC, BUSY, DELAY, SPI>
where
    SPI: SpiDevice<Error = S>,
    RST: OutputPin<Error = R>,
    DC: OutputPin<Error = D>,
    BUSY: InputPin<Error = B>,
{
    pub fn new(
        rst_pin: RST,
        dc: DC,
        busy: BUSY,
        delay: DELAY,
        spi: SPI,
        config: DisplayConfig,
    ) -> Self {
        SSD1680 {
            rst_pin,
            dc,
            busy,
            spi,
            delay,
            config,
        }
    }

    /// Write a byte to the black/white RAM.
    pub async fn write_bw_byte(&mut self, byte: u8) -> Result<(), Error<S, R, D, B>> {
        self.write_command(SsdCommand::WriteRamBW).await?;
        self.write_data(&[byte]).await?;
        Ok(())
    }

    /// Write bytes to the black/white RAM.
    pub async fn write_bw_bytes(&mut self, bytes: &[u8]) -> Result<(), Error<S, R, D, B>> {
        self.write_command(SsdCommand::WriteRamBW).await?;
        self.write_data(bytes).await?;
        Ok(())
    }

    /// Write a byte to the red RAM.
    pub async fn write_red_byte(&mut self, byte: u8) -> Result<(), Error<S, R, D, B>> {
        self.write_command(SsdCommand::WriteRamRed).await?;
        self.write_data(&[byte]).await?;
        Ok(())
    }

    /// Write bytes to the red RAM.
    pub async fn write_red_bytes(&mut self, bytes: &[u8]) -> Result<(), Error<S, R, D, B>> {
        self.write_command(SsdCommand::WriteRamRed).await?;
        self.write_data(bytes).await?;
        Ok(())
    }

    /// Initialize the hardware according to the configuration.
    /// This function is to be used to initialize the hardware the first time, and to reinitialize it after putting it into deep sleep.
    pub async fn hw_init(&mut self) -> Result<(), Error<S, R, D, B>> {
        self.rst_pin.set_low().map_err(Error::RstPinError)?;
        self.delay.delay_ms(20).await;
        self.rst_pin.set_high().map_err(Error::RstPinError)?;
        self.delay.delay_ms(20).await;

        self.wait_for_busy().await?;
        self.sw_reset().await?;
        self.wait_for_busy().await?;

        self.output_control(
            self.config.height,
            self.config.gate_scanning_gd,
            self.config.gate_scanning_sm,
            self.config.gate_scanning_tb,
        )
        .await?;

        self.data_entry_mode(true, true, false).await?;

        self.set_ram_start_end_x(0, self.config.width / 8 - 1)
            .await?;
        self.set_ram_start_end_y(0, self.config.height - 1).await?;

        self.set_border_waveform(self.config.border_waveform_control)
            .await?;

        self.display_update_control_1(
            self.config.ram_content_for_display_update,
            self.config.ram_content_for_display_update,
            self.config.s8_source_output_mode,
        )
        .await?;

        self.select_internal_temperature_sensor(self.config.use_internal_temperature_sensor)
            .await?;

        self.set_ram_counter_x(0).await?;
        self.set_ram_counter_y(0).await?;
        self.wait_for_busy().await?;

        Ok(())
    }

    /// Set the ram counter x position, this represents the data bank (8bits), so you need to divide by 8
    pub async fn set_ram_counter_x(&mut self, x: u16) -> Result<(), Error<S, R, D, B>> {
        self.write_command(SsdCommand::SetRamXCounter).await?;
        self.write_data(&[x as u8]).await?;
        Ok(())
    }

    /// Set the ram counter y position.
    pub async fn set_ram_counter_y(&mut self, y: u16) -> Result<(), Error<S, R, D, B>> {
        self.write_command(SsdCommand::SetRamYCounter).await?;
        self.write_data(&[y as u8, (y >> 8) as u8]).await?;
        Ok(())
    }

    /// Use the internal temperature sensor
    pub async fn select_internal_temperature_sensor(
        &mut self,
        internal: bool,
    ) -> Result<(), Error<S, R, D, B>> {
        self.write_command(SsdCommand::TemperatureSensorControl)
            .await?;
        self.write_data(&[if internal { 0x80 } else { 0x48 }])
            .await?;
        Ok(())
    }

    /// Set the border waveform mode.
    pub async fn set_border_waveform(&mut self, mode: VDBMode) -> Result<(), Error<S, R, D, B>> {
        let data = match mode {
            VDBMode::VCOM => 0x80,
            VDBMode::HiZ => 0xC0,
            VDBMode::FixLevel(level) => 0x10 | level.into_u8(),
            VDBMode::GSTransition(follow_lut, lut) => ((follow_lut as u8) << 2) | lut.into_u8(),
        };
        self.write_command(SsdCommand::BorderWaveformnControl)
            .await?;
        self.write_data(&[data]).await?;
        Ok(())
    }

    /// red : red ram data
    /// bw : black and white ram data
    /// source_output_mode : source output mode true is "Available Source from S8 to S167", false is "Available Source from S0 to S175"
    pub async fn display_update_control_1(
        &mut self,
        red: UpdateRamOption,
        bw: UpdateRamOption,
        source_output_mode: bool,
    ) -> Result<(), Error<S, R, D, B>> {
        let first_byte: u8 = bw as u8 | ((red as u8) << 4);
        let second_byte = (source_output_mode as u8) << 7;

        self.write_command(SsdCommand::DisplayUpdateControl1)
            .await?;
        self.write_data(&[first_byte, second_byte]).await?;
        Ok(())
    }

    pub async fn set_ram_start_end_x(
        &mut self,
        start: u16,
        end: u16,
    ) -> Result<(), Error<S, R, D, B>> {
        self.write_command(SsdCommand::SetRamXStartEnd).await?;
        self.write_data(&[start as u8, end as u8]).await?;
        Ok(())
    }
    pub async fn set_ram_start_end_y(
        &mut self,
        start: u16,
        end: u16,
    ) -> Result<(), Error<S, R, D, B>> {
        self.write_command(SsdCommand::SetRamYStartEnd).await?;
        self.write_data(&[start as u8, (start >> 8) as u8, end as u8, (end >> 8) as u8])
            .await?;
        Ok(())
    }

    /// Write LUT Register, 153 bytes long
    pub async fn write_lut_register(
        &mut self,
        register: &[u8; 153],
    ) -> Result<(), Error<S, R, D, B>> {
        self.write_command(SsdCommand::WriteLutRegister).await?;
        self.write_data(register).await?;
        Ok(())
    }

    pub async fn output_control(
        &mut self,
        height: u16,
        gd: bool,
        sm: bool,
        tb: bool,
    ) -> Result<(), Error<S, R, D, B>> {
        let height = if height == 0 { 0 } else { height - 1 };
        self.write_command(SsdCommand::DriveOutputControl).await?;
        let gate_scanning: u8 = tb as u8 | ((sm as u8) << 1) | ((gd as u8) << 2);
        self.write_data(&[height as u8, (height >> 8) as u8, gate_scanning])
            .await?;
        Ok(())
    }

    /// For x and y, true means increment/false means decrement
    /// For direction, true means the address counter is updated in the Y direction after data has been written to the RAM, false will update the X counter.
    pub async fn data_entry_mode(
        &mut self,
        x: bool,
        y: bool,
        direction: bool,
    ) -> Result<(), Error<S, R, D, B>> {
        let sequence: u8 = (x as u8) | ((y as u8) << 1) | ((direction as u8) << 2);
        self.write_command(SsdCommand::DataEntryModeSetting).await?;
        self.write_data(&[sequence]).await?;

        Ok(())
    }

    /// Send the command to soft reset the chip (used by hw_init)
    pub async fn sw_reset(&mut self) -> Result<(), Error<S, R, D, B>> {
        self.write_command(SsdCommand::SWReset).await
    }

    /// Wait until the busy pin is low, meaning the chip is ready to receive new commands or data.
    pub async fn wait_for_busy(&mut self) -> Result<(), Error<S, R, D, B>> {
        self.busy
            .wait_for_low()
            .await
            .map_err(Error::BusyPinError)?;

        Ok(())
    }

    /// Send a command to the chip.
    pub async fn write_command(&mut self, command: SsdCommand) -> Result<(), Error<S, R, D, B>> {
        self.wait_for_busy().await?;
        self.dc.set_low().map_err(Error::DcPinError)?;
        self.spi
            .write(&[command.into()])
            .await
            .map_err(Error::SpiError)?;
        // self.wait_for_busy()
        Ok(())
    }

    /// Send data to the chip
    pub async fn write_data(&mut self, data: &[u8]) -> Result<(), Error<S, R, D, B>> {
        self.dc.set_high().map_err(Error::DcPinError)?;
        self.spi.write(data).await.map_err(Error::SpiError)?;
        // self.wait_for_busy()
        Ok(())
    }

    /// Set how the display should be updated
    pub async fn display_update_control_2(
        &mut self,
        /*TODO: make an enum */ sequence: u8,
    ) -> Result<(), Error<S, R, D, B>> {
        self.write_command(SsdCommand::DisplayUpdateControl2)
            .await?;
        self.write_data(&[sequence]).await?;
        Ok(())
    }

    /// Run the update sequence, the chip will send the correct frequencies to the display to reflect the image in memory
    pub async fn activate_update(&mut self) -> Result<(), Error<S, R, D, B>> {
        self.write_command(SsdCommand::MasterActivation).await?;
        Ok(())
    }

    /// Refresh screen using a custom sequence
    ///
    /// On the 290_T94 screen, 0xF7 is the full refresh sequence and 0xFC is the partial refresh sequence.
    pub async fn refresh_screen_custom_sequence(
        &mut self,
        sequence: u8,
    ) -> Result<(), Error<S, R, D, B>> {
        self.display_update_control_2(sequence).await?;
        // self.delay.delay_ms(20);
        self.activate_update().await?;
        self.wait_for_busy().await?;
        Ok(())
    }

    /// Partial refresh using the configured sequence
    pub async fn partial_refresh(&mut self) -> Result<(), Error<S, R, D, B>> {
        self.refresh_screen_custom_sequence(self.config.partial_refresh_sequence)
            .await
    }

    /// Full refresh using the configured sequence
    pub async fn full_refresh(&mut self) -> Result<(), Error<S, R, D, B>> {
        self.refresh_screen_custom_sequence(self.config.full_refresh_sequence)
            .await
    }

    /// Let the ssd1680 fill its ram with a single color
    pub async fn fill_bw_screen_internal(&mut self, color: bool) -> Result<(), Error<S, R, D, B>> {
        self.wait_for_busy().await?;

        self.write_command(SsdCommand::AutoWriteBWRam).await?;
        // TODO: different height and width, here its 296x128
        let data = ((color as u8) << 7) | 0x60 | 0x04;
        self.write_data(&[data]).await?;
        self.wait_for_busy().await?;
        Ok(())
    }

    /// Manually fill the memory with a single color
    pub async fn fill_bw_screen(&mut self, color: bool) -> Result<(), Error<S, R, D, B>> {
        self.wait_for_busy().await?;

        self.write_command(SsdCommand::WriteRamBW).await?;
        for _ in 0..(self.config.width * self.config.height / 8) {
            self.write_data(&[(color as u8) * 255]).await?;
        }
        self.write_command(SsdCommand::Nop).await?;
        self.wait_for_busy().await?;

        Ok(())
    }

    /// Enter deep sleep mode, it is recommended to enter deep sleep after drawing to the screen.
    /// The datasheet indicates that keeping the chip running can deteriorate the display faster.
    pub async fn enter_deep_sleep(&mut self) -> Result<(), Error<S, R, D, B>> {
        self.write_command(SsdCommand::DeepSleepMode).await?;
        self.write_data(&[0x01]).await?;
        Ok(())
    }

    /// Read from the chip's RAM
    pub async fn read_ram(&mut self) -> Result<u8, Error<S, R, D, B>> {
        self.wait_for_busy().await?;

        self.write_command(SsdCommand::ReadRam).await?;
        // from the documentation : "first byte is dummy data"
        let mut buf = [0u8; 2];
        self.spi.read(&mut buf).await.map_err(Error::SpiError)?;

        Ok(buf[1])
    }
}
