use std::io::{self};

mod impls;

pub type Result<T> = io::Result<T>;

pub trait ToSexp {
    fn to_sexp<S: SexpSerializer>(&self, s: &mut S) -> Result<()>;
}

pub trait SexpSerializer {
    fn serialize_bool(&mut self, value: bool) -> Result<()>;

    fn serialize_num<I>(&mut self, value: I) -> Result<()>
    where
        I: itoa::Integer;

    fn serialize_float<F>(&mut self, value: F) -> Result<()>
    where
        F: ryu::Float;

    fn serialize_char(&mut self, value: char) -> Result<()>;

    fn serialize_str(&mut self, value: &str) -> Result<()>;

    fn serialize_none(&mut self) -> Result<()>;

    fn serialize_some<T>(&mut self, value: &T) -> Result<()>
    where
        T: ToSexp;

    fn serialize_unit(&mut self) -> Result<()>;

    fn serialize_tuple_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ToSexp;

    fn serialize_struct_field<T>(&mut self, field: &'static str, value: &T) -> Result<()>
    where
        T: ToSexp;

    fn cell<F>(&mut self, name: &'static str, f: F) -> Result<()>
    where
        F: FnOnce(&mut Self) -> Result<()>;

    fn begin_cell(&mut self, name: &'static str) -> Result<()>;

    fn end_cell(&mut self) -> Result<()>;

    fn begin_value(&mut self) -> Result<()>;

    fn end_value(&mut self) -> Result<()>;
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

    fn serialize_num<I>(&mut self, value: I) -> Result<()>
    where
        I: itoa::Integer,
    {
        let mut buf = itoa::Buffer::new();
        let s = buf.format(value);
        self.writer.write_all(s.as_bytes())
    }

    fn serialize_float<F>(&mut self, value: F) -> Result<()>
    where
        F: ryu::Float,
    {
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

    fn serialize_tuple_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ToSexp,
    {
        self.has_value = true;
        value.to_sexp(self)
    }

    fn serialize_struct_field<T>(&mut self, field: &'static str, value: &T) -> Result<()>
    where
        T: ToSexp,
    {
        self.has_value = true;

        self.cell(field, |s| {
            s.begin_value()?;
            value.to_sexp(s)?;
            s.end_value()
        })
    }

    fn cell<F>(&mut self, name: &'static str, f: F) -> Result<()>
    where
        F: FnOnce(&mut Self) -> Result<()>,
    {
        self.begin_cell(name)?;
        f(self)?;
        self.end_cell()
    }

    fn begin_cell(&mut self, name: &'static str) -> Result<()> {
        if self.has_value {
            self.writer.write_all(b"\n")?;
            indent(&mut self.writer, self.indent_depth)?;
        }
        self.indent_depth += 1;

        self.writer.write_all(b"(")?;
        write_string_fragment(&mut self.writer, name)
    }

    fn end_cell(&mut self) -> Result<()> {
        self.indent_depth -= 1;
        self.writer.write_all(b")")
    }

    fn begin_value(&mut self) -> Result<()> {
        self.writer.write_all(b": ")
    }

    fn end_value(&mut self) -> Result<()> {
        Ok(())
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
    let mut last = 0;
    let bytes = value.as_bytes();

    for (i, &b) in bytes.iter().enumerate() {
        let escaped = match b {
            b'\x08' => Some(b'b'),
            b'\t' => Some(b't'),
            b'\n' => Some(b'n'),
            b'\x0c' => Some(b'f'),
            b'\r' => Some(b'r'),
            b'"' => Some(b'\"'),
            _ => None,
        };

        if let Some(ch) = escaped {
            if last < i {
                writer.write_all(&bytes[last..i])?;
            }
            writer.write_all(&[b'\\', ch])?;
            last = i + 1;
        }
    }
    if last < bytes.len() {
        writer.write_all(&bytes[last..])?;
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
