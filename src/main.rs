use clap::{App, Arg};
use regex::Regex;
use std::{fs, fs::File, io::BufWriter};
use yaml_rust::YamlLoader;

fn main() {
    let args = App::new("base16ify")
        .version("0.1")
        .author("raccoon! <raccoon@raccoon.fun>")
        .about("Modifies 16-color paletted image to match a base16 scheme")
        .arg(
            Arg::with_name("SCHEME")
                .required(true)
                .help("base16 scheme to use"),
        )
        .arg(
            Arg::with_name("INPUT")
                .required(true)
                .help("path of png to use"),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .required(true)
                .help("path of png to output"),
        )
        .get_matches();

    let colors = {
        let yaml = &YamlLoader::load_from_str(
            &fs::read_to_string(args.value_of("SCHEME").unwrap()).expect("could not read scheme"),
        )
        .expect("could not parse scheme")[0];

        let re_k = Regex::new(r"base0([0-9a-fA-F])").unwrap();
        let re_v = Regex::new(r"([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})").unwrap();

        let mut colors: [u8; 3 * 16] = [0; 3 * 16];

        for (k, v) in yaml.as_hash().unwrap().iter() {
            if let Some(caps_k) = re_k.captures(k.as_str().unwrap()) {
                let i = usize::from_str_radix(&caps_k[1], 16).unwrap();
                let caps_v = re_v.captures(v.as_str().unwrap()).unwrap();

                colors[i*3] = u8::from_str_radix(&caps_v[1], 16).unwrap();
                colors[i*3+1] = u8::from_str_radix(&caps_v[2], 16).unwrap();
                colors[i*3+2] = u8::from_str_radix(&caps_v[3], 16).unwrap();
            }
        }

        colors
    };

    let (info, mut reader) = {
        let mut decoder = png::Decoder::new(
            File::open(args.value_of("INPUT").unwrap()).expect("could not open input"),
        );

        decoder.set_transformations(png::Transformations::IDENTITY);

        decoder
            .read_info()
            .expect("failed to read metadata from input")
    };

    let mut buf = vec![0; info.buffer_size()];
    reader
        .next_frame(&mut buf)
        .expect("failed to read frame from input");

    let info = reader.info();

    match info.color_type {
        png::ColorType::Indexed => {
            let ref mut bufwriter =
                BufWriter::new(File::create(args.value_of("OUTPUT").unwrap()).unwrap());
            let mut encoder = png::Encoder::new(bufwriter, info.width, info.height);

            encoder.set_color(png::ColorType::Indexed);
            encoder.set_depth(info.bit_depth);
            encoder.set_palette(colors.to_vec());

            let mut writer = encoder
                .write_header()
                .expect("failed to write header to output");

            writer
                .write_image_data(&buf)
                .expect("failed to write image data to output");
        }
        t => panic!("input is {:?} instead of Indexed", t)
    }
}
