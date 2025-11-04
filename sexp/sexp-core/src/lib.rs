use std::io::{self, Write};

mod impls;

pub type Result<T> = io::Result<T>;

pub trait ToSexp {
    fn to_sexp<S: SexpSerializer>(&self, serializer: &mut S) -> Result<()>;
}

pub trait SexpSerializer {
    fn serialize_bool(&mut self, value: bool) -> Result<()>;

    fn serialize_i8(&mut self, value: i8) -> Result<()>;

    fn serialize_i16(&mut self, value: i16) -> Result<()>;

    fn serialize_i32(&mut self, value: i32) -> Result<()>;

    fn serialize_i64(&mut self, value: i64) -> Result<()>;

    fn serialize_i128(&mut self, value: i128) -> Result<()>;

    fn serialize_u8(&mut self, value: u8) -> Result<()>;

    fn serialize_u16(&mut self, value: u16) -> Result<()>;

    fn serialize_u32(&mut self, value: u32) -> Result<()>;

    fn serialize_u64(&mut self, value: u64) -> Result<()>;

    fn serialize_u128(&mut self, value: u128) -> Result<()>;

    fn serialize_f32(&mut self, value: f32) -> Result<()>;

    fn serialize_f64(&mut self, value: f64) -> Result<()>;

    fn serialize_char(&mut self, value: char) -> Result<()>;

    fn serialize_str(&mut self, value: &str) -> Result<()>;

    fn serialize_none(&mut self) -> Result<()>;

    fn serialize_some<T>(&mut self, value: &T) -> Result<()>
    where
        T: ToSexp;

    fn serialize_unit(&mut self) -> Result<()>;

    fn serialize_unit_struct(&mut self, name: &'static str) -> Result<()>;

    fn serialize_unit_variant(
        &mut self,
        name: &'static str,
        idx: u32,
        variant: &'static str,
    ) -> Result<()>;

    fn serialize_newtype_struct<T>(&mut self, name: &'static str, value: &T) -> Result<()>
    where
        T: ToSexp;

    fn serialize_newtype_variant<T>(
        &mut self,
        name: &'static str,
        idx: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ToSexp;

    fn serialize_tuple_struct(&mut self, name: &'static str, len: usize) -> Result<()>;

    fn serialize_tuple_variant(
        &mut self,
        name: &'static str,
        idx: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<()>;

    fn serialize_struct(&mut self, name: &'static str) -> Result<()>;

    fn serialize_struct_variant(&mut self, variant: &'static str) -> Result<()>;

    fn serialize_tuple_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ToSexp;

    fn serialize_struct_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ToSexp;

    fn begin_cell(&mut self) -> Result<()>;

    fn end_cell(&mut self) -> Result<()>;

    fn begin_value(&mut self) -> Result<()>;
}

pub struct SexpSerializerImpl<W> {
    writer: W,
    indent_depth: usize,
    has_value: bool,
}

impl<W> SexpSerializerImpl<W>
where
    W: io::Write,
{
    pub fn new(writer: W) -> Self {
        SexpSerializerImpl {
            writer,
            indent_depth: 0,
            has_value: false,
        }
    }
}

impl<W> SexpSerializer for SexpSerializerImpl<W>
where
    W: io::Write,
{
    fn serialize_bool(&mut self, value: bool) -> Result<()> {
        let s = if value {
            b"true" as &[u8]
        } else {
            b"false" as &[u8]
        };
        self.writer.write_all(s)
    }

    fn serialize_u8(&mut self, value: u8) -> Result<()> {
        let mut buf = itoa::Buffer::new();
        let s = buf.format(value);
        self.writer.write_all(s.as_bytes())
    }

    fn serialize_u16(&mut self, value: u16) -> Result<()> {
        let mut buf = itoa::Buffer::new();
        let s = buf.format(value);
        self.writer.write_all(s.as_bytes())
    }

    fn serialize_u32(&mut self, value: u32) -> Result<()> {
        let mut buf = itoa::Buffer::new();
        let s = buf.format(value);
        self.writer.write_all(s.as_bytes())
    }

    fn serialize_u64(&mut self, value: u64) -> Result<()> {
        let mut buf = itoa::Buffer::new();
        let s = buf.format(value);
        self.writer.write_all(s.as_bytes())
    }

    fn serialize_u128(&mut self, value: u128) -> Result<()> {
        let mut buf = itoa::Buffer::new();
        let s = buf.format(value);
        self.writer.write_all(s.as_bytes())
    }

    fn serialize_i8(&mut self, value: i8) -> Result<()> {
        let mut buf = itoa::Buffer::new();
        let s = buf.format(value);
        self.writer.write_all(s.as_bytes())
    }

    fn serialize_i16(&mut self, value: i16) -> Result<()> {
        let mut buf = itoa::Buffer::new();
        let s = buf.format(value);
        self.writer.write_all(s.as_bytes())
    }

    fn serialize_i32(&mut self, value: i32) -> Result<()> {
        let mut buf = itoa::Buffer::new();
        let s = buf.format(value);
        self.writer.write_all(s.as_bytes())
    }

    fn serialize_i64(&mut self, value: i64) -> Result<()> {
        let mut buf = itoa::Buffer::new();
        let s = buf.format(value);
        self.writer.write_all(s.as_bytes())
    }

    fn serialize_i128(&mut self, value: i128) -> Result<()> {
        let mut buf = itoa::Buffer::new();
        let s = buf.format(value);
        self.writer.write_all(s.as_bytes())
    }

    fn serialize_f32(&mut self, value: f32) -> Result<()> {
        let mut buf = ryu::Buffer::new();
        let s = buf.format_finite(value);
        self.writer.write_all(s.as_bytes())
    }

    fn serialize_f64(&mut self, value: f64) -> Result<()> {
        let mut buf = ryu::Buffer::new();
        let s = buf.format_finite(value);
        self.writer.write_all(s.as_bytes())
    }

    fn serialize_char(&mut self, value: char) -> Result<()> {
        let mut buf = [0_u8; 4];
        self.serialize_str(value.encode_utf8(&mut buf))
    }

    fn serialize_str(&mut self, value: &str) -> Result<()> {
        format_escaped_str(&mut self.writer, value)
    }

    fn serialize_none(&mut self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(&mut self, value: &T) -> Result<()>
    where
        T: ToSexp,
    {
        value.to_sexp(self)
    }

    fn serialize_unit(&mut self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(&mut self, name: &'static str) -> Result<()> {
        write_string_fragment(&mut self.writer, name)
    }

    fn serialize_unit_variant(
        &mut self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<()> {
        write_string_fragment(&mut self.writer, variant)
    }

    fn serialize_newtype_struct<T>(&mut self, name: &'static str, value: &T) -> Result<()>
    where
        T: ToSexp,
    {
        todo!()
    }

    fn serialize_newtype_variant<T>(
        &mut self,
        name: &'static str,
        idx: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ToSexp,
    {
        todo!()
    }

    fn serialize_tuple_struct(&mut self, name: &'static str, len: usize) -> Result<()> {
        todo!()
    }

    fn serialize_tuple_variant(
        &mut self,
        name: &'static str,
        idx: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<()> {
        self.begin_cell()?;
        write_string_fragment(&mut self.writer, variant)
    }

    fn serialize_struct(&mut self, name: &'static str) -> Result<()> {
        self.begin_cell()?;
        write_string_fragment(&mut self.writer, name)
    }

    fn serialize_struct_variant(
        &mut self,
        // name: &'static str,
        // idx: u32,
        variant: &'static str,
        // len: usize,
    ) -> Result<()> {
        self.begin_cell()?;
        write_string_fragment(&mut self.writer, variant)
    }

    fn serialize_tuple_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ToSexp,
    {
        self.has_value = true;
        value.to_sexp(self)
    }

    fn serialize_struct_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ToSexp,
    {
        self.has_value = true;
        {
            self.begin_cell()?;
            write_string_fragment(&mut self.writer, key)?;
            self.begin_value()?;
            value.to_sexp(self)?;
            self.end_cell()
        }
    }

    fn begin_cell(&mut self) -> Result<()> {
        if self.has_value {
            self.writer.write_all(b"\n")?;
            indent(&mut self.writer, self.indent_depth)?;
        }
        self.indent_depth += 1;
        self.writer.write_all(b"(")
    }

    fn end_cell(&mut self) -> Result<()> {
        self.indent_depth -= 1;
        self.writer.write_all(b")")
    }

    fn begin_value(&mut self) -> Result<()> {
        self.writer.write_all(b": ")
    }
}

fn format_escaped_str<W>(writer: &mut W, value: &str) -> Result<()>
where
    W: io::Write,
{
    writer.write_all(b"\"")?;
    format_escaped_str_content(writer, value)?;
    writer.write_all(b"\"")
}

fn format_escaped_str_content<W>(writer: &mut W, value: &str) -> Result<()>
where
    W: io::Write,
{
    for byte in value.as_bytes() {
        let (char, escape) = match byte {
            b'\x08' => (b'b', true),
            b'\t' => (b't', true),
            b'\n' => (b'n', true),
            b'\x0c' => (b'f', true),
            b'\r' => (b'r', true),
            b'"' => (b'\"', true),
            b => (*b, false),
        };

        if escape {
            writer.write_all(&[b'\\'])?;
        }
        writer.write_all(&[char])?;
    }
    Ok(())
}

fn write_string_fragment<W>(writer: &mut W, fragment: &str) -> Result<()>
where
    W: io::Write,
{
    writer.write_all(fragment.as_bytes())
}

fn indent<W>(writer: &mut W, n: usize) -> Result<()>
where
    W: io::Write,
{
    for _ in 0..n {
        writer.write_all(b"  ")?;
    }
    Ok(())
}
