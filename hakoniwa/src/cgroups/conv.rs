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

    // Round and clamp to valid cgroup v1 'cpu.shares' limits.
    shares.round().clamp(2.0, 262144.0) as u64
}
