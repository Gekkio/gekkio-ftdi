pub extern crate gekkio_ftdi_sys as sys;

use bitflags::bitflags;
use std::borrow::BorrowMut;
use std::error::Error;
use std::ffi::CStr;
use std::fmt;
use std::mem;
use std::os::raw::c_int;
use std::str;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FtdiError {
    UsbDeviceUnavailable,
    Other(i32, &'static str),
}

impl fmt::Display for FtdiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FtdiError::UsbDeviceUnavailable => write!(f, "USB device unavailable"),
            FtdiError::Other(code, msg) => write!(f, "libftdi1 error code {}: {}", code, msg),
        }
    }
}

impl Error for FtdiError {}

bitflags! {
    #[repr(transparent)]
    pub struct ModemStatus: u16 {
        /// Error in receiver FIFO
        const RCVR_ERR = 0b1000_0000_0000_0000;
        /// Transmitter empty
        const TEMT = 0b0100_0000_0000_0000;
        /// Transmitter holding register
        const THRE = 0b0010_0000_0000_0000;
        /// Break interrupt
        const BI = 0b0001_0000_0000_0000;
        /// Framing error
        const FE = 0b0000_1000_0000_0000;
        /// Parity error
        const PE = 0b0000_0100_0000_0000;
        /// Overrun error
        const OE = 0b0000_0010_0000_0000;
        /// Data ready
        const DR = 0b0000_0001_0000_0000;
        /// Data Carrier Detect (DCD)
        const DCD = 0b0000_0000_1000_0000;
        /// Ring Indicator (RI)
        const RI = 0b0000_0000_0100_0000;
        /// Data Set Ready (DSR)
        const DSR = 0b0000_0000_0010_0000;
        /// Clear To Send (CTS)
        const CTS = 0b0000_0000_0001_0000;
    }
}

#[repr(u32)]
pub enum FlowControl {
    None = 0x000,
    RtsCts = 0x100,
    DtsDsr = 0x200,
    XonXoff = 0x400,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Interface {
    A = sys::ftdi_interface_INTERFACE_A,
    B = sys::ftdi_interface_INTERFACE_B,
    C = sys::ftdi_interface_INTERFACE_C,
    D = sys::ftdi_interface_INTERFACE_D,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BitMode {
    Reset = sys::ftdi_mpsse_mode_BITMODE_RESET,
    BitBang = sys::ftdi_mpsse_mode_BITMODE_BITBANG,
    Mpsse = sys::ftdi_mpsse_mode_BITMODE_MPSSE,
    SyncBitBang = sys::ftdi_mpsse_mode_BITMODE_SYNCBB,
    Mcu = sys::ftdi_mpsse_mode_BITMODE_MCU,
    Opto = sys::ftdi_mpsse_mode_BITMODE_OPTO,
    Cbus = sys::ftdi_mpsse_mode_BITMODE_CBUS,
    SyncFf = sys::ftdi_mpsse_mode_BITMODE_SYNCFF,
    Ft1284 = sys::ftdi_mpsse_mode_BITMODE_FT1284,
}

fn error_msg(ctx: *mut sys::ftdi_context) -> &'static str {
    unsafe {
        let msg = sys::ftdi_get_error_string(ctx);
        if msg.is_null() {
            ""
        } else {
            str::from_utf8_unchecked(CStr::from_ptr(msg).to_bytes())
        }
    }
}

pub struct Context(Box<sys::ftdi_context>);

impl Context {
    /// Creates and initializes a new FTDI context
    pub fn new() -> Result<Context, FtdiError> {
        unsafe {
            let mut ctx: Box<sys::ftdi_context> = Box::new(mem::zeroed());
            // Bug: libftdi1 leaks memory if libusb initialization fails
            match sys::ftdi_init(ctx.borrow_mut()) {
                code if code < 0 => Err(FtdiError::Other(code, error_msg(ctx.borrow_mut()))),
                _ => Ok(Context(ctx)),
            }
        }
    }
    /// Selects the used chip interface
    pub fn set_interface(&mut self, interface: Interface) -> Result<(), FtdiError> {
        match unsafe { sys::ftdi_set_interface(self.0.borrow_mut(), interface as u32) } {
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(()),
        }
    }
    pub fn raw_mut(&mut self) -> *mut sys::ftdi_context {
        self.0.borrow_mut()
    }
}

impl Context {
    /// Opens the first FTDI device that has the given vendor and product id
    pub fn usb_open(&mut self, vendor: u16, product: u16) -> Result<(), FtdiError> {
        match unsafe { sys::ftdi_usb_open(self.raw_mut(), vendor as c_int, product as c_int) } {
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(()),
        }
    }
    /// Resets the FTDI device
    pub fn usb_reset(&mut self) -> Result<(), FtdiError> {
        match unsafe { sys::ftdi_usb_reset(self.0.borrow_mut()) } {
            -2 => Err(FtdiError::UsbDeviceUnavailable),
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(()),
        }
    }
    /// Clears the read buffer on the chip and the internal read buffer
    pub fn usb_purge_rx_buffer(&mut self) -> Result<(), FtdiError> {
        match unsafe { sys::ftdi_usb_purge_rx_buffer(self.0.borrow_mut()) } {
            -2 => Err(FtdiError::UsbDeviceUnavailable),
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(()),
        }
    }
    /// Clears the write buffer on the chip
    pub fn usb_purge_tx_buffer(&mut self) -> Result<(), FtdiError> {
        match unsafe { sys::ftdi_usb_purge_tx_buffer(self.0.borrow_mut()) } {
            -2 => Err(FtdiError::UsbDeviceUnavailable),
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(()),
        }
    }
    /// Clears the buffers on the chip and the internal read buffer
    pub fn usb_purge_buffers(&mut self) -> Result<(), FtdiError> {
        match unsafe { sys::ftdi_usb_purge_buffers(self.0.borrow_mut()) } {
            -3 => Err(FtdiError::UsbDeviceUnavailable),
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(()),
        }
    }
    /// Closes the FTDI device
    pub fn usb_close(&mut self) -> Result<(), FtdiError> {
        match unsafe { sys::ftdi_usb_close(self.0.borrow_mut()) } {
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(()),
        }
    }
}

impl Context {
    /// Reads the FTDIChip-ID from R-type devices
    pub fn read_chip_id(&mut self) -> Result<u32, FtdiError> {
        let mut result = 0;
        match unsafe { sys::ftdi_read_chipid(self.0.borrow_mut(), &mut result) } {
            -2 => Err(FtdiError::UsbDeviceUnavailable),
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(result),
        }
    }
    /// Gets the latency timer value (in milliseconds)
    pub fn get_latency_timer(&mut self) -> Result<u8, FtdiError> {
        let mut result = 0;
        match unsafe { sys::ftdi_get_latency_timer(self.0.borrow_mut(), &mut result) } {
            -2 => Err(FtdiError::UsbDeviceUnavailable),
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(result),
        }
    }
    /// Sets the latency timer value (in milliseconds)
    pub fn set_latency_timer(&mut self, millis: u8) -> Result<(), FtdiError> {
        match unsafe { sys::ftdi_set_latency_timer(self.0.borrow_mut(), millis) } {
            -3 => Err(FtdiError::UsbDeviceUnavailable),
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(()),
        }
    }
    /// Enable/disable bitbang modes
    pub fn set_bit_mode(&mut self, mask: u8, bit_mode: BitMode) -> Result<(), FtdiError> {
        match unsafe { sys::ftdi_set_bitmode(self.0.borrow_mut(), mask, bit_mode as u8) } {
            -2 => Err(FtdiError::UsbDeviceUnavailable),
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(()),
        }
    }
    /// Disable bitbang mode.
    ///
    /// Equivalent to `set_bit_mode(0, BitMode::Reset)`.
    pub fn disable_bit_bang(&mut self) -> Result<(), FtdiError> {
        match unsafe { sys::ftdi_disable_bitbang(self.0.borrow_mut()) } {
            -2 => Err(FtdiError::UsbDeviceUnavailable),
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(()),
        }
    }
    /// Directly read pin state, circumventing the read buffer
    pub fn read_pins(&mut self) -> Result<u8, FtdiError> {
        let mut result = 0;
        match unsafe { sys::ftdi_read_pins(self.0.borrow_mut(), &mut result) } {
            -2 => Err(FtdiError::UsbDeviceUnavailable),
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(result),
        }
    }
    /// Poll modem status information
    pub fn poll_modem_status(&mut self) -> Result<ModemStatus, FtdiError> {
        let mut result = 0;
        match unsafe { sys::ftdi_poll_modem_status(self.0.borrow_mut(), &mut result) } {
            -2 => Err(FtdiError::UsbDeviceUnavailable),
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(ModemStatus::from_bits_truncate(result)),
        }
    }
    /// Sets both the Data Terminal Ready (DTR) and Request To Send (RTS) signals
    pub fn set_dtr_rts(&mut self, dtr: bool, rts: bool) -> Result<(), FtdiError> {
        match unsafe { sys::ftdi_setdtr_rts(self.0.borrow_mut(), dtr as _, rts as _) } {
            -2 => Err(FtdiError::UsbDeviceUnavailable),
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(()),
        }
    }
    /// Sets the Data Terminal Ready (DTR) signal
    pub fn set_dtr(&mut self, dtr: bool) -> Result<(), FtdiError> {
        match unsafe { sys::ftdi_setdtr(self.0.borrow_mut(), dtr as _) } {
            -2 => Err(FtdiError::UsbDeviceUnavailable),
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(()),
        }
    }
    /// Sets the Request To Send (RTS) signal
    pub fn set_rts(&mut self, rts: bool) -> Result<(), FtdiError> {
        match unsafe { sys::ftdi_setrts(self.0.borrow_mut(), rts as _) } {
            -2 => Err(FtdiError::UsbDeviceUnavailable),
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(()),
        }
    }
    /// Sets the flow control setting
    pub fn set_flow_control(&mut self, flow_control: FlowControl) -> Result<(), FtdiError> {
        match unsafe { sys::ftdi_setflowctrl(self.0.borrow_mut(), flow_control as i32) } {
            -2 => Err(FtdiError::UsbDeviceUnavailable),
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(()),
        }
    }
    /// Sets and enables/disables the special event character
    pub fn set_event_char(&mut self, ch: u8, enable: bool) -> Result<(), FtdiError> {
        match unsafe { sys::ftdi_set_event_char(self.0.borrow_mut(), ch, enable as _) } {
            -2 => Err(FtdiError::UsbDeviceUnavailable),
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(()),
        }
    }
    /// Sets and enables/disables the error character
    pub fn set_error_char(&mut self, ch: u8, enable: bool) -> Result<(), FtdiError> {
        match unsafe { sys::ftdi_set_error_char(self.0.borrow_mut(), ch, enable as _) } {
            -2 => Err(FtdiError::UsbDeviceUnavailable),
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(()),
        }
    }
    pub fn write_data(&mut self, data: &[u8]) -> Result<(), FtdiError> {
        match unsafe { sys::ftdi_write_data(self.0.borrow_mut(), data.as_ptr(), data.len() as _) } {
            -666 => Err(FtdiError::UsbDeviceUnavailable),
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            _ => Ok(()),
        }
    }
    pub fn read_data(&mut self, buf: &mut [u8]) -> Result<usize, FtdiError> {
        match unsafe { sys::ftdi_read_data(self.0.borrow_mut(), buf.as_mut_ptr(), buf.len() as _) }
        {
            -666 => Err(FtdiError::UsbDeviceUnavailable),
            code if code < 0 => Err(FtdiError::Other(code, error_msg(self.raw_mut()))),
            len => Ok(len as usize),
        }
    }
    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), FtdiError> {
        let mut pos = 0;
        while pos < buf.len() {
            let len = self.read_data(&mut buf[pos..])?;
            pos += len;
        }
        Ok(())
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            sys::ftdi_deinit(self.0.borrow_mut());
        }
    }
}
