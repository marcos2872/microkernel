//! Este módulo implementa a escrita para o buffer de texto do modo VGA.
//! Ele fornece um `Writer` global que pode ser usado para imprimir na tela.

use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;

/// Desabilita o cursor de hardware do VGA.
///
/// O cursor piscando pode ser irritante em QEMU. Esta função desabilita
/// o cursor escrevendo nos registradores de controle do VGA.
pub fn disable_cursor() {
    unsafe {
        let mut port_3d4 = Port::new(0x3D4);
        let mut port_3d5 = Port::new(0x3D5);

        port_3d4.write(0x0A as u8);
        let val: u8 = port_3d5.read();
        port_3d5.write(val | 0x20);
    }
}

/// Enum para as cores padrão do modo de texto VGA.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// Representa um código de cor completo, incluindo cor de primeiro plano e de fundo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    /// Cria um novo `ColorCode` com a cor de primeiro plano e de fundo especificadas.
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// Representa um caractere na tela, com seu caractere ASCII e código de cor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// A altura do buffer de texto (geralmente 25).
const BUFFER_HEIGHT: usize = 25;
/// A largura do buffer de texto (geralmente 80).
const BUFFER_WIDTH: usize = 80;

/// Representa o buffer de texto VGA.
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// Um `Writer` que permite escrever no buffer de texto VGA.
///
/// Mantém o controle da posição atual do cursor e da cor do texto.
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Escreve um único byte ASCII na tela.
    ///
    /// Caracteres de nova linha (`\n`) são tratados especialmente.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// Escreve a string fornecida na tela.
    ///
    /// Caracteres que não são ASCII imprimíveis (na faixa de 0x20 a 0x7e)
    /// são impressos como `■`.
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Move o cursor para uma nova linha, rolando a tela se necessário.
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// Limpa uma linha, preenchendo-a com espaços em branco.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    /// Limpa a tela inteira.
    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    /// O `Writer` global que pode ser usado para imprimir na tela.
    ///
    /// É protegido por um `Mutex` para garantir que seja seguro para threads.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

/// Macro para imprimir uma string formatada na tela.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

/// Macro para imprimir uma string formatada na tela, com uma nova linha no final.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Função auxiliar privada usada pelas macros `print!` e `println!`.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

/// Macro para limpar a tela.
#[macro_export]
macro_rules! clear_screen {
    () => ($crate::vga_buffer::_clear_screen());
}

/// Função auxiliar privada usada pela macro `clear_screen!`.
#[doc(hidden)]
pub fn _clear_screen() {
    WRITER.lock().clear_screen();
}
