#[cfg(test)]
mod tests {
    use ntru_rs::convolution_polynomial::{
        center_lift, extended_gcd, gcd, inverse, ternary_polynomial, ConvPoly,
    };
    use rand::Rng;

    mod ternary_polynomial_tests {
        use super::*;

        #[test]
        fn test_ternary_polynomial() {
            let num_tests = 100;

            for _ in 0..num_tests {
                let n = rand::thread_rng().gen_range(1..=15);
                let num_ones = rand::thread_rng().gen_range(0..=n);
                let num_neg_ones = rand::thread_rng().gen_range(0..=(n - num_ones));
                let poly = ternary_polynomial(n, num_ones, num_neg_ones);

                assert!(
                    poly.coeffs.len() <= n,
                    "Number of coefficients should be <= n"
                );
                assert_eq!(
                    poly.coeffs.iter().filter(|&&c| c == 1).count(),
                    num_ones,
                    "Number of 1 coefficients should match. N: {}, Poly: {:?}",
                    n,
                    poly
                );
                assert_eq!(
                    poly.coeffs.iter().filter(|&&c| c == -1).count(),
                    num_neg_ones,
                    "Number of -1 coefficients should match"
                );
                assert!(
                    poly.coeffs.iter().all(|&c| c >= -1 && c <= 1),
                    "Coefficients should be ternary (-1, 0, 1)"
                );
            }
        }
    }

    mod convolution_polynomial_tests {
        use rand::seq::SliceRandom;

        use super::*;

        #[test]
        fn test_trim() {
            // Trim all zeros
            let poly = ConvPoly {
                coeffs: vec![0, 0, 0, 0, 0],
            };
            let expected_trimmed = ConvPoly::constant(0);
            let trimmed = poly.trim();
            assert_eq!(
                expected_trimmed.coeffs, trimmed.coeffs,
                "Trim all zeros failed"
            );

            // Trim some zeros
            let poly = ConvPoly {
                coeffs: vec![0, 0, 0, 1, 0, 0],
            };
            let expected_trimmed = ConvPoly {
                coeffs: vec![0, 0, 0, 1],
            };
            let trimmed = poly.trim();
            assert_eq!(
                expected_trimmed.coeffs, trimmed.coeffs,
                "Trim some zeros failed"
            );

            // Trim no zeros
            let poly = ConvPoly {
                coeffs: vec![0, 0, 3, 4, 5],
            };
            let expected_trimmed = ConvPoly {
                coeffs: vec![0, 0, 3, 4, 5],
            };
            let trimmed = poly.trim();
            assert_eq!(
                expected_trimmed.coeffs, trimmed.coeffs,
                "Trim no zeros failed"
            );
        }

        #[test]
        fn test_constant() {
            // Zero polynomial
            let poly = ConvPoly::constant(0);
            assert_eq!(poly.coeffs, vec![0], "Constant polynomial failed");

            // Positive constant polynomial
            let poly = ConvPoly::constant(5);
            assert_eq!(poly.coeffs, vec![5], "Constant polynomial failed");

            // Negative constant polynomial
            let poly = ConvPoly::constant(-5);
            assert_eq!(poly.coeffs, vec![-5], "Constant polynomial failed");
        }

        #[test]
        fn test_deg() {
            // Normal polynomial
            let poly = ConvPoly {
                coeffs: vec![0, 0, 3, 0, 0], // 3x^2
            };
            assert_eq!(poly.deg(), 2, "Degree should be 2");

            // Zero polynomial
            let zero_poly = ConvPoly { coeffs: vec![0; 5] };
            assert_eq!(zero_poly.deg(), 0, "Degree of zero polynomial should be 0");

            // Completely non-zero polynomial
            let poly = ConvPoly {
                coeffs: vec![1, 2, 3, 4, 5], // 5x^4 + 4x^3 + 3x^2 + 2x + 1
            };
            assert_eq!(poly.deg(), 4, "Degree should be 4");
        }

        #[test]
        fn test_is_zero() {
            // Zero polynomial
            let zero_poly = ConvPoly { coeffs: vec![0; 5] };
            assert!(zero_poly.is_zero(), "Zero polynomial should return true");

            // Non-zero polynomial
            let non_zero_poly = ConvPoly {
                coeffs: vec![0, 1], // x
            };
            assert!(
                !non_zero_poly.is_zero(),
                "Non-zero polynomial should return false"
            );

            // Completely non-zero polynomial
            let poly = ConvPoly {
                coeffs: vec![1, 2, 3, 4, 5], // 5x^4 + 4x^3 + 3x^2 + 2x + 1
            };
            assert!(
                !poly.is_zero(),
                "Completely non-zero polynomial should return false"
            );

            // Partially non-zero polynomial
            let poly = ConvPoly {
                coeffs: vec![1, 1, 0, 0, 1], // x^4 + x + 1
            };
            assert!(
                !poly.is_zero(),
                "Completely non-zero polynomial should return false"
            );
        }

        #[test]
        fn test_lc() {
            // Leading coefficient at the end
            let poly = ConvPoly {
                coeffs: vec![1, 2, 3, 4, 5], // 5x^4 + 4x^3 + 3x^2 + 2x + 1
            };
            assert_eq!(poly.lc(), 5, "Leading coefficient should be 5");

            // Leading coefficient with zeros at the end
            let poly = ConvPoly {
                coeffs: vec![1, 2, 3, 4, 0], // 4x^3 + 3x^2 + 2x + 1
            };
            assert_eq!(poly.lc(), 4, "Leading coefficient should be 4");

            // Leading coefficient with zeros at the beginning
            let poly = ConvPoly {
                coeffs: vec![0, 0, 3, 4, 5], // 5x^4 + 4x^3 + 3x^2
            };
            assert_eq!(poly.lc(), 5, "Leading coefficient should be 5");

            // Leading coefficient surrounded by zeros
            let poly = ConvPoly {
                coeffs: vec![0, 0, 3, 0, 0], // 3x^2
            };
            assert_eq!(poly.lc(), 3, "Leading coefficient should be 3");
        }

        #[test]
        fn test_modulo() {
            // Modulo 5
            let poly = ConvPoly {
                coeffs: vec![1, 2, 3, 4, 5], // 5x^4 + 4x^3 + 3x^2 + 2x + 1
            };
            let expected_result = ConvPoly {
                coeffs: vec![1, 2, 3, 4], // 4x^3 + 3x^2 + 2x + 1
            };
            let result = poly.modulo(5);
            assert_eq!(expected_result.coeffs, result.coeffs, "Modulo 5 failed");

            // Modulo 5 with negative coefficients
            let poly = ConvPoly {
                coeffs: vec![1, -2, 3, -4, 5], // 5x^4 - 4x^3 + 3x^2 - 2x + 1
            };
            let expected_result = ConvPoly {
                coeffs: vec![1, 3, 3, 1], // x^3 + 3x^2 + 3x + 1
            };
            let result = poly.modulo(5);
            assert_eq!(expected_result.coeffs, result.coeffs, "Modulo 5 failed");

            // Modulo 5 with the result being zero
            let poly = ConvPoly {
                coeffs: vec![-5, 10, 15, 20, 1000], // 1000x^4 + 20x^3 + 15x^2 + 10x - 5
            };
            let expected_result = ConvPoly::constant(0);
            let result = poly.modulo(5);
            assert_eq!(expected_result.coeffs, result.coeffs, "Modulo 5 failed");

            // Modulo 5 with all coefficients being zero
            let poly = ConvPoly { coeffs: vec![0; 5] };
            let expected_result = ConvPoly::constant(0);
            let result = poly.modulo(5);
            assert_eq!(expected_result.coeffs, result.coeffs, "Modulo 5 failed");
        }

        #[test]
        fn test_center_lift() {
            // Example in the ring (Z/7Z)[x]/(x^5 - 1)
            let poly = ConvPoly {
                coeffs: vec![5, 3, -6, 2, 4], // 4x^4 + 2x^3 - 6x^2 + 3x + 5
            };
            let expected_lift = ConvPoly {
                coeffs: vec![-2, 3, 1, 2, -3], // −3x^4 + 2x^3 + x^2 + 3x - 2
            };
            let lift = poly.center_lift(7);
            assert_eq!(
                expected_lift.coeffs, lift.coeffs,
                "Center lift modulo 7 failed"
            );

            // Another example in the ring (Z/7Z)[x]/(x^5 - 1)
            let poly = ConvPoly {
                coeffs: vec![3, 0, 5, -6, 3], // 3x^4 - 6x^3 + 5x^2 + 3
            };
            let expected_lift = ConvPoly {
                coeffs: vec![3, 0, -2, 1, 3], // 3x^4 + x^3 - 2x^2 + 3
            };
            let lift = poly.center_lift(7);
            assert_eq!(
                expected_lift.coeffs, lift.coeffs,
                "Center lift modulo 7 failed"
            );
        }

        #[test]
        fn test_add() {
            // Addition without modulo
            let poly1 = ConvPoly {
                coeffs: vec![4, 3, 2], // 2x^2 + 3x + 4
            };
            let poly2 = ConvPoly {
                coeffs: vec![2, 2, 2], // 2x^2 + 2x + 2
            };
            let expected_sum = ConvPoly {
                coeffs: vec![6, 5, 4], // 4x^2 + 5x + 6
            };
            let sum = poly1.add(&poly2);
            assert_eq!(
                expected_sum.coeffs, sum.coeffs,
                "Addition without modulo failed"
            );

            // Addition without modulo with negative coefficients
            let poly1 = ConvPoly {
                coeffs: vec![4, -3, 2], // 2x^2 - 3x + 4
            };
            let poly2 = ConvPoly {
                coeffs: vec![2, 2, 2], // 2x^2 + 2x + 2
            };
            let expected_sum = ConvPoly {
                coeffs: vec![6, -1, 4], // 4x^2 - x + 6
            };
            let sum = poly1.add(&poly2);
            assert_eq!(
                expected_sum.coeffs, sum.coeffs,
                "Addition without modulo failed"
            );

            // Addition of additive inverse
            let poly1 = ConvPoly {
                coeffs: vec![4, 3, 2], // 2x^2 + 3x + 4
            };
            let poly2 = ConvPoly {
                coeffs: vec![-4, -3, -2], // -2x^2 - 3x - 4
            };
            let expected_sum = ConvPoly::constant(0);
            let sum = poly1.add(&poly2);
            assert_eq!(
                expected_sum.coeffs, sum.coeffs,
                "Addition of additive inverse failed"
            );
        }

        #[test]
        fn test_add_mod() {
            // Modulo 5 addition without wraparound
            let poly1 = ConvPoly {
                coeffs: vec![1, 2, 3], // 3x^2 + 2x + 1
            };
            let poly2 = ConvPoly {
                coeffs: vec![3, 2, 1], // x^2 + 2x + 3
            };
            let expected_sum = ConvPoly {
                coeffs: vec![4, 4, 4], // 4x^2 + 4x + 4
            };
            let sum = poly1.add(&poly2).modulo(5);
            assert_eq!(expected_sum.coeffs, sum.coeffs, "Addition modulo 5 failed");

            // Modulo 5 addition with wraparound
            let poly1 = ConvPoly {
                coeffs: vec![4, 3, 2], // 2x^2 + 3x + 4
            };
            let poly2 = ConvPoly {
                coeffs: vec![2, -16, 10], // 10x^2 - 16x + 2
            };
            let expected_sum = ConvPoly {
                coeffs: vec![1, 2, 2], // 2x^2 + 2x + 1
            };
            let sum = poly1.add(&poly2).modulo(5);
            assert_eq!(expected_sum.coeffs, sum.coeffs, "Addition modulo 5 failed");

            // Another modulo 5 addition with wraparound
            let poly1 = ConvPoly {
                coeffs: vec![4, 3, 2],
            };
            let poly2 = ConvPoly { coeffs: vec![1, 2] };
            let expected_sum = ConvPoly {
                coeffs: vec![0, 0, 2],
            };
            let sum = poly1.add(&poly2).modulo(5);
            assert_eq!(
                expected_sum.coeffs, sum.coeffs,
                "Partial add modulo 5 failed"
            );
        }

        #[test]
        fn test_sub() {
            // Subtraction without modulo
            let poly1 = ConvPoly {
                coeffs: vec![4, 3, 2], // 2x^2 + 3x + 4
            };
            let poly2 = ConvPoly {
                coeffs: vec![2, 2, 2], // 2x^2 + 2x + 2
            };
            let expected_diff = ConvPoly {
                coeffs: vec![2, 1], // x + 2
            };
            let diff = poly1.sub(&poly2);
            assert_eq!(
                expected_diff.coeffs, diff.coeffs,
                "Subtraction without modulo failed"
            );

            // Subtraction without modulo with negative coefficients
            let poly1 = ConvPoly {
                coeffs: vec![4, -3, 2], // 2x^2 - 3x + 4
            };
            let poly2 = ConvPoly {
                coeffs: vec![2, 2, 3], // 3x^2 + 2x + 2
            };
            let expected_diff = ConvPoly {
                coeffs: vec![2, -5, -1], // -x^2 - 5x + 2
            };
            let diff = poly1.sub(&poly2);
            assert_eq!(
                expected_diff.coeffs, diff.coeffs,
                "Subtraction without modulo failed"
            );

            // Subtraction of self
            let poly1 = ConvPoly {
                coeffs: vec![4, 3, 2], // 2x^2 + 3x + 4
            };
            let poly2 = ConvPoly {
                coeffs: vec![4, 3, 2], // 2x^2 + 3x + 4
            };
            let expected_diff = ConvPoly::constant(0);
            let diff = poly1.sub(&poly2);
            assert_eq!(
                expected_diff.coeffs, diff.coeffs,
                "Subtraction of self failed"
            );
        }

        #[test]
        fn test_sub_mod() {
            // Modulo 5 subtraction without wraparound
            let poly1 = ConvPoly {
                coeffs: vec![4, 3, 2], // 2x^2 + 3x + 4
            };
            let poly2 = ConvPoly {
                coeffs: vec![2, 2, 2], // 2x^2 + 2x + 2
            };
            let expected_diff = ConvPoly {
                coeffs: vec![2, 1], // x + 2
            };
            let diff = poly1.sub(&poly2).modulo(5);
            assert_eq!(
                expected_diff.coeffs, diff.coeffs,
                "Subtraction modulo 5 failed"
            );

            // Modulo 5 subtraction with wraparound
            let poly1 = ConvPoly {
                coeffs: vec![2, 3, 2], // 2x^2 + 3x + 2
            };
            let poly2 = ConvPoly {
                coeffs: vec![-5, 16, 10], // 10x^2 + 16x - 5
            };
            let expected_diff = ConvPoly {
                coeffs: vec![2, 2, 2], // 2x^2 + 2x + 2
            };
            let diff = poly1.sub(&poly2).modulo(5);
            assert_eq!(
                expected_diff.coeffs, diff.coeffs,
                "Subtraction modulo 5 failed"
            );
        }

        #[test]
        fn test_mul() {
            // Multiplication by zero (but not a well-formed zero polynomial)
            let poly1 = ConvPoly {
                coeffs: vec![1, 2, 3], // 3x^2 + 2x + 1
            };
            let poly2 = ConvPoly { coeffs: vec![0; 3] }; // 0
            let expected_product = ConvPoly::constant(0);
            let product = poly1.mul(&poly2, 11);
            assert_eq!(
                expected_product.coeffs, product.coeffs,
                "Multiplication by zero failed"
            );

            // Multiplication by zero (well-formed zero polynomial)
            let poly1 = ConvPoly {
                coeffs: vec![1, 2, 3], // 3x^2 + 2x + 1
            };
            let poly2 = ConvPoly::constant(0);
            let expected_product = ConvPoly::constant(0);
            let product = poly1.mul(&poly2, 11);
            assert_eq!(
                expected_product.coeffs, product.coeffs,
                "Multiplication by zero failed"
            );

            // Multiplication by one
            let poly1 = ConvPoly {
                coeffs: vec![1, 2, 3], // 3x^2 + 2x + 1
            };
            let poly2 = ConvPoly::constant(1);
            let expected_product = ConvPoly {
                coeffs: vec![1, 2, 3], // 3x^2 + 2x + 1
            };
            let product = poly1.mul(&poly2, 17);
            assert_eq!(
                expected_product.coeffs, product.coeffs,
                "Multiplication by one failed"
            );

            // Multiplication by negative one
            let poly1 = ConvPoly {
                coeffs: vec![1, 2, 3], // 3x^2 + 2x + 1
            };
            let poly2 = ConvPoly::constant(-1);
            let expected_product = ConvPoly {
                coeffs: vec![-1, -2, -3], // 3x^2 + 2x + 1
            };
            let product = poly1.mul(&poly2, 23);
            assert_eq!(
                expected_product.coeffs, product.coeffs,
                "Multiplication by one failed"
            );

            // Example in the ring Z[x]/(x^5 - 1)
            let poly1 = ConvPoly {
                coeffs: vec![1, -2, 0, 4, -1], // -x^4 + 4x^3 - 2x + 1
            };
            let poly2 = ConvPoly {
                coeffs: vec![3, 4, -2, 5, 2], // 2x^4 + 5x^3 - 2x^2 + 4x + 3
            };
            let expected_product = ConvPoly {
                coeffs: vec![-13, 20, -7, 19, 5], // 5x^4 + 19x^3 - 7x^2 + 20x - 13
            };
            let product = poly1.mul(&poly2, 5);
            assert_eq!(
                expected_product.coeffs, product.coeffs,
                "Multiplication failed"
            );

            // Example in the ring Z[x]/(x^5 - 1)
            let poly1 = ConvPoly {
                coeffs: vec![-2, 3, 1, 2, -3], // −3x^4 + 2x^3 + x^2 + 3x - 2
            };
            let poly2 = ConvPoly {
                coeffs: vec![3, 0, -2, 1, 3], // 3x^4 + x^3 - 2x^2 + 3
            };
            let expected_product = ConvPoly {
                coeffs: vec![0, 20, 10, -11, -14], // -14x^4 - 11x^3 + 10x^2 + 20x
            };
            let product = poly1.mul(&poly2, 5);
            assert_eq!(
                expected_product.coeffs, product.coeffs,
                "Multiplication failed"
            );
        }

        #[test]
        fn test_mul_mod() {
            // Example in the ring (Z/11Z)[x]/(x^5 - 1)
            let poly1 = ConvPoly {
                coeffs: vec![1, -2, 0, 4, -1], // -x^4 + 4x^3 - 2x + 1
            };
            let poly2 = ConvPoly {
                coeffs: vec![3, 4, -2, 5, 2], // 2x^4 + 5x^3 - 2x^2 + 4x + 3
            };
            let expected_product = ConvPoly {
                coeffs: vec![9, 9, 4, 8, 5], // 5x^4 + 8x^3 + 4x^2 + 9x + 10
            };
            let product = poly1.mul(&poly2, 5).modulo(11);
            assert_eq!(
                expected_product.coeffs, product.coeffs,
                "Multiplication modulo 11 failed"
            );
        }

        #[test]
        fn test_div_mod() {
            // Division by self should return 1
            let poly1 = ConvPoly {
                coeffs: vec![1, 2, 3], // 3x^2 + 2x + 1
            };
            let poly2 = ConvPoly {
                coeffs: vec![1, 2, 3], // 3x^2 + 2x + 1
            };
            let expected_quotient = ConvPoly::constant(1);
            let expected_remainder = ConvPoly::constant(0);
            let (quotient, remainder) = poly1.div_mod(&poly2, 5, 11).unwrap();
            assert_eq!(
                expected_quotient.coeffs, quotient.coeffs,
                "Division quotient failed (divisor is self)"
            );
            assert_eq!(
                expected_remainder.coeffs, remainder.coeffs,
                "Division remainder failed (divisor is self)"
            );

            // Division by 1 should return self
            let poly1 = ConvPoly {
                coeffs: vec![1, 2, 3], // 3x^2 + 2x + 1
            };
            let poly2 = ConvPoly::constant(1);
            let expected_quotient = ConvPoly {
                coeffs: vec![1, 2, 3], // 3x^2 + 2x + 1
            };
            let expected_remainder = ConvPoly::constant(0);
            let (quotient, remainder) = poly1.div_mod(&poly2, 5, 14).unwrap();
            assert_eq!(
                expected_quotient.coeffs, quotient.coeffs,
                "Division quotient failed"
            );
            assert_eq!(
                expected_remainder.coeffs, remainder.coeffs,
                "Division remainder failed"
            );

            // Example in the ring (Z/2Z)[x]/(x^6 - 1)
            let poly1 = ConvPoly {
                coeffs: vec![-1, 0, 0, 0, 0, 1], // x^5 + 1
            };
            let poly2 = ConvPoly {
                coeffs: vec![1, 1, 0, 0, 1], // x^4 + x + 1
            };
            let expected_quotient = ConvPoly {
                coeffs: vec![0, 1], // x
            };
            let expected_remainder = ConvPoly {
                coeffs: vec![1, 1, 1], // x^2 + x + 1
            };
            let (quotient, remainder) = poly1.div_mod(&poly2, 2, 6).unwrap();
            assert_eq!(
                expected_quotient.coeffs, quotient.coeffs,
                "Division quotient failed"
            );
            assert_eq!(
                expected_remainder.coeffs, remainder.coeffs,
                "Division remainder failed"
            );

            // Another example in the ring (Z/2Z)[x]/(x^6 - 1)
            let poly1 = ConvPoly {
                coeffs: vec![1, 1, 0, 0, 1], // x^4 + x + 1
            };
            let poly2 = ConvPoly {
                coeffs: vec![1, 1, 1], // x^2 + x + 1
            };
            let expected_quotient = ConvPoly {
                coeffs: vec![0, 1, 1], // x^2 + x
            };
            let expected_remainder = ConvPoly::constant(1);
            let (quotient, remainder) = poly1.div_mod(&poly2, 2, 6).unwrap();
            assert_eq!(
                expected_quotient.coeffs, quotient.coeffs,
                "Division quotient failed"
            );
            assert_eq!(
                expected_remainder.coeffs, remainder.coeffs,
                "Division remainder failed"
            );

            // Division of a polynomial that is zero in the ring (Z/2Z)[x]/(x^5 - 1)
            // 0 divided by anything non-zero should return (0, 0)
            let poly1 = ConvPoly {
                coeffs: vec![-1, 0, 0, 0, 0, 1], // x^5 - 1 (this is 0 in the ring)
            };
            let poly2 = ConvPoly {
                coeffs: vec![1, 1, 0, 0, 1], // x^4 + x + 1
            };
            let expected_quotient = ConvPoly::constant(0);
            let expected_remainder = ConvPoly::constant(0);
            let (quotient, remainder) = poly1.div_mod(&poly2, 2, 5).unwrap();
            assert_eq!(
                expected_quotient.coeffs, quotient.coeffs,
                "Division quotient failed"
            );
            assert_eq!(
                expected_remainder.coeffs, remainder.coeffs,
                "Division remainder failed"
            );
        }

        #[test]
        fn test_extended_gcd() {
            // Example in the ring (Z/2Z)[x]/(x^6 - 1) with gcd equal to 1
            let poly1 = ConvPoly {
                coeffs: vec![1, 1, 0, 0, 1], // x^4 + x + 1
            };
            let poly2 = ConvPoly {
                coeffs: vec![-1, 0, 0, 0, 0, 1], // x^5 - 1
            };
            let expected_gcd = ConvPoly::constant(1);
            let expected_s = ConvPoly {
                coeffs: vec![1, 0, 1, 1], // x^3 + x^2 + 1
            };
            let expected_t = ConvPoly {
                coeffs: vec![0, 1, 1], // x^2 + x
            };
            let (gcd, s, t) = ConvPoly::extended_gcd(&poly1, &poly2, 2, 6).unwrap();
            assert_eq!(expected_gcd.coeffs, gcd.coeffs, "Extended GCD gcd failed");
            assert_eq!(expected_s.coeffs, s.coeffs, "Extended GCD s failed");
            assert_eq!(expected_t.coeffs, t.coeffs, "Extended GCD t failed");

            // Example in the ring (Z/19Z)[x]/(x^7 - 1) with gcd not equal to 1
            let poly1 = ConvPoly {
                coeffs: vec![0, 0, 18, 3, 3, 6], // 6x^5 + 3x^4 + 3x^3 + 18x^2
            };
            let poly2 = ConvPoly {
                coeffs: vec![-1, 0, 0, 0, 0, 0, 1], // x^6 - 1
            };
            let expected_gcd = ConvPoly {
                coeffs: vec![11, 1], // x + 11
            };
            let expected_s = ConvPoly {
                coeffs: vec![18, 13, 17, 8, 9], //  9x^4 + 8x^3 + 17x^2 + 13x + 18
            };
            let expected_t = ConvPoly {
                coeffs: vec![8, 18, 1, 3], // 3x^3 + x^2 + 18x + 8
            };
            let (gcd, s, t) = ConvPoly::extended_gcd(&poly1, &poly2, 19, 7).unwrap();
            assert_eq!(expected_gcd.coeffs, gcd.coeffs, "Extended GCD gcd failed");
            assert_eq!(expected_s.coeffs, s.coeffs, "Extended GCD s failed");
            assert_eq!(expected_t.coeffs, t.coeffs, "Extended GCD t failed");
        }

        #[test]
        fn test_inverse() {
            // Example in the ring (Z/2Z)[x]/(x^5 - 1)
            let poly = ConvPoly {
                coeffs: vec![1, 1, 0, 0, 1], // x^4 + x + 1
            };
            let expected_inverse = ConvPoly {
                coeffs: vec![1, 0, 1, 1], // x^3 + x^2 + 1
            };
            let inverse = poly.inverse(2, 5).unwrap();
            assert_eq!(
                expected_inverse.coeffs, inverse.coeffs,
                "Inverse modulo 2 failed"
            );
            // Another example in the ring (Z/2Z)[x]/(x^5 - 1)
            let poly = ConvPoly {
                coeffs: vec![7, 3, 1], // x^2 + 3x + 7
            };
            let expected_inverse = ConvPoly {
                coeffs: vec![0, 1, 1, 0, 1], // x^4 + x^2 + x
            };
            let inverse = poly.inverse(2, 5).unwrap();
            assert_eq!(
                expected_inverse.coeffs, inverse.coeffs,
                "Inverse modulo 2 failed"
            );

            // Another example in the ring (Z/2Z)[x]/(x^5 - 1)
            let poly = ConvPoly {
                coeffs: vec![22, 11, 5, 7], // 7x^3 + 5x^2 + 11x + 22
            };
            let expected_inverse = ConvPoly {
                coeffs: vec![1, 1, 0, 1], // x^3 + x + 1
            };
            let inverse = poly.inverse(2, 5).unwrap();
            assert_eq!(
                expected_inverse.coeffs, inverse.coeffs,
                "Inverse modulo 2 failed"
            );

            // Example in the ring (Z/5Z)[x]/(x^7 - 1)
            let poly = ConvPoly {
                coeffs: vec![112, 34, 239, 234, 105, 180, 137], // 137x^6 + 180x^5 + 105x^4 + 234x^3 + 239x^2 + 34x + 112
            };
            let expected_inverse = ConvPoly {
                coeffs: vec![2, 0, 0, 1, 0, 3], // 3x^5 + x^3 + 2
            };
            let inverse = poly.inverse(5, 7).unwrap();
            assert_eq!(
                expected_inverse.coeffs, inverse.coeffs,
                "Inverse modulo 5 failed"
            );

            // Test random polynomials
            let num_tests = 100;
            let mut rng = rand::thread_rng();
            let mut primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
            let mut num_inverse_found = 0;

            for _ in 0..num_tests {
                primes.shuffle(&mut rng);
                let (n, m) = (rng.gen_range(1..=20), primes[0]);

                let mut poly = ConvPoly { coeffs: vec![] };
                for _ in 0..n {
                    poly.coeffs.push(rng.gen_range(-1000..=1000));
                }
                poly = poly.modulo(m).trim();

                if let Ok(inverse) = poly.inverse(m, n) {
                    let product = poly.mul(&inverse, n).modulo(m);
                    assert_eq!(ConvPoly::constant(1), product, "Inverse failed");
                    num_inverse_found += 1;
                } else {
                    println!("(Z/{}Z)[x]/(x^{} - 1) No inverse found for {}", m, n, poly,);
                }
            }

            println!(
                "Found inverses for {} polynomials out of {}",
                num_inverse_found, num_tests
            );
        }
    }

    mod integer_tests {
        use super::*;

        #[test]
        fn test_gcd() {
            // Test with positive numbers
            assert_eq!(gcd(48, 18), 6, "GCD of 48 and 18 should be 6");
            assert_eq!(gcd(101, 103), 1, "GCD of two primes should be 1");
            assert_eq!(gcd(2, 4), 2, "GCD of 2 and 4 should be 2");
            assert_eq!(gcd(72, 36), 36, "GCD of 72 and 36 should be 36");
            assert_eq!(gcd(72, 54), 18, "GCD of 72 and 54 should be 18");

            // Test with two negative numbers
            assert_eq!(gcd(-48, -18), 6, "GCD of -48 and -18 should be 6");
            assert_eq!(gcd(-101, -103), 1, "GCD of two primes should be 1");
            assert_eq!(gcd(-2, -4), 2, "GCD of -2 and -4 should be 2");
            assert_eq!(gcd(-72, -36), 36, "GCD of -72 and -36 should be 36");
            assert_eq!(gcd(-72, -54), 18, "GCD of -72 and -54 should be 18");

            // Test with one positive and one negative number
            assert_eq!(gcd(-48, 18), 6, "GCD of -48 and 18 should be 6");
            assert_eq!(gcd(101, -103), 1, "GCD of 101 and -103 should be 1");
            assert_eq!(gcd(-2, 4), 2, "GCD of -2 and 4 should be 2");
            assert_eq!(gcd(72, -36), 36, "GCD of 72 and -36 should be 36");
            assert_eq!(gcd(-72, 54), 18, "GCD of -72 and 54 should be 18");

            // Test with both a and b being zero
            assert!(
                std::panic::catch_unwind(|| {
                    let _ = gcd(0, 0);
                })
                .is_err(),
                "GCD of 0 and 0 should panic"
            );

            // Test with a being zero and b being non-zero
            assert_eq!(gcd(0, 5), 5, "GCD of 0 and 5 should be 5");

            // Test with a being non-zero and b being zero
            assert_eq!(gcd(5, 0), 5, "GCD of 5 and 0 should be 5");
        }

        #[test]
        fn test_extended_gcd() {
            let mut rng = rand::thread_rng();
            let num_tests = 100;

            // Test with random positive numbers
            for _ in 0..num_tests {
                let (a, b) = (rng.gen_range(1..=1000), rng.gen_range(1..=1000));
                let (d, x, y) = extended_gcd(a, b);

                assert_eq!(d, gcd(a, b), "GCD calculation failed");
                assert_eq!(d, a * x + b * y, "Extended GCD calculation failed");
            }

            // Test with one random positive and one random negative number
            for _ in 0..num_tests {
                let (a, b) = (rng.gen_range(-1000..=-1), rng.gen_range(1..=1000));
                let (d, x, y) = extended_gcd(a, b);

                assert_eq!(d, gcd(a, b), "GCD calculation failed");
                assert_eq!(
                    d,
                    a.abs() * x + b.abs() * y,
                    "Extended GCD calculation failed"
                );
            }
            for _ in 0..num_tests {
                let (a, b) = (rng.gen_range(1..=1000), rng.gen_range(-1000..=-1));
                let (d, x, y) = extended_gcd(a, b);

                assert_eq!(d, gcd(a, b), "GCD calculation failed");
                assert_eq!(
                    d,
                    a.abs() * x + b.abs() * y,
                    "Extended GCD calculation failed"
                );
            }

            // Test with both random negative numbers
            for _ in 0..num_tests {
                let (a, b) = (rng.gen_range(-1000..=-1), rng.gen_range(-1000..=-1));
                let (d, x, y) = extended_gcd(a, b);

                assert_eq!(d, gcd(a, b), "GCD calculation failed");
                assert_eq!(
                    d,
                    a.abs() * x + b.abs() * y,
                    "Extended GCD calculation failed"
                );
            }

            // Test with both a and b being zero
            let (a, b) = (0, 0);
            assert!(
                std::panic::catch_unwind(|| {
                    let _ = extended_gcd(a, b);
                })
                .is_err(),
                "Extended GCD of 0 and 0 should panic"
            );

            // Test with a being zero and b being non-zero
            let (a, b) = (0, rng.gen_range(1..=1000));
            let (d, x, y) = extended_gcd(a, b);
            assert_eq!(d, b, "GCD calculation failed");
            assert_eq!(d, a * x + b * y, "Extended GCD calculation failed");

            // Test with a being non-zero and b being zero
            let (a, b) = (rng.gen_range(1..=1000), 0);
            let (d, x, y) = extended_gcd(a, b);
            assert_eq!(d, a, "GCD calculation failed");
            assert_eq!(d, a * x + b * y, "Extended GCD calculation failed");
        }

        #[test]
        fn test_inverse_modulo_m() {
            // Prime modulus (so that all elements have inverses)
            let inv = inverse(1, 7).expect("Inverse should exist");
            assert_eq!(inv, 1, "Inverse modulo calculation failed");

            let inv = inverse(2, 7).expect("Inverse should exist");
            assert_eq!(inv, 4, "Inverse modulo calculation failed");

            let inv = inverse(3, 7).expect("Inverse should exist");
            assert_eq!(inv, 5, "Inverse modulo calculation failed");

            // Prime modulus with negative input
            let inv = inverse(-1, 7).expect("Inverse should exist");
            assert_eq!(inv, 6, "Inverse modulo calculation failed");

            let inv = inverse(-2, 7).expect("Inverse should exist");
            assert_eq!(inv, 3, "Inverse modulo calculation failed");

            let inv = inverse(-3, 7).expect("Inverse should exist");
            assert_eq!(inv, 2, "Inverse modulo calculation failed");

            // No inverse exists
            assert!(
                inverse(6, 9).is_err(),
                "Inverse should not exist for non-coprime"
            );

            // Should fail assert and panic when a is 0
            assert!(inverse(0, 7).is_err(), "Inverse should not exist for 0");
        }

        #[test]
        fn test_center_lift() {
            // Test cases with an odd modulus
            assert_eq!(center_lift(-4, 7), 3, "Center lift failed");
            assert_eq!(center_lift(-3, 7), -3, "Center lift failed");
            assert_eq!(center_lift(-2, 7), -2, "Center lift failed");
            assert_eq!(center_lift(-1, 7), -1, "Center lift failed");
            assert_eq!(center_lift(0, 7), 0, "Center lift failed");
            assert_eq!(center_lift(1, 7), 1, "Center lift failed");
            assert_eq!(center_lift(2, 7), 2, "Center lift failed");
            assert_eq!(center_lift(3, 7), 3, "Center lift failed");
            assert_eq!(center_lift(4, 7), -3, "Center lift failed");
            assert_eq!(center_lift(5, 7), -2, "Center lift failed");
            assert_eq!(center_lift(6, 7), -1, "Center lift failed");
            assert_eq!(center_lift(7, 7), 0, "Center lift failed");

            // Test cases with an even modulus
            assert_eq!(center_lift(-5, 8), 3, "Center lift failed");
            assert_eq!(center_lift(-4, 8), 4, "Center lift failed");
            assert_eq!(center_lift(-3, 8), -3, "Center lift failed");
            assert_eq!(center_lift(-2, 8), -2, "Center lift failed");
            assert_eq!(center_lift(-1, 8), -1, "Center lift failed");
            assert_eq!(center_lift(0, 8), 0, "Center lift failed");
            assert_eq!(center_lift(1, 8), 1, "Center lift failed");
            assert_eq!(center_lift(2, 8), 2, "Center lift failed");
            assert_eq!(center_lift(3, 8), 3, "Center lift failed");
            assert_eq!(center_lift(4, 8), 4, "Center lift failed");
            assert_eq!(center_lift(5, 8), -3, "Center lift failed");
            assert_eq!(center_lift(6, 8), -2, "Center lift failed");
            assert_eq!(center_lift(7, 8), -1, "Center lift failed");
            assert_eq!(center_lift(8, 8), 0, "Center lift failed");

            // Test random numbers
            let mut rng = rand::thread_rng();
            let num_tests = 1000;

            for _ in 0..num_tests {
                let a = rng.gen_range(-1000..=1000);
                let m = rng.gen_range(1..=1000);
                let result = center_lift(a, m);

                assert_eq!(result.rem_euclid(m), a.rem_euclid(m), "Center lift failed");
                assert!(result >= -m / 2, "Center lift failed");
                assert!(result <= m / 2, "Center lift failed");
            }
        }
    }
}
