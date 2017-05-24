

struct MagicHelper {
    diag_mask: [u64; 64],
    slide_mask: [u64; 64],
}

impl MagicHelper {
    pub fn new() -> MagicHelper {
        MagicHelper {
            diag_mask: MagicHelper::gen_diag_mask(),
            slide_mask: MagicHelper::gen_slide_mask()
        }
    }

    pub fn default() -> MagicHelper { MagicHelper::new() }

    fn gen_diag_mask() -> [u64; 64] {
        let mut arr: [u64; 64] = [0; 64];
    }

    fn gen_slide_mask() -> [u64; 64] {
        let mut arr: [u64; 64] = [0; 64];

    }
}