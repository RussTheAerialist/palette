extern crate image;
extern crate palette;

use palette::{Gradient, Lch, LinSrgb, Pixel, Srgb};

use image::{GenericImage, RgbImage};

fn main() {
    //A gradient of evenly spaced colors
    let grad1 = Gradient::new(vec![
        LinSrgb::new(1.0, 0.1, 0.1),
        LinSrgb::new(0.1, 0.1, 1.0),
        LinSrgb::new(0.1, 1.0, 0.1),
    ]);

    //The same colors as in grad1, but with the blue point shifted down
    let grad2 = Gradient::with_domain(vec![
        (0.0, LinSrgb::new(1.0, 0.1, 0.1)),
        (0.25, LinSrgb::new(0.1, 0.1, 1.0)),
        (1.0, LinSrgb::new(0.1, 1.0, 0.1)),
    ]);

    //The same colors and offsets as in grad1, but in a color space where the hue
    // is a component
    let grad3 = Gradient::new(vec![
        Lch::from(LinSrgb::new(1.0, 0.1, 0.1)),
        Lch::from(LinSrgb::new(0.1, 0.1, 1.0)),
        Lch::from(LinSrgb::new(0.1, 1.0, 0.1)),
    ]);

    //The same colors and and color space as in grad3, but with the blue point
    // shifted down
    let grad4 = Gradient::with_domain(vec![
        (0.0, Lch::from(LinSrgb::new(1.0, 0.1, 0.1))),
        (0.25, Lch::from(LinSrgb::new(0.1, 0.1, 1.0))),
        (1.0, Lch::from(LinSrgb::new(0.1, 1.0, 0.1))),
    ]);

    let mut image = RgbImage::new(256, 128);

    for (i, ((c1, c2), (c3, c4))) in grad1
        .take(256)
        .zip(grad2.take(256))
        .zip(grad3.take(256).zip(grad4.take(256)))
        .enumerate()
    {
        let c1 = Srgb::from_linear(c1).into_format().into_raw();
        let c2 = Srgb::from_linear(c2).into_format().into_raw();
        let c3 = Srgb::from_linear(c3.into()).into_format().into_raw();
        let c4 = Srgb::from_linear(c4.into()).into_format().into_raw();

        for (_, _, pixel) in image.sub_image(i as u32, 0, 1, 31).pixels_mut() {
            pixel.data = c1
        }

        for (_, _, pixel) in image.sub_image(i as u32, 32, 1, 31).pixels_mut() {
            pixel.data = c2;
        }

        for (_, _, pixel) in image.sub_image(i as u32, 65, 1, 31).pixels_mut() {
            pixel.data = c3;
        }

        for (_, _, pixel) in image.sub_image(i as u32, 97, 1, 31).pixels_mut() {
            pixel.data = c4;
        }
    }

    match image.save("examples/gradient.png") {
        Ok(()) => println!("see 'examples/gradient.png' for the result"),
        Err(e) => println!("failed to write 'examples/gradient.png': {}", e),
    }
}
