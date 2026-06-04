use glam::{DMat4, DVec3};
use kpe_schema::geometry::TransformOp;

pub(crate) fn local_matrix(tf: &Option<TransformOp>) -> DMat4 {
    match tf {
        Some(t) => {
            let mut mat = DMat4::IDENTITY;

            if let Some(trans) = &t.translation {
                mat = DMat4::from_translation(DVec3::new(trans[0], trans[1], trans[2]));
            }

            if let Some(rot) = &t.rotation {
                let rx = DMat4::from_rotation_x(rot[0].to_radians());
                let ry = DMat4::from_rotation_y(rot[1].to_radians());
                let rz = DMat4::from_rotation_z(rot[2].to_radians());
                mat = mat * rz * ry * rx;
            }

            if let Some(scale) = &t.scale {
                mat = mat * DMat4::from_scale(DVec3::new(scale[0], scale[1], scale[2]));
            }

            for pv in &t.pivots {
                match pv.space {
                    kpe_schema::geometry::PivotSpace::Local => {
                        let to_p = DMat4::from_translation(DVec3::new(pv.pivot[0], pv.pivot[1], pv.pivot[2]));
                        let from_p = DMat4::from_translation(DVec3::new(-pv.pivot[0], -pv.pivot[1], -pv.pivot[2]));
                        mat = mat * to_p;
                        if let Some(tv) = &pv.translation {
                            mat = mat * DMat4::from_translation(DVec3::new(tv[0], tv[1], tv[2]));
                        }
                        if let Some(rot) = &pv.rotation {
                            let rx = DMat4::from_rotation_x(rot[0].to_radians());
                            let ry = DMat4::from_rotation_y(rot[1].to_radians());
                            let rz = DMat4::from_rotation_z(rot[2].to_radians());
                            mat = mat * rz * ry * rx;
                        }
                        if let Some(scale) = &pv.scale {
                            mat = mat * DMat4::from_scale(DVec3::new(scale[0], scale[1], scale[2]));
                        }
                        mat = mat * from_p;
                    }
                    kpe_schema::geometry::PivotSpace::World => {
                        let effective = [
                            pv.pivot[0] + pv.translation.unwrap_or([0.0; 3])[0],
                            pv.pivot[1] + pv.translation.unwrap_or([0.0; 3])[1],
                            pv.pivot[2] + pv.translation.unwrap_or([0.0; 3])[2],
                        ];
                        let to_p = DMat4::from_translation(DVec3::new(effective[0], effective[1], effective[2]));
                        let from_p = DMat4::from_translation(DVec3::new(-effective[0], -effective[1], -effective[2]));
                        let mut pivot_mat = to_p;
                        if let Some(rot) = &pv.rotation {
                            let rx = DMat4::from_rotation_x(rot[0].to_radians());
                            let ry = DMat4::from_rotation_y(rot[1].to_radians());
                            let rz = DMat4::from_rotation_z(rot[2].to_radians());
                            pivot_mat = pivot_mat * rz * ry * rx;
                        }
                        if let Some(scale) = &pv.scale {
                            pivot_mat = pivot_mat * DMat4::from_scale(DVec3::new(scale[0], scale[1], scale[2]));
                        }
                        pivot_mat = pivot_mat * from_p;
                        mat = pivot_mat * mat;
                    }
                }
            }

            mat
        }
        None => DMat4::IDENTITY,
    }
}
