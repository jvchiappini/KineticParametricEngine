pub mod solver;
pub mod expression;
pub mod condition;
pub mod rule_engine;
pub mod catalog;
pub mod scene;
pub mod commands;
pub mod history;

pub use solver::Solver;
pub use expression::ExpressionEvaluator;
pub use condition::ConditionEvaluator;
pub use rule_engine::RuleEngine;
pub use catalog::Catalog;
pub use scene::GeometryScene;
pub use commands::{
    Command, find_node, find_parent, find_parent_and_index, collect_ids, reassign_ids,
    next_counter, remove_node_by_id, reparent_node,
    set_node_translation, set_node_parameter,
    SetParameterCommand, AddFeatureCommand, DeleteFeatureCommand,
    MoveNodeCommand, MoveNodeTransformCommand, get_node_translation,
    SetSketchCommand, AddJointCommand, SetJointValueCommand,
    ModifyJointCommand, RemoveJointCommand, CompoundCommand,
    add_box_command, add_cylinder_command, add_sphere_command, add_sketch_command,
};
pub use commands::pivot::{
    AddPivotCommand, RemovePivotCommand, SetPivotTransformCommand, PivotField,
    generate_pivot_id,
};
pub use commands::features::{
    ArrayParams, MirrorPlane,
    build_duplicate_command, build_array_command, build_mirror_command,
    build_fillet_command, build_chamfer_command, build_group_command,
    build_assembly_command, build_delete_command, build_delete_multi_command,
    build_add_joint_command, build_joint_group_command, build_set_joint_value_command,
};
pub use history::CommandHistory;
