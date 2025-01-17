use std::borrow::Cow;
use std::collections::HashMap;
use std::marker::PhantomData;
use ad_trait::AD;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::egui::panel::{Side, TopBottomSide};
use bevy_egui::egui::Ui;
use bevy_egui::{egui, EguiContexts};
use bevy_prototype_debug_lines::DebugLines;
use optima_3d_spatial::optima_3d_pose::{O3DPose, O3DPoseCategory};
use optima_3d_spatial::optima_3d_rotation::O3DRotation;
use optima_3d_spatial::optima_3d_vec::O3DVec;
use optima_bevy_egui::{OEguiButton, OEguiCheckbox, OEguiContainerTrait, OEguiEngineWrapper, OEguiSelector, OEguiSelectorMode, OEguiSidePanel, OEguiSlider, OEguiTopBottomPanel, OEguiWidgetTrait};
use optima_interpolation::InterpolatorTrait;
use optima_linalg::{OLinalgCategory, OVec};
use optima_proximity::pair_group_queries::{OPairGroupQryTrait, OParryDistanceGroupArgs, OParryDistanceGroupQry, OParryIntersectGroupArgs, OParryIntersectGroupQry, OParryPairSelector, OProximityLossFunction, OSkipReason, ToParryProximityOutputTrait};
use optima_proximity::pair_queries::{ParryDisMode, ParryShapeRep};
use optima_robotics::robot::{FKResult, ORobot, SaveRobot};
use crate::optima_bevy_utils::file::get_asset_path_str_from_ostemcellpath;
use crate::optima_bevy_utils::transform::TransformUtils;
use crate::{BevySystemSet, OptimaBevyTrait};
use crate::optima_bevy_utils::storage::BevyAnyHashmap;
use crate::optima_bevy_utils::viewport_visuals::ViewportVisualsActions;
use optima_proximity::shape_scene::ShapeSceneTrait;
use optima_proximity::shapes::OParryShape;
use optima_universal_hashmap::AHashMapWrapper;

pub struct RoboticsActions;
impl RoboticsActions {
    pub fn action_spawn_robot_as_stl_meshes<T: AD, C: O3DPoseCategory, L: OLinalgCategory + 'static>(robot: &ORobot<T, C, L>,
                                                                                                     fk_res: &FKResult<T, C::P<T>>,
                                                                                                     commands: &mut Commands,
                                                                                                     asset_server: &Res<AssetServer>,
                                                                                                     materials: &mut ResMut<Assets<StandardMaterial>>,
                                                                                                     robot_instance_idx: usize) {
        robot.links().iter().enumerate().for_each(|(link_idx, link)| {
            if link.is_present_in_model() {
                let stl_mesh_file_path = link.stl_mesh_file_path();
                if let Some(stl_mesh_file_path) = stl_mesh_file_path {
                    let asset_path_str = get_asset_path_str_from_ostemcellpath(&stl_mesh_file_path);
                    let link_pose = fk_res.get_link_pose(link_idx);
                    if let Some(link_pose) = link_pose {
                        let visual_offset = link.visual()[0].origin().pose();
                        let link_pose = link_pose.mul(visual_offset);

                        let transform = TransformUtils::util_convert_3d_pose_to_y_up_bevy_transform(&link_pose);

                        commands.spawn(PbrBundle {
                            mesh: asset_server.load(&asset_path_str),
                            material: materials.add(StandardMaterial::default()),
                            transform,
                            ..Default::default()
                        }).insert(LinkMeshID {
                            robot_instance_idx,
                            sub_robot_idx: link.sub_robot_idx(),
                            link_idx,
                        });
                    }
                }
            }
        });
    }
    pub fn action_set_state_of_robot<T: AD, C: O3DPoseCategory, L: OLinalgCategory + 'static, V: OVec<T>>(robot: &ORobot<T, C, L>,
                                                                                                          state: &V,
                                                                                                          robot_instance_idx: usize,
                                                                                                          query: &mut Query<(&LinkMeshID, &mut Transform)>) {
        let fk_res = robot.forward_kinematics(state, None);
        for (link_mesh_id, mut transform) in query.iter_mut() {
            let link_mesh_id: &LinkMeshID = &link_mesh_id;
            let transform: &mut Transform = &mut transform;

            if link_mesh_id.robot_instance_idx == robot_instance_idx {
                let link_idx = link_mesh_id.link_idx;
                let link = &robot.links()[link_idx];
                let pose = fk_res.get_link_pose(link_idx).as_ref().unwrap();
                let visual_offset = link.visual()[0].origin().pose();
                *transform = TransformUtils::util_convert_3d_pose_to_y_up_bevy_transform(&(pose.mul(visual_offset)));
            }
        }
    }
    pub fn action_robot_joint_sliders_egui<T: AD, C: O3DPoseCategory, L: OLinalgCategory + 'static>(robot: &ORobot<T, C, L>,
                                                                                                    robot_state_engine: &mut ResMut<RobotStateEngine>,
                                                                                                    egui_engine: &Res<OEguiEngineWrapper>,
                                                                                                    ui: &mut Ui) {
        let mut reset_clicked = false;
        ui.horizontal(|ui| {
            ui.heading("Joint Sliders");
            reset_clicked = ui.button("Reset").clicked();
        });
        ui.group(|ui| {
            egui::ScrollArea::new([true, true])
                .max_height(400.)
                .show(ui, |ui| {
                    robot.joints().iter().for_each(|joint| {
                        let dof_idxs = joint.dof_idxs();
                        for (i, dof_idx) in dof_idxs.iter().enumerate() {
                            let label = format!("joint_slider_dof_{}", dof_idx);
                            let lower = joint.limit().lower()[i];
                            let upper = joint.limit().upper()[i];

                            ui.separator();
                            ui.label(format!("DOF idx {}", dof_idx));
                            ui.label(format!("{}, sub dof {}", joint.name(), i));
                            ui.label(format!("Joint idx {}", joint.joint_idx()));
                            ui.label(format!("Joint type {:?}, Axis {:?}", joint.joint_type(), joint.axis()));
                            OEguiSlider::new(lower.to_constant(), upper.to_constant(), 0.0)
                                .show(&label, ui, &egui_engine, &());

                            let mut mutex_guard = egui_engine.get_mutex_guard();
                            let response = mutex_guard.get_slider_response_mut(&label).expect("error");

                            ui.horizontal(|ui| {
                                if ui.button("0.0").clicked() { response.slider_value = 0.0; }
                                if ui.button("+0.01").clicked() { response.slider_value += 0.01; }
                                if ui.button("-0.01").clicked() { response.slider_value -= 0.01; }
                                if ui.button("+0.1").clicked() { response.slider_value += 0.1; }
                                if ui.button("-0.1").clicked() { response.slider_value -= 0.1; }
                            });
                        }
                    });
                });
        });

        let mut mutex_guard = egui_engine.get_mutex_guard();

        let num_dofs = robot.num_dofs();
        let mut curr_state = vec![T::zero(); robot.num_dofs()];
        for i in 0..num_dofs {
            let label = format!("joint_slider_dof_{}", i);
            let response = mutex_guard.get_slider_response_mut(&label).expect("error");
            if reset_clicked { response.slider_value = 0.0; }
            let value = response.slider_value();
            curr_state[i] = T::constant(value);
        }

        robot_state_engine.add_update_request(0, &OVec::ovec_to_other_ad_type::<T>(&curr_state));
    }
    pub fn action_robot_link_vis_panel_egui<T: AD, C: O3DPoseCategory, L: OLinalgCategory + 'static>(robot: &ORobot<T, C, L>,
                                                                                                     robot_state_engine: &RobotStateEngine,
                                                                                                     lines: &mut ResMut<DebugLines>,
                                                                                                     egui_engine: &Res<OEguiEngineWrapper>,
                                                                                                     ui: &mut Ui) {
        let robot_state = robot_state_engine.get_robot_state(0);
        let robot_state = match robot_state {
            None => { return; }
            Some(robot_state) => { robot_state }
        };
        let robot_state = OVec::ovec_to_other_ad_type::<T>(robot_state);

        let fk_res = robot.forward_kinematics(&robot_state, None);

        let mut select_all = false;
        let mut deselect_all = false;
        ui.horizontal(|ui| {
            ui.heading("Link Panel");
            select_all = ui.button("select all").clicked();
            deselect_all = ui.button("deselect all").clicked();
        });

        ui.label("link axis display length");
        OEguiSlider::new(0.04, 1.0, 0.1)
            .show("link_axis_display_length", ui, egui_engine, &());

        ui.group(|ui| {
            egui::ScrollArea::new([true, true])
                .id_source("links_scroll_area")
                .max_height(400.0)
                .show(ui, |ui| {
                    robot.links().iter().enumerate().for_each(|(link_idx, link)| {
                        if link.is_present_in_model() {

                            let pose = fk_res.get_link_pose(link_idx).as_ref().unwrap();
                            let location = pose.translation();
                            let rotation = pose.rotation();
                            let scaled_axis = rotation.scaled_axis_of_rotation();
                            let unit_quaternion = rotation.unit_quaternion_as_wxyz_slice();
                            let euler_angles = rotation.euler_angles();
                            ui.label(format!("Link {}", link_idx));
                            ui.label(format!("{}", link.name()));
                            let toggle_label = format!("link_toggle_{}", link.name());
                            OEguiCheckbox::new("Show Coordinate Frame")
                                .show(&toggle_label, ui, &egui_engine, &());
                            ui.label(format!("Location: {:.2?}", location));
                            ui.label(format!("quaternion wxyz: {:.2?}", unit_quaternion));
                            ui.label(format!("scaled axis: {:.2?}", scaled_axis));
                            ui.label(format!("euler angles: {:.2?}", euler_angles));

                            let mut mutex_guard = egui_engine.get_mutex_guard();
                            let response = mutex_guard.get_checkbox_response_mut(&toggle_label).unwrap();
                            if select_all { response.currently_selected = true; }
                            if deselect_all { response.currently_selected = false; }

                            if response.currently_selected {
                                let draw_length = mutex_guard.get_slider_response("link_axis_display_length").unwrap().slider_value as f32;
                                let frame_vectors = rotation.coordinate_frame_vectors();
                                let x = &frame_vectors[0];
                                let x_as_vec = draw_length*Vec3::new(x[0].to_constant() as f32, x[1].to_constant() as f32, x[2].to_constant() as f32);
                                let y = &frame_vectors[1];
                                let y_as_vec = draw_length*Vec3::new(y[0].to_constant() as f32, y[1].to_constant() as f32, y[2].to_constant() as f32);
                                let z = &frame_vectors[2];
                                let z_as_vec = draw_length*Vec3::new(z[0].to_constant() as f32, z[1].to_constant() as f32, z[2].to_constant() as f32);

                                let location_as_vec = Vec3::new(location.x().to_constant() as f32, location.y().to_constant() as f32, location.z().to_constant() as f32);

                                ViewportVisualsActions::action_draw_gpu_line_optima_space(lines, location_as_vec, location_as_vec + x_as_vec, Color::rgb(1., 0., 0.), 4.0, 10, 1, 0.0);
                                ViewportVisualsActions::action_draw_gpu_line_optima_space(lines, location_as_vec, location_as_vec + y_as_vec, Color::rgb(0., 1., 0.), 4.0, 10, 1, 0.0);
                                ViewportVisualsActions::action_draw_gpu_line_optima_space(lines, location_as_vec, location_as_vec + z_as_vec, Color::rgb(0., 0., 1.), 4.0, 10, 1, 0.0);
                            }

                            ui.separator();
                        }
                    });
                });
        });


    }
}

pub struct RoboticsSystems;
impl RoboticsSystems {
    pub fn system_spawn_robot_links_as_stl_meshes<T: AD, C: O3DPoseCategory + 'static, L: OLinalgCategory + 'static>(robot: Res<BevyORobot<T, C, L>>,
                                                                                                                     mut commands: Commands,
                                                                                                                     asset_server: Res<AssetServer>,
                                                                                                                     mut materials: ResMut<Assets<StandardMaterial>>) {
        let robot = &robot.0;
        let num_dofs = robot.num_dofs();
        let fk_res = robot.forward_kinematics(&vec![T::zero(); num_dofs], None);
        RoboticsActions::action_spawn_robot_as_stl_meshes(robot, &fk_res, &mut commands, &asset_server, &mut materials, 0);
    }
    pub fn system_robot_state_updater<T: AD, C: O3DPoseCategory + 'static, L: OLinalgCategory + 'static>(robot: Res<BevyORobot<T, C, L>>,
                                                                                                         mut robot_state_engine: ResMut<RobotStateEngine>,
                                                                                                         mut query: Query<(&LinkMeshID, &mut Transform)>) {
        while robot_state_engine.robot_state_update_requests.len() > 0 {
            let robot = &robot.0;
            let request = robot_state_engine.robot_state_update_requests.pop().unwrap();
            let request_state: Vec<T> = request.1.iter().map(|x| T::constant(*x)).collect();
            robot_state_engine.robot_states.insert(request.0, OVec::ovec_to_other_ad_type::<f64>(&request_state));
            RoboticsActions::action_set_state_of_robot(robot, &request_state, request.0, &mut query);
        }
    }
    pub fn system_robot_main_info_panel_egui<T: AD, C: O3DPoseCategory + 'static, L: OLinalgCategory + 'static>(robot: Res<BevyORobot<T, C, L>>,
                                                                                                                mut lines: ResMut<DebugLines>,
                                                                                                                mut contexts: EguiContexts,
                                                                                                                mut robot_state_engine: ResMut<RobotStateEngine>,
                                                                                                                egui_engine: Res<OEguiEngineWrapper>,
                                                                                                                window_query: Query<&Window, With<PrimaryWindow>>) {
        OEguiSidePanel::new(Side::Left, 250.0)
            .show("joint_sliders_side_panel", contexts.ctx_mut(), &egui_engine, &window_query, &(), |ui| {
                egui::ScrollArea::new([true, true])
                    .show(ui, |ui| {
                        RoboticsActions::action_robot_joint_sliders_egui(&robot.0, &mut robot_state_engine, &egui_engine, ui);
                        ui.separator();
                        RoboticsActions::action_robot_link_vis_panel_egui(&robot.0, & *robot_state_engine, &mut lines, &egui_engine, ui);
                    });
            });
    }
    pub fn system_robot_motion_interpolator<T: AD, V: OVec<T>, I: InterpolatorTrait<T, V> + 'static>(interpolator: Res<BevyRobotInterpolator<T, V, I>>,
                                                                                                     mut contexts: EguiContexts,
                                                                                                     mut robot_state_engine: ResMut<RobotStateEngine>,
                                                                                                     mut h: ResMut<BevyAnyHashmap>,
                                                                                                     egui_engine: Res<OEguiEngineWrapper>,
                                                                                                     time: Res<Time>,
                                                                                                     window_query: Query<&Window, With<PrimaryWindow>>) {
        OEguiTopBottomPanel::new(TopBottomSide::Bottom, 100.0)
            .show("interpolator_bottom_pannel", contexts.ctx_mut(), &egui_engine, &window_query, &(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Playback Slider: ");
                    OEguiSlider::new(0.0, interpolator.0.max_t().to_constant(), 0.0)
                        .show("playback_slider", ui, &egui_engine, &());

                    let playing = h.0.get_or_insert(&"playing".to_string(), false).clone();
                    let button_str = match playing {
                        true => { "⏸" }
                        false => { "⏵" }
                    };

                    OEguiButton::new(button_str)
                        .show("play_stop", ui, &egui_engine, &());

                    ui.label("Speed Slider: ");
                    OEguiSlider::new(0.0, 3.0, 1.0)
                        .show("speed_slider", ui, &egui_engine, &());

                    let binding = egui_engine.get_mutex_guard();
                    let response = binding.get_button_response("play_stop").unwrap();
                    if response.widget_response().clicked() { h.0.insert("playing".to_string(), !playing); }
                    drop(binding);

                    if playing {
                        let mut binding = egui_engine.get_mutex_guard();
                        let response2 = binding.get_slider_response("speed_slider").unwrap();
                        let speed = response2.slider_value.clone();
                        let response = binding.get_slider_response_mut("playback_slider").unwrap();
                        response.slider_value += speed * time.delta_seconds_f64();
                        if response.slider_value > interpolator.0.max_t().to_constant() { response.slider_value = 0.0; }
                    }
                });
            });

        let binding = egui_engine.get_mutex_guard();
        let slider_result = binding.get_slider_response("playback_slider");
        if let Some(slider_result) = slider_result {
            if slider_result.widget_response().dragged() { h.0.insert("playing".to_string(), false); }

            let slider_value = slider_result.slider_value;

            let state = interpolator.0.interpolate(T::constant(slider_value));
            robot_state_engine.add_update_request(0, &state);
        }
    }
    pub fn system_robot_self_collision_vis<T: AD, C: O3DPoseCategory + 'static, L: OLinalgCategory + 'static>(mut robot: ResMut<BevyORobot<T, C, L>>,
                                                                                                              mut robot_state_engine: ResMut<RobotStateEngine>,
                                                                                                              mut contexts: EguiContexts,
                                                                                                              egui_engine: Res<OEguiEngineWrapper>,
                                                                                                              keys: Res<Input<KeyCode>>,
                                                                                                              window_query: Query<&Window, With<PrimaryWindow>>) {
        OEguiSidePanel::new(Side::Left, 300.0)
            .show("side_panel", contexts.ctx_mut(), &egui_engine, &window_query, &(), |ui| {
                egui::ScrollArea::new([true, true])
                    .show(ui, |ui| {
                        RoboticsActions::action_robot_joint_sliders_egui(&robot.0, &mut robot_state_engine, &egui_engine, ui);

                        ui.group(|ui| {
                            let state = robot_state_engine.get_robot_state(0);
                            if let Some(state) = state {
                                let state = OVec::ovec_to_other_ad_type::<T>(state);
                                // let p = robot.0.parry_shape_scene().get_shape_poses(&(&robot.0, &state));
                                let p = robot.0.get_shape_poses(&state);
                                let s = robot.0.parry_shape_scene().get_shapes();
                                let skips = robot.0.parry_shape_scene().get_pair_skips();
                                let a = robot.0.parry_shape_scene().get_pair_average_distances();

                                let binding = egui_engine.get_mutex_guard();
                                let parry_pair_selector_response = binding.get_selector_response("selector1");
                                let parry_shape_rep_response = binding.get_selector_response("selector2");

                                if let (Some(parry_pair_selector_response), Some(parry_shape_rep_response)) = (parry_pair_selector_response, parry_shape_rep_response) {
                                    let p1 = parry_pair_selector_response.current_selections::<OParryPairSelector>();
                                    let p2 = parry_shape_rep_response.current_selections::<ParryShapeRep>();

                                    // let fr = ParryIntersectGroupSequenceFilter::query(s, s, p.as_ref(), p.as_ref(), &ParryPairSelector::HalfPairs, skips, a, &ParryIntersectGroupSequenceFilterArgs::new(vec![], vec![]));
                                    let res = OParryIntersectGroupQry::query(s, s, p.as_ref(), p.as_ref(), &p1[0], skips, &(), false, &OParryIntersectGroupArgs::new(p2[0].clone(), p2[0].clone(), false, false));

                                    // let fr = ParryDistanceGroupSequenceFilter::query(s, s, p.as_ref(), p.as_ref(), &ParryPairSelector::HalfPairs, skips, a, &ParryDistanceGroupSequenceFilterArgs::new(vec![], vec![], T::constant(0.6), true, ParryDisMode::ContactDis));
                                    let res2 = OParryDistanceGroupQry::query(s, s, p.as_ref(), p.as_ref(), &p1[0], skips, a, false, &OParryDistanceGroupArgs::new(p2[0].clone(), p2[0].clone(), ParryDisMode::ContactDis, true, false, T::constant(f64::MIN), true));

                                    let proximity_objective_value = res2.get_proximity_objective_value(T::constant(0.6), T::constant(20.0), OProximityLossFunction::Hinge);

                                    let intersect = res.intersect();
                                    ui.heading(format!("In collision: {:?}", intersect));
                                    ui.label(format!("Min. dis. with respect to average: {:.3}", res2.min_dis_wrt_average()));
                                    ui.label(format!("Proximity objective value:         {:.3}", proximity_objective_value));

                                    ui.separator();
                                    ui.separator();

                                    if ui.button("Mark as non-collision state").clicked() {
                                        if intersect {
                                            robot.0.add_non_collision_state(state.clone(), SaveRobot::Save(None));
                                        }
                                    }

                                    ui.separator();
                                    ui.separator();

                                    if ui.button("Clear non-collision states").clicked() {
                                        robot.0.reset_non_collision_states(SaveRobot::Save(None));
                                    }

                                    ui.separator();
                                    ui.separator();

                                    drop(binding);
                                    ui.label("Any distances wrt average ");
                                    ui.label("less than this value will ");
                                    ui.label("be skipped. ");
                                    OEguiSlider::new(0.0, 2.0, 0.5)
                                        .show("distance_threshold", ui, &egui_engine, &());

                                    let binding = egui_engine.get_mutex_guard();
                                    let response = binding.get_slider_response("distance_threshold").expect("error");

                                    ui.separator();
                                    ui.separator();

                                    if ui.button("Mark as close proximity state").clicked() {
                                        robot.0.add_close_proximity_state(state.clone(), T::constant(response.slider_value), SaveRobot::Save(None));
                                    }

                                    ui.separator();
                                    ui.separator();

                                    if ui.button("Clear close proximity states").clicked() {
                                        robot.0.reset_close_proximity_states(SaveRobot::Save(None));
                                    }

                                    ui.separator();
                                    ui.separator();
                                }
                            }

                            ui.group(|ui| {
                                OEguiSelector::new(OEguiSelectorMode::Checkboxes, vec![OParryPairSelector::HalfPairs, OParryPairSelector::HalfPairsSubcomponents], vec![OParryPairSelector::HalfPairsSubcomponents], None, false)
                                    .show("selector1", ui, &egui_engine, &*keys);
                                ui.separator();
                                OEguiSelector::new(OEguiSelectorMode::Checkboxes, vec![ParryShapeRep::BoundingSphere, ParryShapeRep::OBB, ParryShapeRep::Full], vec![ParryShapeRep::Full], None, false)
                                    .show("selector2", ui, &egui_engine, &*keys);
                            });
                        });
                    });
            });
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait BevyRoboticsTrait<T: AD> {
    fn bevy_display(&self);
    fn bevy_get_display_app(&self) -> App;
    fn bevy_motion_playback<V: OVec<T>, I: InterpolatorTrait<T, V> + 'static>(&self, interpolator: &I);
    fn bevy_get_motion_playback_app<V: OVec<T>, I: InterpolatorTrait<T, V> + 'static>(&self, interpolator: &I) -> App;
    fn bevy_self_collision_visualization(&mut self);
    fn bevy_get_self_collision_visualization_app(&mut self) -> App;
}

impl<T: AD, C: O3DPoseCategory + 'static, L: OLinalgCategory + 'static> BevyRoboticsTrait<T> for ORobot<T, C, L> {
    fn bevy_display(&self) {
        self.bevy_get_display_app().run();
    }

    fn bevy_get_display_app(&self) -> App {
        let mut app = App::new();
        app
            .optima_bevy_base()
            .optima_bevy_robotics_base(self.clone())
            .optima_bevy_pan_orbit_camera()
            .optima_bevy_starter_lights()
            .optima_bevy_spawn_robot::<T, C, L>()
            .optima_bevy_robotics_scene_visuals_starter()
            .optima_bevy_egui()
            .add_systems(Update, RoboticsSystems::system_robot_main_info_panel_egui::<T, C, L>.before(BevySystemSet::Camera));
        app
    }

    fn bevy_motion_playback<V: OVec<T>, I: InterpolatorTrait<T, V> + 'static>(&self, interpolator: &I) {
        self.bevy_get_motion_playback_app(interpolator).run();
    }

    fn bevy_get_motion_playback_app<V: OVec<T>, I: InterpolatorTrait<T, V> + 'static>(&self, interpolator: &I) -> App {
        let mut app = App::new();
        app
            .optima_bevy_base()
            .optima_bevy_robotics_base(self.clone())
            .optima_bevy_pan_orbit_camera()
            .optima_bevy_starter_lights()
            .optima_bevy_spawn_robot::<T, C, L>()
            .optima_bevy_robotics_scene_visuals_starter()
            .optima_bevy_egui()
            .insert_resource(BevyRobotInterpolator(interpolator.clone(), PhantomData::default()))
            .add_systems(Update, RoboticsSystems::system_robot_motion_interpolator::<T, V, I>.before(BevySystemSet::Camera));
        app
    }

    fn bevy_self_collision_visualization(&mut self) {
        self.bevy_get_self_collision_visualization_app().run();
    }

    fn bevy_get_self_collision_visualization_app(&mut self) -> App {
        assert!(self.has_been_preprocessed(), "robot must be preprocessed first.");
        let mut app = App::new();
        app
            .optima_bevy_base()
            .optima_bevy_robotics_base(self.clone())
            .optima_bevy_pan_orbit_camera()
            .optima_bevy_starter_lights()
            .optima_bevy_spawn_robot::<T, C, L>()
            .optima_bevy_robotics_scene_visuals_starter()
            .optima_bevy_egui()
            .add_systems(Update, RoboticsSystems::system_robot_self_collision_vis::<T, C, L>.before(BevySystemSet::Camera));
        app
    }
}

/*
impl<T: AD, C: O3DPoseCategory + 'static, L: OLinalgCategory + 'static> BevyRoboticsTrait<T> for ORobotSet<T, C, L> {
    fn bevy_display(&self) {
        self.as_robot().bevy_display();
    }

    fn get_bevy_display_app(&self) -> App {
        self.as_robot().get_bevy_display_app()
    }

    fn bevy_motion_playback<V: OVec<T>, I: InterpolatorTrait<T, V> + 'static>(&self, interpolator: &I) {
        self.as_robot().bevy_motion_playback(interpolator);
    }

    fn get_bevy_motion_playback_app<V: OVec<T>, I: InterpolatorTrait<T, V> + 'static>(&self, interpolator: &I) -> App {
        todo!()
    }

    fn bevy_self_collision_visualization(&mut self) {
        panic!("not handled for RobotSet");
    }

    fn get_bevy_self_collision_visualization_app(&mut self) -> App { panic!("not handled for RobotSet"); }
}
*/

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Component)]
pub struct LinkMeshID {
    pub robot_instance_idx: usize,
    pub sub_robot_idx: usize,
    pub link_idx: usize
}

#[derive(Resource)]
pub struct RobotStateEngine {
    pub (crate) robot_states: HashMap<usize, Vec<f64>>,
    pub (crate) robot_state_update_requests: Vec<(usize, Vec<f64>)>
}
impl RobotStateEngine {
    pub fn new() -> Self {
        Self { robot_states: Default::default(), robot_state_update_requests: vec![] }
    }
    pub fn add_update_request<T: AD, V: OVec<T>>(&mut self, robot_instance_idx: usize, state: &V) {
        let save_state = state.to_constant_vec();
        self.robot_state_update_requests.push( (robot_instance_idx, save_state) );
    }
    pub fn get_robot_state(&self, robot_instance_idx: usize) -> Option<&Vec<f64>> {
        self.robot_states.get(&robot_instance_idx)
    }
}

#[derive(Resource)]
pub struct BevyORobot<T: AD, C: O3DPoseCategory + Send + 'static, L: OLinalgCategory + 'static>(pub ORobot<T, C, L>, pub usize);
impl<T: AD, C: O3DPoseCategory + Send + 'static, L: OLinalgCategory + 'static> ShapeSceneTrait<T, C::P<T>> for BevyORobot<T, C, L> {
    type ShapeType = OParryShape<T, C::P<T>>;
    type GetPosesInput = Vec<T>;
    type PairSkipsType = AHashMapWrapper<(u64, u64), Vec<OSkipReason>>;

    #[inline(always)]
    fn get_shapes(&self) -> &Vec<Self::ShapeType> {
        self.0.parry_shape_scene().get_shapes()
    }

    #[inline(always)]
    fn get_shape_poses<'a>(&'a self, input: &'a Self::GetPosesInput) -> Cow<'a, Vec<C::P<T>>> {
        self.0.get_shape_poses(input)
    }

    fn sample_pseudorandom_input(&self) -> Self::GetPosesInput {
        self.0.sample_pseudorandom_state()
    }

    #[inline(always)]
    fn get_pair_skips(&self) -> &Self::PairSkipsType {
        self.0.parry_shape_scene().get_pair_skips()
    }

    #[inline(always)]
    fn shape_id_to_shape_str(&self, id: u64) -> String {
        self.0.parry_shape_scene().shape_id_to_shape_str(id)
    }
}

#[derive(Resource)]
pub struct BevyRobotInterpolator<T: AD, V: OVec<T>, I: InterpolatorTrait<T, V> + 'static>(pub I, PhantomData<(T, V)>);
unsafe impl<T: AD, V: OVec<T>, I: InterpolatorTrait<T, V>> Send for BevyRobotInterpolator<T, V, I> { }
unsafe impl<T: AD, V: OVec<T>, I: InterpolatorTrait<T, V>> Sync for BevyRobotInterpolator<T, V, I> { }

