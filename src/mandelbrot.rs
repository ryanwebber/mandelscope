use std::ops::AddAssign;

pub fn compute_reference_orbit<const N: usize>(
    z: rug::Complex,
    c: rug::Complex,
    radius: rug::Float,
) -> [glam::f32::Vec2; N] {
    let rsqr = radius.square();
    let mut orbit: [glam::f32::Vec2; N] = [glam::f32::Vec2::ZERO; N];
    let mut z = z;
    // let mut z = rug::Complex::parse("(-3.499370100999999999999999999999999999996e-1 -4.293244312274789964509138456000000000002e-1)").unwrap().complete((128, 128));
    for i in 0..N {
        z.square_mut();
        z.add_assign(&c);

        if (z.real().clone().square()) + (z.imag().clone().square()) > rsqr {
            println!("Invalid reference point after {} iterations: {}", i, z);
        }

        orbit[i] = (z.real().to_f32(), z.imag().to_f32()).into();
    }
    orbit
}

pub fn compute_series_coefficients(
    z: rug::Complex,
    c: rug::Complex,
    radius: rug::Float,
    iterations: usize,
) -> [glam::f32::Vec2; 4] {
    let p = {
        let precisions = [
            z.prec().0,
            z.prec().1,
            c.prec().0,
            c.prec().1,
            radius.prec(),
        ];

        *precisions.iter().max().unwrap()
    };

    let rsqr = radius.square();
    let mut z = z;
    let mut coefficients = [
        rug::Complex::with_val(p, (1.0, 0.0)),
        rug::Complex::with_val(p, (0.0, 0.0)),
        rug::Complex::with_val(p, (0.0, 0.0)),
        rug::Complex::with_val(p, (0.0, 0.0)),
    ];

    for i in 0..iterations {
        z.square_mut();
        z.add_assign(&c);

        if (z.real().clone().square()) + (z.imag().clone().square()) > rsqr {
            println!("Invalid reference point after {} iterations: {}", i, z);
        }

        let z2 = z.clone().square();
        let [a, b, c, d] = coefficients;
        coefficients = [
            a.clone() * z2.clone() + 1.0,
            b.clone() * z2.clone() + a.clone().square(),
            c.clone() * z2.clone() + b.clone() * a.clone() * 2.0,
            d.clone() * z2.clone() + c.clone() * a.clone() * 2.0 + b.clone().square(),
        ];
    }

    let [a, b, c, d] = coefficients;
    let coefficients = [
        (a.real().to_f32(), a.imag().to_f32()).into(),
        (b.real().to_f32(), b.imag().to_f32()).into(),
        (c.real().to_f32(), c.imag().to_f32()).into(),
        (d.real().to_f32(), d.imag().to_f32()).into(),
    ];

    coefficients
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    const PRECISION: u32 = 64;

    #[test]
    fn test_compute_reference_orbit() {
        let c = rug::Complex::with_val(PRECISION, (0.5, 0.0));
        let z = rug::Complex::with_val(PRECISION, (0.5, 0.0));
        let orbits = compute_reference_orbit::<1>(z, c, rug::Float::with_val(PRECISION, 2.0));
        assert_eq!(orbits[0], (0.75, 0.0).into());

        let c = rug::Complex::with_val(PRECISION, (0.5, 0.0));
        let z = rug::Complex::with_val(PRECISION, (0.5001, 0.001));
        let orbits = compute_reference_orbit::<1>(z, c, rug::Float::with_val(PRECISION, 2.0));
        assert_eq!(orbits[0], (0.750099, 0.0010002).into());
    }
}
