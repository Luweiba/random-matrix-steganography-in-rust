/// 实现hide与reveal逻辑
use crate::random_matrix::RandMatrix;
use image::io::Reader as ImageReader;
use std::error::Error;
use std::path::Path;
use image::{DynamicImage, GenericImageView, RgbImage, Rgb, ImageBuffer, Rgba, RgbaImage};
use std::ops::Index;

#[derive(Debug)]
pub struct RMSteg {
    random_matrix: RandMatrix,
    seed: u64,
}

impl RMSteg {
    /// 创建
    pub fn new(seed: u64) -> Self {
        let random_matrix = RandMatrix::from_seed_u64(seed);
        Self {
            random_matrix,
            seed,
        }
    }

    pub fn hide<P>(&self, path: P, cipher_text: &[u8]) -> ImageBuffer<image::Rgba<u8>, Vec<u8>>
    where  P: AsRef<Path>
    {
        let cipher_text_9 = Self::transform(cipher_text);
        let carrier = ImageReader::open(path).unwrap().decode().unwrap();
        let (carrier_x, carrier_y) = carrier.dimensions();
        let payload_len = cipher_text_9.len();
        if Self::is_payload_to_large(payload_len, carrier_x, carrier_y) {
            panic!("Cipher text is too large");
        }
        let mut payload = Self::transform_length_prefix(payload_len);
        payload.extend_from_slice(cipher_text_9.as_slice());
        // 要生成的图片
        let mut img: RgbaImage = ImageBuffer::new(carrier_x, carrier_y);
        let mut pixel_x_mut= &mut Rgba([0,0,0,0]);
        let mut pixel_y_mut= &mut Rgba([0,0,0,0]);
        let mut pixel_x_carrier= Rgba([0,0,0,0]);
        let mut pixel_y_carrier= Rgba([0,0,0,0]);
        let mut state = 0;
        let mut payload_cursor = 0;
        let mut flag = false;
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            if flag {
                let pixel_carrier = carrier.get_pixel(x, y);
                *pixel = Rgba([*pixel_carrier.index(0), *pixel_carrier.index(1), *pixel_carrier.index(2), *pixel_carrier.index(3)]);
                continue;
            }
            if state == 0 {
                pixel_x_carrier = carrier.get_pixel(x, y);
                pixel_x_mut = pixel;
                state = 1;
            } else {
                pixel_y_carrier = carrier.get_pixel(x, y);
                //println!("x: {}, y: {}, Carrier_Pixel_x: {:?}, Carrier_pixel_y: {:?}", x, y, pixel_x_carrier, pixel_y_carrier);
                pixel_y_mut = pixel;
                let g1 = *pixel_x_carrier.index(0);
                let g2 = *pixel_y_carrier.index(0);
                let payload_byte = payload[payload_cursor];
                let (new_x, new_y) = self.random_matrix.search_val(g1 as usize, g2 as usize, payload_byte);
                //println!("({}, {}) => ({}, {})", g1, g2, new_x, new_y);
                //println!("Payload: {}, reveal: {}", payload_byte, self.random_matrix.get_val_from_random_matrix(new_x as usize, new_y as usize));
                //println!("[{}]Hide: {}", payload_cursor, payload_byte);
                *pixel_x_mut = Rgba([new_x, *pixel_x_carrier.index(1), *pixel_x_carrier.index(2), *pixel_x_carrier.index(3)]);
                *pixel_y_mut = Rgba([new_y, *pixel_y_carrier.index(1), *pixel_y_carrier.index(2), *pixel_y_carrier.index(3)]);
                payload_cursor += 1;
                if payload_cursor == payload.len() {
                    flag = true;
                }
                state = 0;
            }
        }
        img
    }

    /// 解密
    pub fn reveal<P>(&self, path: P) -> String
    where P:AsRef<Path>
    {
        let carrier = image::open(path).unwrap();
        let (carrier_x, carrier_y) = carrier.dimensions();

        let prefix_length = 5;
        let mut cipher_text_length = carrier_x*carrier_y / 2;
        let mut cipher_text_in_9 = vec![];
        let mut pixel_x_carrier= Rgba([0,0,0,0]);
        let mut pixel_y_carrier= Rgba([0,0,0,0]);
        let mut state = 0;
        let mut prefix_cnt = 0;
        let mut byte_cnt = 0;
        let mut break_flag = false;
        for y in 0..carrier_y {
            for x in 0..carrier_x {
                if state == 0 {
                    pixel_x_carrier = carrier.get_pixel(x, y);
                    state = 1;
                } else {
                    pixel_y_carrier = carrier.get_pixel(x, y);
                    let g1 = *pixel_x_carrier.index(0);
                    let g2 = *pixel_y_carrier.index(0);
                    let cipher_text_byte = self.random_matrix.get_val_from_random_matrix(g1 as usize, g2 as usize);
                    cipher_text_in_9.push(cipher_text_byte);
                    prefix_cnt += 1;
                    byte_cnt += 1;
                    if prefix_cnt == prefix_length {
                        cipher_text_length = Self::re_transform_length_prefix(&mut cipher_text_in_9);
                        byte_cnt = 0;
                    }
                    if byte_cnt == cipher_text_length {
                        break_flag = true;
                        break;
                    }
                    state = 0;
                }
            }
            if break_flag {
                break;
            }
        }
        let cipher_text = Self::re_transform(cipher_text_in_9);
        String::from_utf8(cipher_text).unwrap()
    }
    /// 判断隐写密文是否过大
    fn is_payload_to_large(payload_len: usize, carrier_x: u32, carrier_y: u32) -> bool {
        //println!("X: {}, Y: {}, Parload_len: {}", carrier_x, carrier_y, payload_len);
        let carrier_total_bytes = ((carrier_x-1) * (carrier_y-1)) as usize;
        if carrier_total_bytes / 2 > payload_len + 5 {
            false
        } else {
            true
        }
    }
    fn re_transform_length_prefix(prefix_length: &mut Vec<u8>) -> u32 {
        let mut len = 0;
        let mut scale = 1;
        for &x in prefix_length.iter().rev() {
            len += (x as u32) * scale;
            scale *= 9;
        }
        prefix_length.clear();
        len
    }
    fn transform_length_prefix(len: usize) -> Vec<u8> {
        let mut length_prefix = vec![];
        let mut len_tmp = len;
        let prefix_4 = (len_tmp % 9) as u8;
        len_tmp /= 9;
        let prefix_3 = (len_tmp % 9) as u8;
        len_tmp /= 9;
        let prefix_2 = (len_tmp % 9) as u8;
        len_tmp /= 9;
        let prefix_1 = (len_tmp % 9) as u8;
        len_tmp /= 9;
        let prefix_0 = (len_tmp % 9) as u8;
        length_prefix.push(prefix_0);
        length_prefix.push(prefix_1);
        length_prefix.push(prefix_2);
        length_prefix.push(prefix_3);
        length_prefix.push(prefix_4);
        length_prefix

    }
    fn re_transform(cipher_text_in_9: Vec<u8>) -> Vec<u8> {
        let mut cipher_text = vec![];
        let len = cipher_text_in_9.len();
        for tuple in cipher_text_in_9.chunks(3) {
            let mut cipher_text_byte: u32 = 0;
            let mut scale: u32 = 1;
            for &byte in tuple.iter().rev() {
                cipher_text_byte += ((byte as u32)* scale) % 256;
                scale *= 9;
            }
            cipher_text.push(cipher_text_byte as u8);
        }
        cipher_text
    }
    fn transform(cipher_text: &[u8]) -> Vec<u8> {
        let mut cipher_text_9 = vec![];
        /// 将一个字节映射为三个字节
        /// item :0-255 => high: 0-9, middle: 0-9, low: 0-9
        for item in cipher_text.iter() {
            let mut raw_byte = *item;
            let low = raw_byte % 9;
            raw_byte /= 9;
            let middle = raw_byte % 9;
            raw_byte /= 9;
            let high = raw_byte % 9;
            cipher_text_9.push(high);
            cipher_text_9.push(middle);
            cipher_text_9.push(low);
            //println!("u8: {} => {}{}{} in nine", item, high, middle, low);
        }
        cipher_text_9
    }
}