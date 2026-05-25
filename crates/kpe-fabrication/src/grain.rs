use kpe_schema::fabrication::{CutPiece, GrainDirection, NestingConfig};

pub struct GrainConstraintEngine;

impl GrainConstraintEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_grain(&self, piece: &CutPiece, config: &NestingConfig) -> bool {
        if !config.respect_grain {
            return true;
        }

        match piece.grain_direction {
            GrainDirection::Lengthwise => piece.width >= piece.height,
            GrainDirection::Widthwise => piece.height >= piece.width,
            GrainDirection::None => true,
        }
    }

    pub fn should_rotate(&self, piece: &CutPiece, config: &NestingConfig) -> bool {
        if !config.respect_grain {
            return false;
        }

        match piece.grain_direction {
            GrainDirection::Lengthwise => piece.height > piece.width,
            GrainDirection::Widthwise => piece.width > piece.height,
            GrainDirection::None => false,
        }
    }

    pub fn pieces_by_grain<'a>(&self, pieces: &'a [CutPiece]) -> (Vec<&'a CutPiece>, Vec<&'a CutPiece>) {
        let lengthwise: Vec<_> = pieces.iter()
            .filter(|p| matches!(p.grain_direction, GrainDirection::Lengthwise))
            .collect();
        let others: Vec<_> = pieces.iter()
            .filter(|p| !matches!(p.grain_direction, GrainDirection::Lengthwise))
            .collect();
        (lengthwise, others)
    }
}

impl Default for GrainConstraintEngine {
    fn default() -> Self {
        Self::new()
    }
}
