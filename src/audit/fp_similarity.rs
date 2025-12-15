// src/audit/fp_similarity.rs
//! Tests for fingerprint similarity detection.

#[cfg(test)]
mod tests {
    use crate::audit::fingerprint::similarity;
    use crate::audit::types::Fingerprint;

    #[test]
    fn test_cfg_hash_equivalence() {
        let fp1 = Fingerprint {
            hash: 1,
            cfg_hash: 100,
            depth: 5,
            node_count: 20,
            branch_count: 2,
            loop_count: 1,
            exit_count: 1,
        };
        let fp2 = Fingerprint {
            hash: 2,
            cfg_hash: 100, // Same CFG hash
            depth: 5,
            node_count: 22,
            branch_count: 2,
            loop_count: 1,
            exit_count: 1,
        };
        let sim = similarity(&fp1, &fp2);
        assert!(sim >= 0.85, "CFG-equivalent should be >= 85%, got {sim}");
    }

    #[test]
    fn test_different_cfg_similar_metrics() {
        let fp1 = Fingerprint {
            hash: 1,
            cfg_hash: 100,
            depth: 5,
            node_count: 20,
            branch_count: 2,
            loop_count: 1,
            exit_count: 1,
        };
        let fp2 = Fingerprint {
            hash: 2,
            cfg_hash: 200, // Different CFG
            depth: 5,
            node_count: 20,
            branch_count: 2, // Same metrics
            loop_count: 1,
            exit_count: 1,
        };
        let sim = similarity(&fp1, &fp2);
        assert!(sim >= 0.9, "Same CFG metrics should be >= 90%, got {sim}");
    }

    #[test]
    fn test_exact_match() {
        let fp = Fingerprint {
            hash: 12345,
            cfg_hash: 100,
            depth: 5,
            node_count: 20,
            branch_count: 2,
            loop_count: 1,
            exit_count: 1,
        };
        assert!((similarity(&fp, &fp) - 1.0).abs() < f64::EPSILON);
    }
}