use num_traits::Float;

use std::ops::{Add, Div, Mul, Sub};
use std::marker::PhantomData;

use {Alpha, Lab, Luma, Yxy};
use {Component, ComponentWise, Limited, Mix, Pixel, Shade};
use white_point::{D65, WhitePoint};
use rgb::{Rgb, RgbSpace, RgbStandard};
use luma::LumaStandard;
use encoding::pixel::RawPixel;
use matrix::{multiply_rgb_to_xyz, rgb_to_xyz_matrix};
use {cast, clamp};

/// CIE 1931 XYZ with an alpha component. See the [`Xyza` implementation in
/// `Alpha`](struct.Alpha.html#Xyza).
pub type Xyza<Wp = D65, T = f32> = Alpha<Xyz<Wp, T>, T>;

///The CIE 1931 XYZ color space.
///
///XYZ links the perceived colors to their wavelengths and simply makes it
///possible to describe the way we see colors as numbers. It's often used when
///converting from one color space to an other, and requires a standard
///illuminant and a standard observer to be defined.
///
///Conversions and operations on this color space depend on the defined white point
#[derive(Debug, PartialEq, FromColor)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[palette_internal]
#[palette_white_point = "Wp"]
#[palette_component = "T"]
#[palette_manual_from(Xyz, Rgb, Lab, Yxy, Luma)]
#[repr(C)]
pub struct Xyz<Wp = D65, T = f32>
where
    T: Component + Float,
    Wp: WhitePoint,
{
    ///X is the scale of what can be seen as a response curve for the cone
    ///cells in the human eye. Its range depends
    ///on the white point and goes from 0.0 to 0.95047 for the default D65.
    pub x: T,

    ///Y is the luminance of the color, where 0.0 is black and 1.0 is white.
    pub y: T,

    ///Z is the scale of what can be seen as the blue stimulation. Its range depends
    ///on the white point and goes from 0.0 to 1.08883 for the defautl D65.
    pub z: T,

    ///The white point associated with the color's illuminant and observer.
    ///D65 for 2 degree observer is used by default.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub white_point: PhantomData<Wp>,
}

impl<Wp, T> Copy for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
{
}

impl<Wp, T> Clone for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
{
    fn clone(&self) -> Xyz<Wp, T> {
        *self
    }
}

unsafe impl<Wp: WhitePoint, T: Component + Float> Pixel<T> for Xyz<Wp, T> {
    const CHANNELS: usize = 3;
}

impl<T> Xyz<D65, T>
where
    T: Component + Float,
{
    ///CIE XYZ with whtie point D65.
    pub fn new(x: T, y: T, z: T) -> Xyz<D65, T> {
        Xyz {
            x: x,
            y: y,
            z: z,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
{
    ///CIE XYZ.
    pub fn with_wp(x: T, y: T, z: T) -> Xyz<Wp, T> {
        Xyz {
            x: x,
            y: y,
            z: z,
            white_point: PhantomData,
        }
    }
}

///<span id="Xyza"></span>[`Xyza`](type.Xyza.html) implementations.
impl<T> Alpha<Xyz<D65, T>, T>
where
    T: Component + Float,
{
    ///CIE Yxy and transparency with white point D65.
    pub fn new(x: T, y: T, luma: T, alpha: T) -> Xyza<D65, T> {
        Alpha {
            color: Xyz::new(x, y, luma),
            alpha: alpha,
        }
    }
}

///<span id="Xyza"></span>[`Xyza`](type.Xyza.html) implementations.
impl<Wp, T> Alpha<Xyz<Wp, T>, T>
where
    T: Component + Float,
    Wp: WhitePoint,
{
    ///CIE XYZ and transparency.
    pub fn with_wp(x: T, y: T, z: T, alpha: T) -> Xyza<Wp, T> {
        Alpha {
            color: Xyz::with_wp(x, y, z),
            alpha: alpha,
        }
    }
}

impl<Wp, T, S> From<Rgb<S, T>> for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
    S: RgbStandard,
    S::Space: RgbSpace<WhitePoint = Wp>,
{
    fn from(color: Rgb<S, T>) -> Self {
        let transform_matrix = rgb_to_xyz_matrix::<S::Space, T>();
        multiply_rgb_to_xyz(&transform_matrix, &color.into_linear())
    }
}

impl<Wp, T> From<Yxy<Wp, T>> for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
{
    fn from(color: Yxy<Wp, T>) -> Self {
        let mut xyz = Xyz {
            y: color.luma,
            ..Default::default()
        };
        // If denominator is zero, NAN or INFINITE leave x and z at the default 0
        if color.y.is_normal() {
            xyz.x = color.luma * color.x / color.y;
            xyz.z = color.luma * (T::one() - color.x - color.y) / color.y;
        }
        xyz
    }
}

impl<Wp, T> From<Lab<Wp, T>> for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
{
    fn from(color: Lab<Wp, T>) -> Self {
        let y = (color.l + cast(16.0)) / cast(116.0);
        let x = y + (color.a / cast(500.0));
        let z = y - (color.b / cast(200.0));

        fn convert<T: Component + Float>(c: T) -> T {
            let epsilon: T = cast(6.0 / 29.0);
            let kappa: T = cast(108.0 / 841.0);
            let delta: T = cast(4.0 / 29.0);

            if c > epsilon {
                c.powi(3)
            } else {
                (c - delta) * kappa
            }
        }

        Xyz::with_wp(convert(x), convert(y), convert(z)) * Wp::get_xyz()
    }
}

impl<Wp, T, S> From<Luma<S, T>> for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
    S: LumaStandard<WhitePoint = Wp>,
{
    fn from(color: Luma<S, T>) -> Self {
        Wp::get_xyz() * color.luma
    }
}

impl<Wp, T> Limited for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
{
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn is_valid(&self) -> bool {
        let xyz_ref: Self = Wp::get_xyz();
        self.x >= T::zero() && self.x <= xyz_ref.x &&
        self.y >= T::zero() && self.y <= xyz_ref.y &&
        self.z >= T::zero() && self.z <= xyz_ref.z
    }

    fn clamp(&self) -> Xyz<Wp, T> {
        let mut c = *self;
        c.clamp_self();
        c
    }

    fn clamp_self(&mut self) {
        let xyz_ref: Self = Wp::get_xyz();
        self.x = clamp(self.x, T::zero(), xyz_ref.x);
        self.y = clamp(self.y, T::zero(), xyz_ref.y);
        self.z = clamp(self.z, T::zero(), xyz_ref.z);
    }
}

impl<Wp, T> Mix for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
{
    type Scalar = T;

    fn mix(&self, other: &Xyz<Wp, T>, factor: T) -> Xyz<Wp, T> {
        let factor = clamp(factor, T::zero(), T::one());

        Xyz {
            x: self.x + factor * (other.x - self.x),
            y: self.y + factor * (other.y - self.y),
            z: self.z + factor * (other.z - self.z),
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> Shade for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
{
    type Scalar = T;

    fn lighten(&self, amount: T) -> Xyz<Wp, T> {
        Xyz {
            x: self.x,
            y: self.y + amount,
            z: self.z,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> ComponentWise for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
{
    type Scalar = T;

    fn component_wise<F: FnMut(T, T) -> T>(&self, other: &Xyz<Wp, T>, mut f: F) -> Xyz<Wp, T> {
        Xyz {
            x: f(self.x, other.x),
            y: f(self.y, other.y),
            z: f(self.z, other.z),
            white_point: PhantomData,
        }
    }

    fn component_wise_self<F: FnMut(T) -> T>(&self, mut f: F) -> Xyz<Wp, T> {
        Xyz {
            x: f(self.x),
            y: f(self.y),
            z: f(self.z),
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> Default for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
{
    fn default() -> Xyz<Wp, T> {
        Xyz::with_wp(T::zero(), T::zero(), T::zero())
    }
}

impl<Wp, T> Add<Xyz<Wp, T>> for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
{
    type Output = Xyz<Wp, T>;

    fn add(self, other: Xyz<Wp, T>) -> Xyz<Wp, T> {
        Xyz {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> Add<T> for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
{
    type Output = Xyz<Wp, T>;

    fn add(self, c: T) -> Xyz<Wp, T> {
        Xyz {
            x: self.x + c,
            y: self.y + c,
            z: self.z + c,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> Sub<Xyz<Wp, T>> for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
{
    type Output = Xyz<Wp, T>;

    fn sub(self, other: Xyz<Wp, T>) -> Xyz<Wp, T> {
        Xyz {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> Sub<T> for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
{
    type Output = Xyz<Wp, T>;

    fn sub(self, c: T) -> Xyz<Wp, T> {
        Xyz {
            x: self.x - c,
            y: self.y - c,
            z: self.z - c,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> Mul<Xyz<Wp, T>> for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
{
    type Output = Xyz<Wp, T>;

    fn mul(self, other: Xyz<Wp, T>) -> Xyz<Wp, T> {
        Xyz {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> Mul<T> for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
{
    type Output = Xyz<Wp, T>;

    fn mul(self, c: T) -> Xyz<Wp, T> {
        Xyz {
            x: self.x * c,
            y: self.y * c,
            z: self.z * c,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> Div<Xyz<Wp, T>> for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
{
    type Output = Xyz<Wp, T>;

    fn div(self, other: Xyz<Wp, T>) -> Xyz<Wp, T> {
        Xyz {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T> Div<T> for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
{
    type Output = Xyz<Wp, T>;

    fn div(self, c: T) -> Xyz<Wp, T> {
        Xyz {
            x: self.x / c,
            y: self.y / c,
            z: self.z / c,
            white_point: PhantomData,
        }
    }
}

impl<Wp, T, P> AsRef<P> for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
    P: RawPixel<T> + ?Sized,
{
    fn as_ref(&self) -> &P {
        self.as_raw()
    }
}

impl<Wp, T, P> AsMut<P> for Xyz<Wp, T>
where
    T: Component + Float,
    Wp: WhitePoint,
    P: RawPixel<T> + ?Sized,
{
    fn as_mut(&mut self) -> &mut P {
        self.as_raw_mut()
    }
}

#[cfg(test)]
mod test {
    use super::Xyz;
    use LinSrgb;
    use LinLuma;
    use white_point::D65;
    const X_N: f64 = 0.95047;
    const Y_N: f64 = 1.0;
    const Z_N: f64 = 1.08883;

    #[test]
    fn luma() {
        let a = Xyz::from(LinLuma::new(0.5));
        let b = Xyz::new(0.475235, 0.5, 0.544415);
        assert_relative_eq!(a, b, epsilon = 0.0001);
    }

    #[test]
    fn red() {
        let a = Xyz::from(LinSrgb::new(1.0, 0.0, 0.0));
        let b = Xyz::new(0.41240, 0.21260, 0.01930);
        assert_relative_eq!(a, b, epsilon = 0.0001);
    }

    #[test]
    fn green() {
        let a = Xyz::from(LinSrgb::new(0.0, 1.0, 0.0));
        let b = Xyz::new(0.35760, 0.71520, 0.11920);
        assert_relative_eq!(a, b, epsilon = 0.0001);
    }

    #[test]
    fn blue() {
        let a = Xyz::from(LinSrgb::new(0.0, 0.0, 1.0));
        let b = Xyz::new(0.18050, 0.07220, 0.95030);
        assert_relative_eq!(a, b, epsilon = 0.0001);
    }

    #[test]
    fn ranges() {
        assert_ranges!{
            Xyz<D65, f64>;
            limited {
                x: 0.0 => X_N,
                y: 0.0 => Y_N,
                z: 0.0 => Z_N
            }
            limited_min {}
            unlimited {}
        }
    }

    raw_pixel_conversion_tests!(Xyz<D65>: x, y, z);
    raw_pixel_conversion_fail_tests!(Xyz<D65>: x, y, z);

    #[cfg(feature = "serde")]
    #[test]
    fn serialize() {
        let serialized = ::serde_json::to_string(&Xyz::new(0.3, 0.8, 0.1)).unwrap();

        assert_eq!(serialized, r#"{"x":0.3,"y":0.8,"z":0.1}"#);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize() {
        let deserialized: Xyz = ::serde_json::from_str(r#"{"x":0.3,"y":0.8,"z":0.1}"#).unwrap();

        assert_eq!(deserialized, Xyz::new(0.3, 0.8, 0.1));
    }
}
