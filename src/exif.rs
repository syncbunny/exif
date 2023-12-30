use std::collections::HashMap;

pub struct EXIF {
    byte_order: ENDIAN,
    pub values: HashMap<String, String>,
}

enum ENDIAN {
    BIG_ENDIAN, LITTLE_ENDIAN
}

impl EXIF {
    pub fn load(data:&[u8]) -> Option<EXIF> {
        let mut exif = EXIF {
            byte_order: ENDIAN::LITTLE_ENDIAN,
            values: HashMap::new(),
        };
    
        // byte order
        exif.byte_order = match data[0] {
            0x49 => {
                println!("Intel");
                ENDIAN::LITTLE_ENDIAN
            }
            0x4D => {
                println!("Motorola");
                ENDIAN::BIG_ENDIAN
            }
            _ => {
                println!("unknown byte order");
                return None;
            }
        };
        let offset = read_4bytes(data, 4, &exif.byte_order);
        exif.load_ifd(data, offset.unwrap() as usize);

        Some(exif)
    }

    fn load_ifd(&mut self, data:&[u8], offset:usize) -> Option<usize> {
        let mut cnt:usize = 0;
        let mut ifd_offset:usize = offset;

        let ifd_cnt = read_2bytes(data, ifd_offset, &self.byte_order);
        if ifd_cnt.is_none() {
            return None;
        }
        ifd_offset += 2;

        cnt += 2;

        for _ in 0..ifd_cnt.unwrap() {
            let tag = read_2bytes(data, ifd_offset, &self.byte_order);
            if tag.is_none() {
                return None;
            }
            ifd_offset += 2;

            let ifd_type = read_2bytes(data, ifd_offset, &self.byte_order);
            if ifd_type.is_none() {
                return None;
            }
            ifd_offset += 2;

            let count = read_4bytes(data, ifd_offset, &self.byte_order);
            if count.is_none() {
                return None;
            }
            ifd_offset += 4;

            let value = read_4bytes(data, ifd_offset, &self.byte_order);
            if value.is_none() {
                return None;
            }
            ifd_offset += 4;

            self.tag(tag.unwrap(), ifd_type.unwrap(), count.unwrap(), value.unwrap(), data);
        }

        Some(cnt)
    }

    fn tag(&mut self, tag: u16, ifd_type: u16, count: u32, value: u32, data:&[u8]) {
        match tag {
            0x010F => {
                self.tag_string(ifd_type, count, value, data, "Make");
            }
            0x0110 => {
                self.tag_string(ifd_type, count, value, data, "Model");
            }
            0x0112 => {
                self.values.insert("Orientation".to_string(), value.to_string());
            }
            0x11A => {
                self.tag_rational(ifd_type, count, value, data, "XResolution");
            }
            0x11B => {
                self.tag_rational(ifd_type, count, value, data, "YResolution");
            }
            0x128 => {
                self.values.insert("ResolutionUnit".to_string(), value.to_string());
            }
            0x131 => {
                self.tag_string(ifd_type, count, value, data, "Software");
            }
            0x132 => {
                self.tag_string(ifd_type, count, value, data, "DateTime");
            }
            0x13B => {
                self.tag_string(ifd_type, count, value, data, "Artist");
            }
            0x213 => {
                self.values.insert("YCbCrPositioning".to_string(), value.to_string());
            }
            0x8298 => {
                self.tag_string(ifd_type, count, value, data, "Copyright");
            }
            0x829A => {
                self.tag_rational(ifd_type, count, value, data, "ExposureTime");
            }
            0x829D => {
                self.tag_rational(ifd_type, count, value, data, "FNumber");
            }
            0x8769 => {
                //let data_slice = &data[value as usize..];
                //dump(data_slice);
//                let exif = EXIF::load(data_slice);
                // TODO: ExifOffset
                self.load_ifd(data, value as usize);
            }
            0x8822 => {
                self.values.insert("ExposureProgram".to_string(), value.to_string());
            }
            0x8825 => {
                // TODO: GPSInfo
            }
            0x8827 => {
                self.values.insert("PhotographicSensivility".to_string(), value.to_string());
            }
            0x8830 => {
                self.values.insert("SensitivityType".to_string(), value.to_string());
            }
            0x9000 => {
                // TODO: ExifVersion
            }
            0x9003 => {
                self.tag_string(ifd_type, count, value, data, "DateTimeOriginal");
            }
            0x9004 => {
                self.tag_string(ifd_type, count, value, data, "DateTimeDigitized");
            }
            0x9010 => {
                self.tag_string(ifd_type, count, value, data, "OffsetTime");
            }
            0x9101 => {
                // TODO: ComponentsConfiguration
                // self.values.insert("ComponentsConfiguration".to_string(), format!("{:08X}", value));
            }
            0x9201 => {
                self.tag_srational(ifd_type, count, value, data, "ShutterSpeedValue");
            }
            0x9202 => {
                self.tag_rational(ifd_type, count, value, data, "ApertureValue");
            }
            0x9203 => {
                self.tag_srational(ifd_type, count, value, data, "BrightnessValue");
            }
            0x9204 => {
                self.tag_srational(ifd_type, count, value, data, "ExposureBiasValue");
            }
            0x9205 => {
                self.tag_rational(ifd_type, count, value, data, "MaxApertureValue");
            }
            0x9207 => {
                self.values.insert("MeteringMode".to_string(), value.to_string());
            }
            0x9208 => {
                self.values.insert("LightSource".to_string(), value.to_string());
            }
            0x9209 => {
                self.values.insert("Flash".to_string(), value.to_string());
            }
            0x920A => {
                self.tag_rational(ifd_type, count, value, data, "FocalLength");
            }
            0x9214 => {
                self.values.insert("SubjectArea".to_string(), value.to_string());
            }
            0x927C => {
                // perhaps MakerNote is string?
                self.values.insert("MakerNote".to_string(), value.to_string());
            }
            0x9286 => {
                // perhaps UserComment is string?
                self.values.insert("UserComment".to_string(), value.to_string());
            }
            0x9290 => {
                self.tag_string(ifd_type, count, value, data, "SubSecTime");
            }
            0x9291 => {
                self.tag_string(ifd_type, count, value, data, "SubSecTimeOriginal");
            }
            0x9292 => {
                self.tag_string(ifd_type, count, value, data, "SubSecTimeDigitized");
            }
            0xA000 => {
                self.values.insert("FlashPixVersion".to_string(), value.to_string());
            }
            0xA001 => {
                self.values.insert("ColorSpace".to_string(), value.to_string());
            }
            0xA002 => {
                self.values.insert("PixelXDimension".to_string(), value.to_string());
            }
            0xA003 => {
                self.values.insert("PixelYDimension".to_string(), value.to_string());
            }
            0xA20E => {
                self.tag_rational(ifd_type, count, value, data, "FocalPlaneXResolution");
            }
            0xA20F => {
                self.tag_rational(ifd_type, count, value, data, "FocalPlaneYResolution");
            }
            0xA210 => {
                self.values.insert("FocalPlaneResolutionUnit".to_string(), value.to_string());
            }
            0xA217 => {
                self.values.insert("SensingMethod".to_string(), value.to_string());
            }
            0xA300 => {
                self.values.insert("FileSource".to_string(), value.to_string());
            }
            0xA301 => {
                self.values.insert("SceneType".to_string(), value.to_string());
            }
            0xA302 => {
                self.values.insert("CFAPattern".to_string(), value.to_string());
            }
            0xA401 => {
                self.values.insert("CustomRendered".to_string(), value.to_string());
            }
            0xA402 => {
                self.values.insert("ExposureMode".to_string(), value.to_string());
            }
            0xA403 => {
                self.values.insert("WhiteBalance".to_string(), value.to_string());
            }
            0xA404 => {
                self.tag_rational(ifd_type, count, value, data, "DigitalZoomRatio");
            }
            0xA405 => {
                self.values.insert("FocalLengthIn35mmFilm".to_string(), value.to_string());
            }
            0xA406 => {
                self.values.insert("SceneCaptureType".to_string(), value.to_string());
            }
            0xA407 => {
                self.values.insert("GainControl".to_string(), value.to_string());
            }
            0xA408 => {
                self.values.insert("Contrast".to_string(), value.to_string());
            }
            0xA409 => {
                self.values.insert("Saturation".to_string(), value.to_string());
            }
            0xA40A => {
                self.values.insert("Sharpness".to_string(), value.to_string());
            }
            0xA40C => {
                self.values.insert("SubjectDistanceRange".to_string(), value.to_string());
            }
            0xA431 => {
                self.tag_string(ifd_type, count, value, data, "BodySerialNumber");
            }
            0xA432 => {
                self.tag_rational(ifd_type, count, value, data, "LensSpecification");
            }
            0xA433 => {
                self.tag_string(ifd_type, count, value, data, "LensMake");
            }
            0xA434 => {
                self.tag_string(ifd_type, count, value, data, "LensModel");
            }
            0x214 => {
                self.tag_rational(ifd_type, count, value+0, data, "ReferenceBlackWhite0");
                self.tag_rational(ifd_type, count, value+8, data, "ReferenceBlackWhite1");
                self.tag_rational(ifd_type, count, value+16, data, "ReferenceBlackWhite2");
                self.tag_rational(ifd_type, count, value+24, data, "ReferenceBlackWhite3");
                self.tag_rational(ifd_type, count, value+32, data, "ReferenceBlackWhite4");
                self.tag_rational(ifd_type, count, value+40, data, "ReferenceBlackWhite5");
            }
            _ => {
                println!("unknown tag");
                println!("tag: {:04X}", tag);
                println!("type: {:04X}", ifd_type);
                println!("count: {}", count);
                println!("value: {}", value);
            }
        }
    }

    fn tag_string(&mut self, ifd_type: u16, count: u32, value: u32, data:&[u8], tag_name:&str) {
        if ifd_type != 2 {
            println!("invalid type");
            return;
        }
        if count <= 4 {
            return;
        }
        let offset = value as usize;
        let data_slice = &data[offset..offset+count as usize];
        self.values.insert(tag_name.to_string(), String::from_utf8_lossy(data_slice).to_string());
    }

    fn tag_rational(&mut self, ifd_type: u16, count: u32, value: u32, data:&[u8], tag_name:&str) {
        if ifd_type != 5 {
            println!("invalid type");
            return;
        }
        let offset = value as usize;
        let top = read_4bytes(data, offset, &self.byte_order);
        let bottom = read_4bytes(data, offset+4, &self.byte_order);
        self.values.insert(tag_name.to_string(), format!("{}/{}", top.unwrap(), bottom.unwrap()));
    }

    fn tag_srational(&mut self, ifd_type: u16, count: u32, value: u32, data:&[u8], tag_name:&str) {
        if ifd_type != 10 {
            println!("invalid type");
            return;
        }
        let offset = value as usize;
        let top = read_4bytes_signed(data, offset, &self.byte_order);
        let bottom = read_4bytes_signed(data, offset+4, &self.byte_order);
        self.values.insert(tag_name.to_string(), format!("{}/{}", top.unwrap(), bottom.unwrap()));
    }
}

fn read_2bytes(data: &[u8], offset: usize, endian: &ENDIAN) -> Option<u16> {
    let mut r:u16 = 0;

    match endian {
        ENDIAN::BIG_ENDIAN => {
            r = data[offset] as u16;
            r <<= 8;
            r |= data[offset+1] as u16;
        }
        ENDIAN::LITTLE_ENDIAN =>{
            r = data[offset+1] as u16;
            r <<= 8;
            r |= data[offset] as u16;
        }
        _ => {
            return None;
        }
    }

    Some(r)
}

fn read_4bytes(data: &[u8], offset: usize, endian: &ENDIAN) -> Option<u32> {
    let mut r:u32 = 0;

    match endian {
        ENDIAN::BIG_ENDIAN => {
            r = data[offset] as u32;
            r <<= 8;
            r |= data[offset+1] as u32;
            r <<= 8;
            r |= data[offset+2] as u32;
            r <<= 8;
            r |= data[offset+3] as u32;
        }
        ENDIAN::LITTLE_ENDIAN => {
            r = data[offset+3] as u32;
            r <<= 8;
            r |= data[offset+2] as u32;
            r <<= 8;
            r |= data[offset+1] as u32;
            r <<= 8;
            r |= data[offset] as u32;
        }
        _ => {
            return None;
        }
    }

    Some(r)
}

fn read_4bytes_signed(data: &[u8], offset: usize, endian: &ENDIAN) -> Option<i32> {
    let mut r:u32 = 0;

    match endian {
        ENDIAN::BIG_ENDIAN => {
            r = data[offset] as u32;
            r <<= 8;
            r |= data[offset+1] as u32;
            r <<= 8;
            r |= data[offset+2] as u32;
            r <<= 8;
            r |= data[offset+3] as u32;
        }
        ENDIAN::LITTLE_ENDIAN => {
            r = data[offset+3] as u32;
            r <<= 8;
            r |= data[offset+2] as u32;
            r <<= 8;
            r |= data[offset+1] as u32;
            r <<= 8;
            r |= data[offset] as u32;
        }
        _ => {
            return None;
        }
    }

    Some(r as i32)
}

fn dump(data: &[u8]) {
    let mut cnt:usize = 0;
    for d in data {
        print!("{:02X} ", d);
        cnt += 1;
        if cnt % 16 == 0 {
            println!("");
        }
    }
    println!("");
}