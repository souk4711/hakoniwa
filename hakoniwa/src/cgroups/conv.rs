/// Converts cgroup v2 CPU Weight (1-10000) to cgroup v1 CPU shares (2-262144).
///
/// The industry-standard transformation is defined as: W = 10^((log2(S)^2 + 125*log2(S))/612 - 7/34)
/// To reverse this, we treat it as a quadratic equation: aL^2 + bL + c = 0, where L = log2(S).
pub(crate) fn convert_weight_to_shares(weight: u64) -> u64 {
    // Clamp to valid range for the systemd 'CPUWeight' D-Bus property.
    let w = weight.clamp(1, 10000) as f64;

    // E = log10(W)
    let e = w.log10();

    // Coefficients for the quadratic formula: aL^2 + bL + c = 0
    let a = 1.0_f64 / 612.0_f64;
    let b = 125.0_f64 / 612.0_f64;
    let c = -((7.0_f64 / 34.0_f64) + e);

    // Quadratic formula: L = (-b + sqrt(b^2 - 4ac)) / 2a
    let discriminant = b.powi(2) - (4.0_f64 * a * c);
    let l = (-b + discriminant.sqrt()) / (2.0_f64 * a);

    // Convert log2 result back to linear shares (S = 2^L)
    let shares = 2.0_f64.powf(l);

    // Floor (not round) and clamp to valid cgroup v1 'cpu.shares' limits.
    //
    // libcgroups converts shares back to CPUWeight with `ceil` (same as runc),
    // so each weight W owns the shares range where the curve lies in (W-1, W].
    // Flooring the exact solution lands inside that range; rounding up would
    // land in the next one and yield W+1.
    shares.floor().clamp(2.0, 262144.0) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference shares -> weight mapping used by libcgroups (systemd and v2
    /// managers) and runc: https://github.com/opencontainers/cgroups/pull/20
    ///
    /// Copied from libcgroups 0.7 (`convert_shares_to_cgroup2` in
    /// src/systemd/cpu.rs and src/v2/cpu.rs), which keeps it private. Re-check
    /// it when bumping libcgroups.
    fn convert_shares_to_weight(shares: u64) -> u64 {
        if shares <= 2 {
            return 1;
        }
        if shares >= 262_144 {
            return 10_000;
        }
        let l = (shares as f64).log2();
        let exponent = (l * l + 125.0 * l) / 612.0 - 7.0 / 34.0;
        10f64.powf(exponent).ceil() as u64
    }

    #[test]
    fn test_convert_weight_to_shares_roundtrip() {
        for weight in 1..=10_000 {
            let shares = convert_weight_to_shares(weight);
            assert_eq!(
                convert_shares_to_weight(shares),
                weight,
                "weight {weight} -> shares {shares}"
            );
        }
    }

    #[test]
    fn test_convert_weight_to_shares_defaults() {
        assert_eq!(convert_weight_to_shares(100), 1024);
        assert_eq!(convert_weight_to_shares(1), 2);
        assert_eq!(convert_weight_to_shares(10_000), 262_144);
    }
}
