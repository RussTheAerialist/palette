extern crate image;
extern crate num_traits;
extern crate palette;

use image::{GenericImage, RgbImage};

use palette::{Gradient, LinSrgb, Mix, Pixel, Srgb};

mod color_spaces {
    use palette::{Hue, Lch, LinSrgb, Srgb};
    use display_colors;

    pub fn run() {
        let lch_color: Lch = Srgb::new(0.8, 0.2, 0.1).into();
        let new_color = LinSrgb::from(lch_color.shift_hue(180.0));

        display_colors(
            "examples/readme_color_spaces.png",
            &[
                ::palette::Srgb::new(0.8, 0.2, 0.1).into_format(),
                Srgb::from_linear(new_color.into()).into_format(),
            ],
        );
    }
}

mod manipulation {
    use palette::{Color, Saturate, Shade, Srgb};
    use display_colors;

    pub fn run() {
        let color: Color = Srgb::new(0.8, 0.2, 0.1).into_linear().into();
        let lighter = color.lighten(0.1);
        let desaturated = color.desaturate(0.5);

        display_colors(
            "examples/readme_manipulation.png",
            &[
                Srgb::from_linear(color.into()).into_format(),
                Srgb::from_linear(lighter.into()).into_format(),
                Srgb::from_linear(desaturated.into()).into_format(),
            ],
        );
    }
}

mod gradients {
    use palette::{Gradient, Hsv, LinSrgb};
    use display_gradients;

    pub fn run() {
        let grad1 = Gradient::new(vec![
            LinSrgb::new(1.0, 0.1, 0.1),
            LinSrgb::new(0.1, 1.0, 1.0),
        ]);

        let grad2 = Gradient::new(vec![
            Hsv::from(LinSrgb::new(1.0, 0.1, 0.1)),
            Hsv::from(LinSrgb::new(0.1, 1.0, 1.0)),
        ]);

        display_gradients("examples/readme_gradients.png", grad1, grad2);
    }
}

fn display_colors(filename: &str, colors: &[Srgb<u8>]) {
    let mut image = RgbImage::new(colors.len() as u32 * 64, 64);
    for (i, &color) in colors.iter().enumerate() {
        for (_, _, pixel) in image.sub_image(i as u32 * 64, 0, 64, 64).pixels_mut() {
            pixel.data = *color.as_raw();
        }
    }

    match image.save(filename) {
        Ok(()) => println!("see '{}' for the result", filename),
        Err(e) => println!("failed to write '{}': {}", filename, e),
    }
}

fn display_gradients<A: Mix<Scalar = f32> + Clone, B: Mix<Scalar = f32> + Clone>(
    filename: &str,
    grad1: Gradient<A>,
    grad2: Gradient<B>,
) where
    LinSrgb: From<A>,
    LinSrgb: From<B>,
{
    let mut image = RgbImage::new(256, 64);

    for (x, _, pixel) in image.sub_image(0, 0, 256, 32).pixels_mut() {
        pixel.data = Srgb::from_linear(grad1.get(x as f32 / 255.0).into())
            .into_format()
            .into_raw();
    }

    for (x, _, pixel) in image.sub_image(0, 32, 256, 32).pixels_mut() {
        pixel.data = Srgb::from_linear(grad2.get(x as f32 / 255.0).into())
            .into_format()
            .into_raw();
    }

    match image.save(filename) {
        Ok(()) => println!("see '{}' for the result", filename),
        Err(e) => println!("failed to write '{}': {}", filename, e),
    }
}

fn main() {
    color_spaces::run();
    manipulation::run();
    gradients::run();
}
