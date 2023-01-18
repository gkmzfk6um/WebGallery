mod error;
pub mod ser 
{
    use serde::{ser, Serialize};

    use crate::error::{Error, Result};

    pub struct Serializer {
        output: Vec<(String,String)>,
        path: String
    }

    pub struct SeqSerializer<'a> {
        serializer: &'a mut Serializer,
        index: usize

    }
    pub struct MapSerializer<'a> {
        serializer: &'a mut Serializer,
        last_path: Option<String>

    }

    pub fn to_string<T>(value: &T) -> Result<String>
    where T: Serialize
    {
        let res = to_form_pairs(value)?;
        serde_urlencoded::to_string(res).map_err(|e| Error::from(e) )
    }

    pub fn to_form_pairs<T>(value: &T) -> Result<Vec<(String,String)>>
    where T: Serialize
    {
        let mut serializer = Serializer {
            output: Vec::new(),
            path: "".into()
        };
        value.serialize(&mut serializer)?;

        return Ok(serializer.output)
    }
    
    impl <'a> SeqSerializer<'a>
    {
        fn new(serializer: &'a mut Serializer) -> SeqSerializer<'a>
        {
            SeqSerializer::<'a> {
                serializer,
                index: 0
            }
        }
    }

    impl <'a> MapSerializer<'a>
    {
        fn new(serializer: &'a mut Serializer) -> MapSerializer<'a>
        {
            MapSerializer::<'a> {
                serializer,
                last_path: None
            }
        }
    }
    
    impl<'a> ser::SerializeMap for  MapSerializer<'a> {
        type Ok = ();
        type Error = Error;

        fn serialize_key<T>(&mut self, key: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            match &self.last_path 
            {
                None => { 
                    self.last_path = Some(self.serializer.path.clone());
                    let mut child_serializer = Serializer {
                        output: Vec::new(),
                        path: String::from("")
                    };
                    key.serialize(&mut child_serializer)?;

                    if child_serializer.output.len() != 1
                    {
                        return Err(Error::new(self.last_path.clone(), "Key serializer returned more than one output".into()))
                    }
                    let (key_path,key_val) = child_serializer.output.pop().unwrap();
                    if key_path != ""
                    {
                        return Err(Error::new(self.last_path.clone(), "Key serializer returned non empty path".into()))
                    }
                    if  self.last_path.as_ref().unwrap() == ""
                    {
                        self.serializer.path = format!("{}",key_val);

                    }
                    else {
                        self.serializer.path = format!("{}[{}]",self.serializer.path,key_val);
                    }
                    Ok(())
                },
                Some(_) => {
                        return Err(Error::new(self.last_path.clone(), "serialize_key called with last_path = Some".into()))
                }
            }
        }

        // It doesn't make a difference whether the colon is printed at the end of
        // `serialize_key` or at the beginning of `serialize_value`. In this case
        // the code is a bit simpler having it here.
        fn serialize_value<T>(&mut self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            
            value.serialize(&mut *self.serializer)?;
            let mut tmp = None;
            std::mem::swap(&mut tmp, &mut self.last_path);
            self.serializer.path =tmp.unwrap();
            Ok(())
        }

        fn end(self) -> Result<()> {
            Ok(())
        }
    }



    
    impl<'a> ser::SerializeSeq for  SeqSerializer<'a> {
        // Must match the `Ok` type of the serializer.
        type Ok = ();
        // Must match the `Error` type of the serializer.
        type Error = Error;

        // Serialize a single element of the sequence.
        fn serialize_element<T>(&mut self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize
        {

            let path = self.serializer.path.clone();
            self.serializer.path = format!("{}[{}]",path,self.index);
            value.serialize(&mut *self.serializer)?;
            self.serializer.path = path;
            self.index += 1;
            Ok(())
        }

        // Close the sequence.
        fn end(self) -> Result<()> {
            Ok(())
        }
    }
    impl<'a> ser::SerializeTuple for SeqSerializer<'a> {
        type Ok = ();
        type Error = Error;

        fn serialize_element<T>(&mut self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            <Self as ser::SerializeSeq>::serialize_element::<T>(self,value)
        }

        fn end(self) -> Result<()> {
            Ok(())
        }
    }
    // Same thing but for tuple structs.
    impl<'a> ser::SerializeTupleStruct for SeqSerializer<'a> {
        type Ok = ();
        type Error = Error;

        fn serialize_field<T>(&mut self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            <Self as ser::SerializeSeq>::serialize_element::<T>(self,value)
        }

        fn end(self) -> Result<()> {
            Ok(())
        }
    }

    impl<'a> ser::SerializeTupleVariant for SeqSerializer<'a> {
        type Ok = ();
        type Error = Error;

        fn serialize_field<T>(&mut self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            <Self as ser::SerializeSeq>::serialize_element::<T>(self,value)
        }

        fn end(self) -> Result<()> {
            Ok(())
        }
    }

    impl<'a> ser::Serializer for &'a mut Serializer {
        // The output type produced by this `Serializer` during successful
        // serialization. Most serializers that produce text or binary output should
        // set `Ok = ()` and serialize into an `io::Write` or buffer contained
        // within the `Serializer` instance, as happens here. Serializers that build
        // in-memory data structures may be simplified by using `Ok` to propagate
        // the data structure around.
        type Ok = ();

        // The error type when some error occurs during serialization.
        type Error = Error;

        // Associated types for keeping track of additional state while serializing
        // compound data structures like sequences and maps. In this case no
        // additional state is required beyond what is already stored in the
        // Serializer struct.
        type SerializeSeq = SeqSerializer<'a>;
        type SerializeTuple = SeqSerializer<'a>;
        type SerializeTupleStruct = SeqSerializer<'a>;
        type SerializeTupleVariant = SeqSerializer<'a>;
        type SerializeMap = MapSerializer<'a>;
        type SerializeStruct = Self;
        type SerializeStructVariant = Self;

        // Here we go with the simple methods. The following 12 methods receive one
        // of the primitive types of the data model and map it to JSON by appending
        // into the output string.
        fn serialize_bool(self, v: bool) -> Result<()> {
            let val =  if v { "true" } else { "false" };
            self.output.push( (self.path.clone(),val.into()) );
            Ok(())
        }

        // JSON does not distinguish between different sizes of integers, so all
        // signed integers will be serialized the same and all unsigned integers
        // will be serialized the same. Other formats, especially compact binary
        // formats, may need independent logic for the different sizes.
        fn serialize_i8(self, v: i8) -> Result<()> {
            self.serialize_i64(i64::from(v))
        }

        fn serialize_i16(self, v: i16) -> Result<()> {
            self.serialize_i64(i64::from(v))
        }

        fn serialize_i32(self, v: i32) -> Result<()> {
            self.serialize_i64(i64::from(v))
        }

        // Not particularly efficient but this is example code anyway. A more
        // performant approach would be to use the `itoa` crate.
        fn serialize_i64(self, v: i64) -> Result<()> {
            self.output.push( (self.path.clone(), v.to_string()) );
            Ok(())
        }

        fn serialize_u8(self, v: u8) -> Result<()> {
            self.serialize_u64(u64::from(v))
        }

        fn serialize_u16(self, v: u16) -> Result<()> {
            self.serialize_u64(u64::from(v))
        }

        fn serialize_u32(self, v: u32) -> Result<()> {
            self.serialize_u64(u64::from(v))
        }

        fn serialize_u64(self, v: u64) -> Result<()> {
            self.output.push( (self.path.clone(), v.to_string()) );
            Ok(())
        }

        fn serialize_f32(self, v: f32) -> Result<()> {
            self.serialize_f64(f64::from(v))
        }

        fn serialize_f64(self, v: f64) -> Result<()> {
            self.output.push( (self.path.clone(), v.to_string()) );
            Ok(())
        }

        // Serialize a char as a single-character string. Other formats may
        // represent this differently.
        fn serialize_char(self, v: char) -> Result<()> {
            self.output.push( (self.path.clone(), v.to_string()) );
            Ok(())
        }

        // This only works for strings that don't require escape sequences but you
        // get the idea. For example it would emit invalid JSON if the input string
        // contains a '"' character.
        fn serialize_str(self, v: &str) -> Result<()> {
            self.output.push( (self.path.clone(), v.to_string()) );
            Ok(())
        }

        // Serialize a byte array as an array of bytes. Could also use a base64
        // string here. Binary formats will typically represent byte arrays more
        // compactly.
        fn serialize_bytes(self, _: &[u8]) -> Result<()> {
            return Err(Error::new(Some(self.path.clone()), "Bytes serializer not implemented".into()))
        }

        // An absent optional is represented as the JSON `null`.
        fn serialize_none(self) -> Result<()> {
            self.serialize_unit()
        }

        // A present optional is represented as just the contained value. Note that
        // this is a lossy representation. For example the values `Some(())` and
        // `None` both serialize as just `null`. Unfortunately this is typically
        // what people expect when working with JSON. Other formats are encouraged
        // to behave more intelligently if possible.
        fn serialize_some<T>(self, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            value.serialize(self)
        }

        // In Serde, unit means an anonymous value containing no data. Map this to
        // JSON as `null`.
        fn serialize_unit(self) -> Result<()> {
            self.output.push( (self.path.clone(), format!("{}","null")) );
            Ok(())
        }

        // Unit struct means a named value containing no data. Again, since there is
        // no data, map this to JSON as `null`. There is no need to serialize the
        // name in most formats.
        fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
            self.serialize_unit()
        }

        // When serializing a unit variant (or any other kind of variant), formats
        // can choose whether to keep track of it by index or by name. Binary
        // formats typically use the index of the variant and human-readable formats
        // typically use the name.
        fn serialize_unit_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            variant: &'static str,
        ) -> Result<()> {
            self.serialize_str(variant)
        }

        // As is done here, serializers are encouraged to treat newtype structs as
        // insignificant wrappers around the data they contain.
        fn serialize_newtype_struct<T>(
            self,
            _name: &'static str,
            value: &T,
        ) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            value.serialize(self)
        }

        // Note that newtype variant (and all of the other variant serialization
        // methods) refer exclusively to the "externally tagged" enum
        // representation.
        //
        // Serialize this to JSON in externally tagged form as `{ NAME: VALUE }`.
        fn serialize_newtype_variant<T>(
            self,
            _name: &'static str,
            _variant_index: u32,
            variant: &'static str,
            value: &T,
        ) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            self.path = format!("{}[\"{}\"]",self.path,variant);
            value.serialize(self)?;
            Ok(())
        }

        // Now we get to the serialization of compound types.
        //
        // The start of the sequence, each value, and the end are three separate
        // method calls. This one is responsible only for serializing the start,
        // which in JSON is `[`.
        //
        // The length of the sequence may or may not be known ahead of time. This
        // doesn't make a difference in JSON because the length is not represented
        // explicitly in the serialized form. Some serializers may only be able to
        // support sequences for which the length is known up front.
        fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
            Ok( SeqSerializer::new(self))
        }

        // Tuples look just like sequences in JSON. Some formats may be able to
        // represent tuples more efficiently by omitting the length, since tuple
        // means that the corresponding `Deserialize implementation will know the
        // length without needing to look at the serialized data.
        fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
            self.serialize_seq(Some(len))
        }

        // Tuple structs look just like sequences in JSON.
        fn serialize_tuple_struct(
            self,
            _name: &'static str,
            len: usize,
        ) -> Result<Self::SerializeTupleStruct> {
            self.serialize_seq(Some(len))
        }

        // Tuple variants are represented in JSON as `{ NAME: [DATA...] }`. Again
        // this method is only responsible for the externally tagged representation.
        fn serialize_tuple_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleVariant> {
            self.path = format!("{}[\"{}\"]",self.path,variant);
            Ok(SeqSerializer::new(self))
        }

        // Maps are represented in JSON as `{ K: V, K: V, ... }`.
        fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
            Ok(MapSerializer::new(self) )
        }

        // Structs look just like maps in JSON. In particular, JSON requires that we
        // serialize the field names of the struct. Other formats may be able to
        // omit the field names when serializing structs because the corresponding
        // Deserialize implementation is required to know what the keys are without
        // looking at the serialized data.
        fn serialize_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStruct> {
            Ok(self)
        }

        // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }`.
        // This is the externally tagged representation.
        fn serialize_struct_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStructVariant> {
            self.path = format!("{}[\"{}\"]",self.path,variant);
            Ok(self)
        }
    }




    // Structs are like maps in which the keys are constrained to be compile-time
    // constant strings.
    impl<'a> ser::SerializeStruct for &'a mut Serializer {
        type Ok = ();
        type Error = Error;

        fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
        where
            T: ?Sized + Serialize,
        {
            let last_path = self.path.clone();
            if  last_path == ""
            {
                self.path = format!("{}",key);

            }
            else {
                self.path = format!("{}[{}]",self.path,key);
            }
            value.serialize(&mut **self)?;
            self.path = last_path;
            Ok(())
            
        }

        fn end(self) -> Result<()> {
            Ok(())
        }
    }
    impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        <Self as ser::SerializeStruct>::serialize_field::<T>(self,key,value)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}
}


////////////////////////////////////////////////////////////////////////////////

mod tests {
use serde::Serialize;

#[test]
fn test_struct() {
    #[derive(Serialize)]
    struct NestedTest {
        val: String,
        included: bool,
    }
    #[derive(Serialize)]
    struct Test {
        int: u32,
        nested: NestedTest,
        seq: Vec<&'static str>
    }

    let test = Test {
        int: 1,
        nested: NestedTest {
            val: "hello".into(),
            included: true
        },
        seq: vec!["a", "b"],
    };
    
    let res = crate::ser::to_string(&test).unwrap();

    let expected ="int=1&nested%5Bval%5D=hello&nested%5Bincluded%5D=true&seq%5B0%5D=a&seq%5B1%5D=b";

    assert_eq!(res,expected)


}
}