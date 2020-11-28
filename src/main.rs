mod random_matrix;
mod steganography;
mod error;

use rand::rngs::StdRng;
use rand::{SeedableRng, Rng};
use structopt::StructOpt;
use random_matrix::RandMatrix;
use steganography::RMSteg;

#[derive(Debug, StructOpt)]
#[structopt(name="Random Matrix Steganogrophy Cli", about="A Command Line tool for Steganography based on Random Matrix.")]
struct Opt {
    #[structopt(short, long, default_value="19990529", help="伪随机数生成器（PRNG）的密钥种子（Seed）")]
    seed: u64,
    #[structopt(short, help="flag to switch between Hide/Reveal", about="flag to do reveal some cipher text from an Image(Rgba)")]
    de_steg: bool,
    #[structopt(short, help="待嵌入秘密信息的图片路径（PNG）", default_value="D:/rsprojects/random_matrix_steganography/images/carrier.png")]
    carrier_image_path: String,
    #[structopt(short, help="嵌入秘密信息的图片输出路径（PNG）", default_value="D:/rsprojects/random_matrix_steganography/images/output.png")]
    output_image_path: String,
    #[structopt(short, help="待提取秘密信息的载密图片路径（PNG）", default_value="D:/rsprojects/random_matrix_steganography/images/output.png")]
    reveal_image_path: String,
    #[structopt(short="m", help="嵌入密文", default_value="Luweiba Never Give Up!")]
    cipher_text: String,
}
fn main() {
    let opt: Opt = Opt::from_args();
    let seed = opt.seed;
    let rmsteg = RMSteg::new(seed);
    // Reveal Mode
    if !opt.de_steg {
        let carrier_image_path = opt.carrier_image_path;
        let output_image_path = opt.output_image_path;
        let cipher_text = opt.cipher_text;
        let output_image = rmsteg.hide(carrier_image_path.as_str(), cipher_text.as_bytes());
        output_image.save(output_image_path.as_str()).unwrap();
        println!("Successfully Hide cipher text: {}", cipher_text);
    } else {
        let reveal_image_path = opt.reveal_image_path;
        let cipher_text = rmsteg.reveal(reveal_image_path);
        println!("Cipher Text: {}", cipher_text);
    }
}
