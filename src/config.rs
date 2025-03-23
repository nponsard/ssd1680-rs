#[derive(Clone, Copy)]
pub enum LUTSelect {
    LUT0 = 0x00,
    LUT1 = 0x01,
    LUT2 = 0x02,
    LUT3 = 0x03,
}
impl From<LUTSelect> for u8 {
    fn from(val: LUTSelect) -> Self {
        val as u8
    }
}
impl LUTSelect {
    pub fn into_u8(&self) -> u8 {
        *self as u8
    }
}

/// In border waveform control, bit 4 and 5
#[derive(Clone, Copy)]
pub enum VDBLevel {
    VSS = 0x00,
    VSH1 = 0x10,
    VSL = 0x20,
    VSH2 = 0x30,
}
impl From<VDBLevel> for u8 {
    fn from(val: VDBLevel) -> Self {
        val as u8
    }
}
impl VDBLevel {
    pub fn into_u8(&self) -> u8 {
        *self as u8
    }
}

#[derive(Clone, Copy)]
pub enum VDBMode {
    /// GSTransistion( followLut (false is "output VCOM @ RED"), lut_select )
    GSTransition(bool, LUTSelect),
    FixLevel(VDBLevel),
    VCOM,
    HiZ,
}

#[derive(Clone, Copy)]
pub enum UpdateRamOption {
    Normal = 0x0,
    /// Bypass RAM content as 0
    Bypass0 = 0x4,
    /// Inverse RAM content
    Inverse = 0x8,
}
impl From<UpdateRamOption> for u8 {
    fn from(val: UpdateRamOption) -> Self {
        val as u8
    }
}

#[derive(Clone, Copy)]
pub struct DisplayConfig {
    pub width: u16,
    pub height: u16,

    // gate scanning sequence :
    pub gate_scanning_gd: bool,
    pub gate_scanning_sm: bool,
    pub gate_scanning_tb: bool,

    // which sequence to use to refresh (command 0x22)
    pub partial_refresh_sequence: u8,
    pub full_refresh_sequence: u8,

    pub border_waveform_control: VDBMode,

    // display update control 1
    pub ram_content_for_display_update: UpdateRamOption,
    pub s8_source_output_mode: bool,

    pub use_internal_temperature_sensor: bool,
    // TODO: handle rotation
}

/// Sensible defaults using the full ram
impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            width: 176,
            height: 296,
            gate_scanning_gd: false,
            gate_scanning_sm: false,
            gate_scanning_tb: false,
            partial_refresh_sequence: 0xFC,
            full_refresh_sequence: 0xF7,
            border_waveform_control: VDBMode::GSTransition(true, LUTSelect::LUT1),
            ram_content_for_display_update: UpdateRamOption::Normal,
            s8_source_output_mode: true,
            use_internal_temperature_sensor: true,
        }
    }
}

impl DisplayConfig {
    /// Default config for EPD 2.9" T94
    pub fn epd_290_t94() -> Self {
        DisplayConfig::default().with_width(128).with_height(296)
    }

    pub fn with_width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    pub fn with_height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    pub fn with_gate_scanning(mut self, gd: bool, sm: bool, tb: bool) -> Self {
        self.gate_scanning_gd = gd;
        self.gate_scanning_sm = sm;
        self.gate_scanning_tb = tb;
        self
    }

    pub fn with_border_waveform_control(mut self, mode: VDBMode) -> Self {
        self.border_waveform_control = mode;
        self
    }

    pub fn with_partial_refresh_sequence(mut self, sequence: u8) -> Self {
        self.partial_refresh_sequence = sequence;
        self
    }

    pub fn with_full_refresh_sequence(mut self, sequence: u8) -> Self {
        self.full_refresh_sequence = sequence;
        self
    }

    pub fn with_ram_content_for_display_update(
        mut self,
        option: UpdateRamOption,
        s8_source_output_mode: bool,
    ) -> Self {
        self.ram_content_for_display_update = option;
        self.s8_source_output_mode = s8_source_output_mode;
        self
    }

    pub fn with_use_internal_temperature_sensor(mut self, internal: bool) -> Self {
        self.use_internal_temperature_sensor = internal;
        self
    }
}
